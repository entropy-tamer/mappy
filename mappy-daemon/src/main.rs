//! mappy-daemon: a silly, fast, semi-unreliable KV store.
//!
//! A smol-based Unix-socket server (socket-only, no TCP) backed by mappy-core's
//! `Engine`. With mappy-core's default `smol` feature, the Engine runs natively
//! on the smol executor — no runtime bridging needed.
//!
//! ## Line protocol (UTF-8 commands, values are length-framed raw bytes)
//!
//! ```text
//! PING                              -> PONG
//! SET <key> <len>\n<bytes>\n        -> OK
//! SETEX <key> <ttl> <len>\n<bytes>\n -> OK
//! GET <key>                         -> VAL <len>\n<bytes>\n | NIL
//! DEL <key>                         -> OK <n>            (n = 0 or 1)
//! KEYS <pattern>                    -> KEYS <n>\n + n frames "<len>\n<bytes>\n"
//! STATS                             -> <json line>
//! HEALTH                            -> OK\n
//! QUIT                              -> close
//! ```
//!
//! Keys must not contain spaces. `*` and `?` are supported as globs in KEYS.

use anyhow::Context as _;
use async_net::unix::UnixListener;
use futures_lite::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use mappy_core::types::MapletConfig;
use mappy_core::{Engine, EngineConfig, PersistenceMode};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Glob matcher (classic O(n) wildcard: '*' = any sequence, '?' = one char)
// ---------------------------------------------------------------------------

fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(sp) = star {
            pi = sp + 1;
            ti = mark + 1;
            mark += 1;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

// ---------------------------------------------------------------------------
// Connection handler
// ---------------------------------------------------------------------------

async fn handle_conn(stream: async_net::unix::UnixStream, engine: Arc<Engine>) {
    let read_stream = stream.clone();
    let mut reader = BufReader::new(read_stream);
    let mut writer = stream;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                tracing::debug!("read error: {e}");
                break;
            }
        }
        let cmd = line.trim_end_matches(['\r', '\n']);
        if cmd.is_empty() {
            continue;
        }
        if !handle_command(cmd, &mut reader, &mut writer, &engine).await {
            break;
        }
    }
}

/// The buffered reader half of a client connection.
type ClientReader = BufReader<async_net::unix::UnixStream>;

/// The raw writer half of a client connection.
type ClientWriter = async_net::unix::UnixStream;

/// Returns `false` to close the connection (QUIT).
async fn handle_command(
    cmd: &str,
    reader: &mut ClientReader,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) -> bool {
    let mut parts = cmd.split(' ');
    let verb = parts.next().unwrap_or("");

    match verb {
        "PING" => write_line(writer, "PONG").await,

        "SET" => handle_set(&mut parts, reader, writer, engine).await,

        "SETEX" => handle_setex(&mut parts, reader, writer, engine).await,

        "GET" => handle_get(&mut parts, writer, engine).await,

        "DEL" => handle_del(&mut parts, writer, engine).await,

        "KEYS" => handle_keys(&mut parts, writer, engine).await,

        "STATS" => handle_stats(writer, engine).await,

        "HEALTH" => write_line(writer, "OK").await,

        "QUIT" => return false,

        _ => write_line(writer, "ERR unknown command").await,
    }
    true
}

