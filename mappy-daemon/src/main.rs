//! mappy-daemon: a silly, fast, semi-unreliable KV store.
//!
//! A smol-based Unix-socket server (socket-only, no TCP) backed by mappy-core's
//! `Engine`. Because mappy-core's `Engine` is Tokio-bound, each KV operation is
//! dispatched onto a small embedded Tokio runtime; the socket accept loop and
//! per-connection tasks run entirely on smol.
//!
//! ## Line protocol (UTF-8 commands, values are length-framed raw bytes)
//!
//! ```text
//! PING                         -> PONG
//! SET <key> <len>\n<bytes>\n   -> OK          (len = decimal byte count)
//! SETEX <key> <ttl> <len>\n<bytes>\n -> OK   (ttl seconds)
//! GET <key>                    -> VAL <len>\n<bytes>\n | NIL
//! DEL <key>                    -> OK <0|1>
//! KEYS <pattern>               -> KEYS <n>\n  then n lines: <len>\n<bytes>\n
//! STATS                        -> STATS <json>\n
//! QUIT                         -> (closes connection)
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

struct State {
    engine: Arc<Engine>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mappy_daemon=info".into()),
        )
        .init();

    let socket_path = socket_path();
    let persistence = persistence_mode();
    let capacity = env_parse("MAPPY_CAPACITY", 100_000);
    let false_positive_rate = env_parse("MAPPY_FALSE_POSITIVE_RATE", 0.01_f64);
    let data_dir = std::env::var("MAPPY_DATA_DIR").ok().filter(|s| !s.is_empty());

    let tokio_rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .context("failed to build tokio runtime for mappy engine")?,
    );

    let engine = Arc::new(
        tokio_rt
            .block_on(Engine::new(EngineConfig {
                persistence_mode: persistence,
                data_dir,
                maplet: MapletConfig {
                    capacity,
                    false_positive_rate,
                    ..Default::default()
                },
                ..Default::default()
            }))
            .context("failed to create mappy engine")?,
    );

    // Periodic flush for durable modes so a crash loses at most ~10s of writes.
    if persistence != PersistenceMode::Memory {
        let engine = engine.clone();
        let rt = tokio_rt.clone();
        smol::spawn(async move {
            loop {
                smol::Timer::after(Duration::from_secs(10)).await;
                                let _ = rt.spawn({
                    let engine = engine.clone();
                    async move { engine.flush().await }
                });
            }
        })
        .detach();
    }

    let state = Arc::new(State { engine, tokio_rt });

    // Bind the Unix socket (remove any stale socket file first).
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    if let Some(parent) = socket_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("failed to bind unix socket {}", socket_path.display()))?;
    set_socket_perms(&socket_path);
    tracing::info!("mappy-daemon listening on {}", socket_path.display());

    // Remove the socket file when the process exits.
    let _guard = SocketGuard::new(socket_path.clone());

    smol::block_on(async {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    smol::spawn(handle_conn(stream, state)).detach();
                }
                Err(e) => tracing::error!("accept error: {e}"),
            }
        }
    })
}

async fn handle_conn(stream: async_net::unix::UnixStream, state: Arc<State>) {
    // Clone the stream so we can buffer reads while writing responses on the
    // other clone. Both share the same fd; the connection is request/response,
    // so there is no read/write concurrency on a single connection.
    let mut reader = BufReader::new(stream.clone());
    let mut writer = stream;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF: client disconnected
            Ok(_) => {}
            Err(e) => {
                tracing::debug!("connection read error: {e}");
                break;
            }
        }
        let cmd = line.trim_end_matches(['\r', '\n']);
        if cmd.is_empty() {
            continue;
        }
        if !handle_command(cmd, &mut reader, &mut writer, state.clone()).await {
            break;
        }
    }
}

