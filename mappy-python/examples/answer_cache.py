"""A query-scoped answer cache backed by the embedded mappy engine."""

from __future__ import annotations

import base64
import hashlib
import json
from pathlib import Path
from typing import Iterable

try:
    import mappy_python
except ModuleNotFoundError:
    mappy_python = None


DEFAULT_CAPACITY = 100_000
DEFAULT_TTL_FLOOR_SECONDS = 2


class AnswerCache:
    """Store exact replies in mappy and reject replies from a different query."""

    def __init__(
        self,
        data_dir: str | Path,
        persistence_mode: str = "hybrid",
        capacity: int = DEFAULT_CAPACITY,
        ttl_floor_seconds: int = DEFAULT_TTL_FLOOR_SECONDS,
    ) -> None:
        if mappy_python is None:
            raise RuntimeError("mappy_python is not importable")
        config = mappy_python.PyEngineConfig(
            capacity,
            0.01,
            persistence_mode,
            str(data_dir),
            None,
            1000,
            True,
            1000,
        )
        self.engine = mappy_python.PyEngine(config)
        self.ttl_floor_seconds = ttl_floor_seconds

    def _key(self, query: str) -> str:
        return "answer:" + hashlib.sha256(query.encode("utf-8")).hexdigest()

    def get(self, query: str) -> bytes | None:
        raw = self.engine.get(self._key(query))
        if raw is None:
            return None
        try:
            envelope = json.loads(raw)
            if envelope["query"] != hashlib.sha256(query.encode("utf-8")).hexdigest():
                return None
            return base64.b64decode(envelope["reply"])
        except (KeyError, TypeError, ValueError):
            return None

    def set(self, query: str, reply: bytes, ttl_seconds: int) -> None:
        envelope = {
            "query": hashlib.sha256(query.encode("utf-8")).hexdigest(),
            "reply": base64.b64encode(reply).decode("ascii"),
        }
        self.engine.set(self._key(query), json.dumps(envelope).encode("utf-8"))
        self.engine.expire(self._key(query), max(ttl_seconds, self.ttl_floor_seconds))

    def replay(self, entries: Iterable[tuple[str, bytes, int]]) -> int:
        count = 0
        for query, reply, ttl_seconds in entries:
            self.set(query, reply, ttl_seconds)
            count += 1
        return count

    def flush(self) -> None:
        self.engine.flush()

    def close(self) -> None:
        self.engine.close()
