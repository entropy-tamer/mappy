from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import answer_cache


@unittest.skipIf(answer_cache.mappy_python is None, "mappy_python is not built")
class AnswerCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.data_dir = Path(self.tmp.name) / "state"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_round_trip(self) -> None:
        cache = answer_cache.AnswerCache(self.data_dir)
        try:
            cache.set("suggest:read the", b'["file"]', 60)
            self.assertEqual(cache.get("suggest:read the"), b'["file"]')
        finally:
            cache.close()

    def test_reply_from_another_query_is_rejected(self) -> None:
        cache = answer_cache.AnswerCache(self.data_dir)
        try:
            cache.set("suggest:read the", b'["file"]', 60)
            self.assertIsNone(cache.get("suggest:write the"))
        finally:
            cache.close()

    def test_ttl_floor(self) -> None:
        cache = answer_cache.AnswerCache(self.data_dir, ttl_floor_seconds=2)
        try:
            cache.set("suggest:short", b"[]", 0)
            self.assertGreaterEqual(cache.engine.ttl(cache._key("suggest:short")), 2)
        finally:
            cache.close()

    def test_replay_is_idempotent_after_restart(self) -> None:
        first = answer_cache.AnswerCache(self.data_dir)
        first.set("suggest:restart", b"[]", 600)
        first.flush()
        first.close()

        second = answer_cache.AnswerCache(self.data_dir)
        try:
            self.assertEqual(second.replay([("suggest:restart", b"[]", 600)]), 1)
            self.assertEqual(second.get("suggest:restart"), b"[]")
            self.assertEqual(second.replay([("suggest:restart", b"[]", 600)]), 1)
            self.assertEqual(second.get("suggest:restart"), b"[]")
        finally:
            second.close()


if __name__ == "__main__":
    unittest.main()
