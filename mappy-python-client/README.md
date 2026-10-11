# mappy-python-client — deprecated, do not use

`mappy_client.py` in this directory speaks **HTTP over a unix socket**. That is
the wire protocol of `mappy-server`.

The only mappy service running locally is **`mappy-daemon`**, which speaks a
**length-prefixed line protocol**. The two do not interoperate, and the failure
mode is worse than an error — it is a hang:

```python
# MappyClient._request_socket reads until the peer closes the connection.
while True:
    chunk = sock.recv(4096)
    if not chunk:
        break
    response += chunk
```

`mappy-daemon` never closes the connection; it is waiting for the next command.
So `health()` blocks forever against a perfectly healthy daemon — no exception,
no timeout, nothing to catch. Verified on 2026-10-04: a `health()` call against
the running user-service daemon had to be killed by a 5s timeout.

```console
$ timeout 5 python3 -c "...MappyClient(socket_path='/run/user/1000/mappy/mappy.sock').health()"
exit=124   # 124 = timeout killed it; the daemon was up the whole time
```

## Use one of these instead

- **`forgejo_client.mappy_socket.MappySocketClient`** — the maintained Python
  line-protocol client. Resolves the socket path from `MAPPY_SOCKET_PATH` or
  `$XDG_RUNTIME_DIR`, carries a 5s socket timeout, and supports `GET`, `SET`,
  `SETEX`, `DEL`, `KEYS`, `STATS`, `PING`. Tested in
  `forgejo-client/tests/test_mappy_cache.py`.
  See [`~/src/forgejo-client/src/forgejo_client/mappy_socket.py`](file:///home/kade/src/forgejo-client/src/forgejo_client/mappy_socket.py).
- **`mappy_client::MappyClient`** (Rust) — the upstream async client in
  [`../mappy-client/`](file:///home/kade/src/mappy/mappy-client), if a typed
  client is wanted.

## Status

Nothing in this repository imports `mappy_client.py`. The HTTP router it targets
is not deployed anywhere: [`~/ansible/roles/mappy`](file:///home/kade/ansible/roles/mappy/defaults/main.yml)
installs `mappy-daemon`, "smol, socket-only, line protocol", and
[`deploy-mappy.yml`](file:///home/kade/ansible/playbooks/deploy-mappy.yml) runs it
as `reynard-mappy.service` on the mesh servers.

The file is kept only as a reference for the old `mappy-server` wire format.