/// `SET <key> <len>` followed by `<len>` value bytes.
async fn handle_set(
    parts: &mut impl Iterator<Item = &str>,
    reader: &mut ClientReader,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) {
    let Some((key, len)) = next_key_len(parts) else {
        write_line(writer, "ERR usage: SET <key> <len>").await;
        return;
    };
    let val = match read_value(reader, len).await {
        Ok(v) => v,
        Err(e) => {
            write_line(writer, &format!("ERR read value: {e}")).await;
            return;
        }
    };
    match engine.set(key.to_string(), val).await {
        Ok(()) => write_line(writer, "OK").await,
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

/// `SETEX <key> <ttl> <len>` followed by `<len>` value bytes.
async fn handle_setex(
    parts: &mut impl Iterator<Item = &str>,
    reader: &mut ClientReader,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) {
    let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
        write_line(writer, "ERR usage: SETEX <key> <ttl> <len>").await;
        return;
    };
    let Some(ttl) = parts.next().and_then(|s| s.parse::<u64>().ok()) else {
        write_line(writer, "ERR bad ttl").await;
        return;
    };
    let Some(len) = parts.next().and_then(|s| s.parse::<usize>().ok()) else {
        write_line(writer, "ERR bad len").await;
        return;
    };
    let val = match read_value(reader, len).await {
        Ok(v) => v,
        Err(e) => {
            write_line(writer, &format!("ERR read value: {e}")).await;
            return;
        }
    };
    match engine.set(key.to_string(), val).await {
        Ok(()) => {
            // Best-effort TTL; ignore expiry failures.
            let _ = engine.expire(key, ttl).await;
            write_line(writer, "OK").await;
        }
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

/// `GET <key>`
async fn handle_get(
    parts: &mut impl Iterator<Item = &str>,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) {
    let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
        write_line(writer, "ERR usage: GET <key>").await;
        return;
    };
    match engine.get(key).await {
        Ok(Some(v)) => {
            let _ = writer
                .write_all(format!("VAL {}\n", v.len()).as_bytes())
                .await;
            let _ = writer.write_all(&v).await;
            let _ = writer.write_all(b"\n").await;
        }
        Ok(None) => write_line(writer, "NIL").await,
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

/// `DEL <key>`
async fn handle_del(
    parts: &mut impl Iterator<Item = &str>,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) {
    let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
        write_line(writer, "ERR usage: DEL <key>").await;
        return;
    };
    match engine.delete(key.to_string()).await {
        Ok(removed) => {
            write_line(writer, if removed { "OK 1" } else { "OK 0" }).await;
        }
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

/// `KEYS [pattern]`
async fn handle_keys(
    parts: &mut impl Iterator<Item = &str>,
    writer: &mut ClientWriter,
    engine: &Arc<Engine>,
) {
    let pattern = parts.next().unwrap_or("*");
    match engine.keys().await {
        Ok(keys) => {
            let matched: Vec<&String> = keys.iter().filter(|k| glob_match(pattern, k)).collect();
            let _ = writer
                .write_all(format!("KEYS {}\n", matched.len()).as_bytes())
                .await;
            for k in matched {
                let _ = writer.write_all(format!("{}\n", k.len()).as_bytes()).await;
                let _ = writer.write_all(k.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
            }
        }
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

/// `STATS` — the engine counters as a JSON object.
async fn handle_stats(writer: &mut ClientWriter, engine: &Arc<Engine>) {
    match engine.stats().await {
        Ok(st) => {
            let json = serde_json::json!({
                "keys": st.maplet_stats.len,
                "capacity": st.maplet_stats.capacity,
                "false_positive_rate": st.maplet_stats.false_positive_rate,
                "memory_usage": st.storage_stats.memory_usage,
                "uptime_seconds": st.uptime_seconds,
                "total_operations": st.total_operations,
                "persistence": format!("{:?}", engine.persistence_mode()),
            });
            write_line(writer, &json.to_string()).await;
        }
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn write_line(w: &mut async_net::unix::UnixStream, s: &str) {
    let _ = w.write_all(s.as_bytes()).await;
    let _ = w.write_all(b"\n").await;
}

/// Read `<len>` value bytes followed by a trailing `\n`.
async fn read_value(reader: &mut ClientReader, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    // Consume the trailing newline (best-effort).
    let mut nl = [0u8; 1];
    let _ = reader.read_exact(&mut nl).await;
    Ok(buf)
}

/// Parse `<key> <len>` from the remaining tokens.
fn next_key_len<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Option<(&'a str, usize)> {
    let key = parts.next().filter(|k| !k.is_empty())?;
    let len = parts.next()?.parse::<usize>().ok()?;
    Some((key, len))
}

fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAPPY_SOCKET_PATH") {
        return PathBuf::from(p);
    }
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    PathBuf::from(runtime).join("mappy").join("mappy.sock")
}

fn persistence_mode() -> PersistenceMode {
    match std::env::var("MAPPY_PERSISTENCE_MODE").as_deref() {
        Ok("disk") => PersistenceMode::Disk,
        Ok("aof") => PersistenceMode::AOF,
        Ok("hybrid") => PersistenceMode::Hybrid,
        _ => PersistenceMode::Memory,
    }
}

fn set_socket_perms(path: &Path) {
    if let Ok(md) = std::fs::metadata(path) {
        let mut perms = md.permissions();
        perms.set_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }
}

/// Removes the socket file on drop.
struct SocketGuard {
    path: PathBuf,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mappy_daemon=info".into()),
        )
        .init();

    let socket_path = socket_path();
    let persistence = persistence_mode();
    let capacity: usize = std::env::var("MAPPY_CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000);
    let fpr: f64 = std::env::var("MAPPY_FALSE_POSITIVE_RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.01);
    let data_dir = std::env::var("MAPPY_DATA_DIR").ok();

    smol::block_on(async {
        // Build the engine (runs natively on smol — no runtime bridging).
        let engine = Arc::new(
            Engine::new(EngineConfig {
                persistence_mode: persistence,
                data_dir,
                maplet: MapletConfig {
                    capacity,
                    false_positive_rate: fpr,
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .context("failed to create mappy engine")?,
        );

        // Periodic flush for non-memory persistence modes.
        if persistence != PersistenceMode::Memory {
            let engine = Arc::clone(&engine);
            smol::spawn(async move {
                loop {
                    smol::Timer::after(Duration::from_secs(10)).await;
                    if let Err(e) = engine.flush().await {
                        tracing::warn!("periodic flush failed: {e}");
                    }
                }
            })
            .detach();
        }

        // Remove stale socket, bind, set perms.
        let _ = std::fs::remove_file(&socket_path);
        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("failed to bind unix socket {}", socket_path.display()))?;
        set_socket_perms(&socket_path);

        tracing::info!("mappy-daemon listening on {}", socket_path.display());

        let _guard = SocketGuard {
            path: socket_path.clone(),
        };

        // Accept loop.
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let engine = Arc::clone(&engine);
                    smol::spawn(handle_conn(stream, engine)).detach();
                }
                Err(e) => tracing::error!("accept error: {e}"),
            }
        }
    })
}