/// Returns `false` to close the connection (QUIT / fatal).
async fn handle_command(
    cmd: &str,
    reader: &mut BufReader<async_net::unix::UnixStream>,
    writer: &mut async_net::unix::UnixStream,
    state: Arc<State>,
) -> bool {
        let mut parts = cmd.split(' ');
    let verb = parts.next().unwrap_or("");

    match verb {
        "PING" => write_line(writer, "PONG").await,
        "SET" => {
            let Some((key, len)) = next_key_len(&mut parts) else {
                write_line(writer, "ERR usage: SET <key> <len>").await;
                return true;
            };
            let val = match read_value(reader, len).await {
                Ok(v) => v,
                Err(e) => {
                    write_line(writer, &format!("ERR read value: {e}")).await;
                    return true;
                }
            };
            let engine = state.engine.clone();
            let key = key.to_string();
            let res = state
                .tokio_rt
                .spawn(async move { engine.set(key, val).await })
                .await;
            write_engine_result(writer, res).await;
        }
        "SETEX" => {
            let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
                write_line(writer, "ERR usage: SETEX <key> <ttl> <len>").await;
                return true;
            };
            let Some(ttl) = parts.next().and_then(|s| s.parse::<u64>().ok()) else {
                write_line(writer, "ERR bad ttl").await;
                return true;
            };
            let Some(len) = parts.next().and_then(|s| s.parse::<usize>().ok()) else {
                write_line(writer, "ERR bad len").await;
                return true;
            };
            let val = match read_value(reader, len).await {
                Ok(v) => v,
                Err(e) => {
                    write_line(writer, &format!("ERR read value: {e}")).await;
                    return true;
                }
            };
            let engine = state.engine.clone();
            let key = key.to_string();
            let set_res = state
                .tokio_rt
                .spawn({
                    let key = key.clone();
                    async move { engine.set(key, val).await }
                })
                .await;
            match set_res {
                Ok(Ok(())) => {
                    // Best-effort TTL; ignore expiry failures.
                    let engine2 = state.engine.clone();
                    let _ = state
                        .tokio_rt
                        .spawn({
                            let key = key.clone();
                            async move { engine2.expire(&key, ttl).await }
                        })
                        .await;
                    write_line(writer, "OK").await;
                }
                Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
                Err(e) => write_line(writer, &format!("ERR {e}")).await,
            }
        }
        "GET" => {
            let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
                write_line(writer, "ERR usage: GET <key>").await;
                return true;
            };
            let engine = state.engine.clone();
            let key = key.to_string();
            let res = state
                .tokio_rt
                .spawn(async move { engine.get(&key).await })
                .await;
            match res {
                Ok(Ok(Some(v))) => {
                    let _ = writer.write_all(format!("VAL {}\n", v.len()).as_bytes()).await;
                    let _ = writer.write_all(&v).await;
                    let _ = writer.write_all(b"\n").await;
                }
                Ok(Ok(None)) => write_line(writer, "NIL").await,
                Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
                Err(e) => write_line(writer, &format!("ERR {e}")).await,
            }
        }
        "DEL" => {
            let Some(key) = parts.next().filter(|k| !k.is_empty()) else {
                write_line(writer, "ERR usage: DEL <key>").await;
                return true;
            };
            let engine = state.engine.clone();
            let key = key.to_string();
            let res = state
                .tokio_rt
                .spawn(async move { engine.delete(key).await })
                .await;
                        match res {
                Ok(Ok(removed)) => {
                    write_line(writer, if removed { "OK 1" } else { "OK 0" }).await
                }
                Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
                Err(e) => write_line(writer, &format!("ERR {e}")).await,
            }
        }
        "KEYS" => {
            let pattern = parts.next().unwrap_or("*");
            let engine = state.engine.clone();
            let res = state
                .tokio_rt
                .spawn(async move { engine.keys().await })
                .await;
            match res {
                Ok(Ok(keys)) => {
                    let matched: Vec<&String> =
                        keys.iter().filter(|k| glob_match(pattern, k)).collect();
                    let _ = writer
                        .write_all(format!("KEYS {}\n", matched.len()).as_bytes())
                        .await;
                    for k in matched {
                        let _ = writer.write_all(format!("{}\n", k.len()).as_bytes()).await;
                        let _ = writer.write_all(k.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }
                }
                Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
                Err(e) => write_line(writer, &format!("ERR {e}")).await,
            }
        }
        "STATS" => {
            let engine = state.engine.clone();
            let res = state
                .tokio_rt
                .spawn(async move { engine.stats().await })
                .await;
            match res {
                Ok(Ok(st)) => {
                    let json = serde_json::json!({
                        "keys": st.maplet_stats.len,
                        "capacity": st.maplet_stats.capacity,
                        "false_positive_rate": st.maplet_stats.false_positive_rate,
                        "memory_usage": st.storage_stats.memory_usage,
                        "uptime_seconds": st.uptime_seconds,
                        "total_operations": st.total_operations,
                        "persistence": format!("{:?}", state.engine.persistence_mode()),
                    });
                    write_line(writer, &json.to_string()).await;
                }
                Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
                Err(e) => write_line(writer, &format!("ERR {e}")).await,
            }
        }
        "QUIT" => return false,
        _ => write_line(writer, "ERR unknown command").await,
    }
    true
}

async fn write_engine_result(
    writer: &mut async_net::unix::UnixStream,
    res: Result<std::result::Result<(), mappy_core::MapletError>, tokio::task::JoinError>,
) {
    match res {
        Ok(Ok(())) => write_line(writer, "OK").await,
        Ok(Err(e)) => write_line(writer, &format!("ERR {e}")).await,
        Err(e) => write_line(writer, &format!("ERR {e}")).await,
    }
}

fn next_key_len<'a, I>(parts: &mut I) -> Option<(&'a str, usize)>
where
    I: Iterator<Item = &'a str>,
{
    let key = parts.next().filter(|k| !k.is_empty())?;
    let len = parts.next().and_then(|s| s.parse().ok())?;
    Some((key, len))
}

async fn read_value(
    reader: &mut BufReader<async_net::unix::UnixStream>,
    len: usize,
) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    // Consume the trailing newline that terminates the value frame.
    let mut nl = [0u8; 1];
    let _ = reader.read_exact(&mut nl).await;
    Ok(buf)
}

async fn write_line(writer: &mut async_net::unix::UnixStream, s: &str) {
    let _ = writer.write_all(s.as_bytes()).await;
    let _ = writer.write_all(b"\n").await;
}

fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAPPY_SOCKET_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
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

fn env_parse<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn set_socket_perms(path: &Path) {
    if let Ok(mut perms) = std::fs::metadata(path).map(|m| m.permissions()) {
        perms.set_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }
}

struct SocketGuard {
    path: PathBuf,
}

impl SocketGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Minimal `*`/`?` glob matcher.
fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0_usize, 0_usize);
    let (mut star, mut mark) = (None, 0_usize);
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
