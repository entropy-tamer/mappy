# Offline AOF compaction

`mappy-aof-compact` rewrites a mappy append-only file (AOF) into the smallest equivalent log: one `Set` record for each live key, in original record order. Deleted keys disappear. It is an offline tool. Stop the service that owns the AOF before running it.

## What it does

The compactor makes two sequential passes and never loads vector values into memory.

1. The first pass reads each JSON line and records the byte offset of the latest `Set` for each key. A `Delete` removes that key from the pending set.
2. The second pass copies only those surviving lines into a temporary file in the same directory. The file is flushed and synced, then renamed over the original with `std::fs::rename`. The parent directory is synced as well.

Memory use scales with the number of distinct keys, not with the number of historical writes or the size of each value. CPU use is one parse pass and one copy pass; there are no worker threads and no random seeks.

`--dry-run` reports the same metrics without writing to the file, so the shape of a compaction can be checked while the service is still running. The tool requires `--offline` as an explicit confirmation that the writer is stopped. Before the original is replaced, it creates a hard-link backup at `<stem>.aof.pre-compact`. A hard link costs no meaningful disk space and keeps the original inode intact. The tool refuses to overwrite an existing backup unless `--overwrite-backup` is passed. Use `--skip-backup` only when disk space is critically short and another backup exists.

On Linux the tool scans `/proc/*/fd` before touching the file. If any process still holds the AOF open, it refuses with a short list of process IDs. This check is a guard, not a distributed lock. Stop the writer regardless of what the scan reports.

## Usage

```sh
cargo build --release -p mappy-aof-compact
systemctl --user stop remi-daemon.service remi-watcher.service
./target/release/mappy-aof-compact <mappy-path>/embedding_cache/mappy.aof --offline
systemctl --user start remi-watcher.service remi-daemon.service
```

On the mesh, [compact-mappy-aof.yml](file:///home/kade/ansible/playbooks/compact-mappy-aof.yml) wraps the whole window and waits for the wg0 health endpoint after restart.

The command prints one report line with these fields:

- `input_bytes`, `output_bytes`: file sizes before and after.
- `records_read`, `records_written`: JSON lines scanned and retained.
- `keys_retained`: distinct live keys kept in the rewrite.
- `tombstones_dropped`: `Delete` records consumed by the rewrite.
- The hard-linked original stays at `<stem>.aof.pre-compact`.

A truncated final line is treated as the tail of an interrupted append and ignored. A malformed line anywhere else aborts the run before the original is changed.

## Recovery

If the compacted AOF does not reload cleanly, stop the service and restore the backup:

```sh
systemctl --user stop remi-daemon.service remi-watcher.service
mv <mappy-path>/embedding_cache/mappy.aof.pre-compact <mappy-path>/embedding_cache/mappy.aof
systemctl --user start remi-watcher.service remi-daemon.service
```

Keep the backup until the service has loaded the compacted file and served at least one query.

## Tests

`cargo test -p mappy-core --lib storage::aof::tests::test_compact_aof` covers last-value-wins semantics, deletes, malformed input, and backup behavior. `cargo test -p mappy-aof-compact --test compact_cli` covers the dry run, the explicit confirmation guard, and the CLI path end to end, then loads the compacted file through `AOFStorage` and checks the surviving values.
