# Mappy

![A red fox in a forest, holding a map divided into rectangular sections. The word MAPPY is drawn in large orange letters.](docs/mappy_header_text.webp)

Mappy is a Rust implementation of **maplets**, the approximate key-value structure described by Bender, Conway, Farach-Colton, Johnson and Pandey in *Time To Replace Your Filter: How Maplets Simplify System Design*. A maplet keeps a fingerprint of each key alongside its value, at `O(log 1/ε + v)` bits per item for false-positive rate ε and value size v; a Bloom filter needs `O(log 1/ε)` bits for the key alone, so the extra cost is the value. Reads are fast and usually exact; when two fingerprints collide, the value returned is the true value combined with the colliding values through an associative operator the application supplies. For a counter the value can only be too large; for a set it can only be a superset; for a maximum it can only be an upper bound. That is the one-sided error the paper promises, and it is why an approximate structure can still answer value queries.

The rest of this file covers the workspace, the two library entry points, the network services, and the places where the code has moved ahead of the older documentation.

## The workspace

Mappy is a Cargo workspace with one crate per job. `mappy-core` holds the maplet, the quotient filter behind it, the merge operators, the `Engine`, the memory, disk, append-only, and hybrid storage backends, and TTL support. `mappy-client` is the network client; it speaks to a running service over a Unix socket or HTTP. `mappy-server` is the Axum service that exposes engine operations, health and metrics endpoints, and a server-sent error stream. `mappy-daemon` is a smaller socket-only server built on smol, with a compact line protocol, for cases where the full HTTP surface is unnecessary. `mappy-proxy` forwards raw TCP bytes to the daemon's Unix socket for clients on another machine. `mappy-python` wraps the maplet and engine with PyO3 and builds as a maturin extension module.

The older `mappy-python-client` and `python-mappy-ml` directories are not workspace members and are not built by CI. Everything in the workspace uses edition 2024 and Rust 1.101 or newer.

## The maplet

The maplet is the small, self-contained structure. It takes the key and value types and a merge operator, and it returns approximate values.

```rust
use std::sync::Arc;
use mappy_core::{CounterOperator, Maplet, MapletResult};

async fn count_page_views() -> MapletResult<()> {
    let counts = Arc::new(Maplet::<String, u64, CounterOperator>::new(10_000, 0.01)?);

    Arc::clone(&counts).insert("page:home".to_string(), 1).await?;
    Arc::clone(&counts).insert("page:home".to_string(), 1).await?;

    let home = Arc::clone(&counts).query("page:home".to_string()).await;
    let stats = Arc::clone(&counts).stats().await;

    println!("home = {home:?}, load = {:.1}%", stats.load_factor * 100.0);
    Ok(())
}
```

Operations take `Arc<Self>` and are asynchronous because the filter and the value map sit behind async locks. Cloning the `Arc` is the idiomatic way to keep using the handle after a call, and the runtime is the caller's choice: `mappy-core` defaults to the `smol` feature and can be built with `tokio` instead. The `quotient-filter` feature is enabled by default and adds `find_slot_for_key`, which walks runs and shifts to report the physical slot holding a key's fingerprint. That is an inspection and debugging tool; ordinary reads and writes do not need it.

## The engine

Most applications want the `Engine`; the bare maplet is the lower-level piece. The engine stores exact bytes in a storage backend and uses the maplet as a membership prefilter, so `get` returns `None` as soon as the filter says no and only reaches storage on a maybe. It also adds TTL, persistence and statistics.

```rust
use mappy_core::{Engine, EngineConfig};

async fn store_a_value() -> mappy_core::MapletResult<()> {
    let engine = Engine::new(EngineConfig::default()).await?;

    engine.set("user:123", b"Alice".to_vec()).await?;
    assert_eq!(engine.get("user:123").await?, Some(b"Alice".to_vec()));

    Ok(())
}
```

`PersistenceMode` selects how much durability you want: `Memory` keeps everything in RAM, `Disk` uses Sled, `AOF` appends writes to a file, and `Hybrid` combines an in-memory cache with an append-only log. On startup the engine rebuilds the maplet from persistent storage so membership survives a restart. `expire` and `ttl` add per-key lifetimes, with a background cleanup task. See [offline AOF compaction](docs/AOF_COMPACTION.md) when an append-only log needs to be rewritten.

## The network services

The three network binaries cover different deployment shapes. `mappy-server` listens on the Unix socket at `/var/run/reynard/mappy.sock` by default and leaves HTTP disabled until `MAPPY_ENABLE_HTTP=true` is set; when enabled, HTTP defaults to `0.0.0.0:8003`. Its routes cover `/health`, `/status`, `/set`, `/get/{key}`, metrics, and an error log with an SSE stream.

`mappy-daemon` is the leaner option. It binds `$XDG_RUNTIME_DIR/mappy/mappy.sock` unless `MAPPY_SOCKET_PATH` says otherwise, sets the socket to mode `0600`, and speaks a line protocol with `PING`, `SET`, `SETEX`, `GET`, `DEL`, `KEYS`, `STATS`, `HEALTH`, and `QUIT`. `mappy-proxy` listens on `127.0.0.1:6380` by default and pipes bytes to that socket, which is enough for a remote client to use the same protocol without an HTTP translation layer.

The server and daemon read their knobs from the environment: `MAPPY_CAPACITY`, `MAPPY_FALSE_POSITIVE_RATE`, `MAPPY_DATA_DIR`, `MAPPY_PERSISTENCE_MODE`, and `RUST_LOG` are the common ones. The proxy uses `MAPPY_PROXY_LISTEN` and `MAPPY_SOCKET_PATH`. The defaults are conservative and in-memory, so a service started without configuration will not write to disk by accident.

## Python

The Python module mirrors the two Rust entry points. `PyMaplet` is a `u64`-valued approximate map, and `PyEngine` exposes the durable engine with bytes values, TTL, persistence and statistics.

```python
import mappy_python

counts = mappy_python.PyMaplet(capacity=10_000, false_positive_rate=0.01)
counts.insert("page:home", 1)
counts.insert("page:home", 1)

print(counts.query("page:home"))  # 2
```

Build the extension from `mappy-python` with `maturin develop --release`, or `maturin build --release` for a wheel. The crate is a `cdylib` named `mappy_python`, and the PyO3 module of the same name is what the import above resolves.

## Benchmarks

Criterion benchmarks live in `mappy-core/benches` and run with `cargo bench -p mappy-core`. They cover insertion, query, deletion, slot finding, hash-function comparisons, storage backends, and a maplet-versus-standard-collections comparison.

The Redis comparison and the machine-learning tables that appeared in earlier revisions of this file were measured against a different repository layout. The ML harness now lives in the separate [stilts](https://git.sly.so/kade/stilts) project, and the benchmark script paths printed in those revisions no longer exist. Treat the old numbers as historical; re-run the benches before quoting any figure.

## Building and testing

```bash
cargo build --release
cargo test --release
cargo clippy --workspace --all-targets --release -- -D warnings -W clippy::pedantic
cargo fmt --all --check
```

The dependencies are git dependencies hosted on the `git.sly.so` Forgejo instance, so the first build needs network access to that host. The CI pipeline runs the same sequence and adds a dependency-pact check over `Cargo.lock`.

## Status and limitations

Two parts of the public surface are thinner than their names suggest. `Maplet::merge` returns `MergeFailed`; merge operators are applied within a single maplet when fingerprints collide, but combining two maplets is not implemented. `Maplet::resize` swaps in a fresh quotient filter without reinserting the existing fingerprints, so keys that were already stored stop being found through the filter, although their values remain in the map. Until that is fixed, size the maplet up front or leave `auto_resize` off. The engine is not immune: it inserts into a maplet, so more keys than its configured capacity will reach the same path.

The documents under `docs/` predate the workspace split. `docs/API.md` still shows configuration fields such as `memory_capacity` and `aof_sync_interval_ms` that the code does not have, and the component READMEs describe earlier network designs. The source and this file are the current reference; the technical guide in `docs/TECHNICAL_README.md` remains useful for the model and the internals, with that caveat.

## Documentation

The technical guide at [docs/TECHNICAL_README.md](docs/TECHNICAL_README.md) covers the maplet model, the quotient filter, and the storage layer in depth. [docs/QUOTIENT_FILTER.md](docs/QUOTIENT_FILTER.md) collects the slot-finding details and benchmark notes, and [mappy-core/benches/README.md](mappy-core/benches/README.md) explains the benchmark harness. Generate the Rust API documentation with `cargo doc --workspace --no-deps`.

## License

MIT. See [LICENSE](LICENSE).

## References

> Bender, M. A., Conway, A., Farach-Colton, M., Johnson, R., & Pandey, P. (2025). Time To Replace Your Filter: How Maplets Simplify System Design. arXiv preprint [arXiv:2510.05518](https://arxiv.org/abs/2510.05518).
