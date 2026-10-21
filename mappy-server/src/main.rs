//! Mappy Server - Network server for the Mappy service

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        Json,
        sse::{Event, Sse},
    },
    routing::{delete, get as get_route, post},
};
use mappy_core::{Engine, EngineConfig, PersistenceMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::{PermissionsExt, chown};
use std::path::Path as StdPath;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::{RwLock, broadcast};
use tower::Service;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
struct AppState {
    engine: Arc<RwLock<Option<Engine>>>,
    error_log: Arc<RwLock<Vec<MappyError>>>,
    error_broadcast: Arc<broadcast::Sender<MappyError>>,
    start_time: Arc<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type")]
pub enum MappyError {
    CapacityExceeded {
        timestamp: u64,
        key: String,
        severity: ErrorSeverity,
    },
    CollisionLimitExceeded {
        timestamp: u64,
        key: String,
        fingerprint: u64,
        chain_length: usize,
        severity: ErrorSeverity,
    },
    ResizeFailed {
        timestamp: u64,
        reason: String,
        severity: ErrorSeverity,
    },
    SerializationError {
        timestamp: u64,
        key: String,
        reason: String,
        severity: ErrorSeverity,
    },
    Internal {
        timestamp: u64,
        message: String,
        severity: ErrorSeverity,
    },
    OneSidedError {
        timestamp: u64,
        key: String,
        expected_value: String,
        actual_value: String,
        severity: ErrorSeverity,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Set socket permissions with group ownership for shared access
fn set_socket_permissions(socket_path: &StdPath) -> anyhow::Result<()> {
    let mut perms = fs::metadata(socket_path)?.permissions();
    perms.set_mode(0o660); // Group readable/writable for shared services
    fs::set_permissions(socket_path, perms)?;

    // Set group ownership to reynard for shared access
    if let Ok(group_file) = fs::read_to_string("/etc/group") {
        for line in group_file.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3
                && parts[0] == "reynard"
                && let Ok(gid) = parts[2].parse::<u32>()
            {
                // chown can fail if user doesn't have permission, but that's ok
                // The socket will still be accessible to group members
                let _ = chown(socket_path, None, Some(gid));
                break;
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    status: String,
    engine_ready: bool,
}

#[derive(Deserialize)]
struct SetRequest {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
struct GetResponse {
    key: String,
    value: Option<String>,
    found: bool,
}

#[derive(Deserialize)]
struct ErrorsQuery {
    limit: Option<usize>,
    error_type: Option<String>,
    severity: Option<String>,
    since: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct ErrorsResponse {
    errors: Vec<MappyError>,
    total_count: usize,
    filtered_count: usize,
}

#[derive(Serialize, Deserialize)]
struct ErrorStatsResponse {
    total_errors: usize,
    by_type: HashMap<String, usize>,
    by_severity: HashMap<String, usize>,
    error_rate: f64,
    collision_count: usize,
    max_chain_length: usize,
}

#[derive(Serialize, Deserialize)]
struct OneSidedErrorsResponse {
    violations: Vec<MappyError>,
    total_violations: usize,
    prob_exceed_chain: f64,
}

#[derive(Serialize, Deserialize)]
struct MetricsResponse {
    uptime: u64,
    total_operations: u64,
    total_keys: usize,
    memory_usage: usize,
    disk_usage: usize,
    operations_per_second: f64,
    error_rate: f64,
    load_factor: f64,
    response_time_ms: f64,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "Mappy".to_string(),
    })
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let engine_ready = state.engine.read().await.is_some();
    Json(StatusResponse {
        status: "running".to_string(),
        engine_ready,
    })
}

async fn set(
    State(state): State<AppState>,
    Json(request): Json<SetRequest>,
) -> Result<StatusCode, StatusCode> {
    let engine_guard = state.engine.read().await;
    if let Some(engine) = &*engine_guard {
        engine
            .set(request.key.clone(), request.value.as_bytes().to_vec())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_value(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<GetResponse>, StatusCode> {
    let engine_guard = state.engine.read().await;
    if let Some(engine) = &*engine_guard {
        match engine.get(&key).await {
            Ok(Some(value)) => Ok(Json(GetResponse {
                key,
                value: Some(String::from_utf8_lossy(&value).to_string()),
                found: true,
            })),
            Ok(None) => Ok(Json(GetResponse {
                key,
                value: None,
                found: false,
            })),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_errors(
    Query(params): Query<ErrorsQuery>,
    State(state): State<AppState>,
) -> Json<ErrorsResponse> {
    let error_log = state.error_log.read().await;
    let limit = params.limit.unwrap_or(100).min(1000);
    let error_type_filter = params.error_type;
    let severity_filter = params.severity;
    let since = params.since.unwrap_or(0);

    let filtered: Vec<MappyError> = error_log
        .iter()
        .filter(|e| {
            let type_match = error_type_filter.as_deref().is_none_or(|et| match e {
                MappyError::CapacityExceeded { .. } => et == "CapacityExceeded",
                MappyError::CollisionLimitExceeded { .. } => et == "CollisionLimitExceeded",
                MappyError::ResizeFailed { .. } => et == "ResizeFailed",
                MappyError::SerializationError { .. } => et == "SerializationError",
                MappyError::Internal { .. } => et == "Internal",
                MappyError::OneSidedError { .. } => et == "OneSidedError",
            });

            let severity_match = severity_filter.as_deref().is_none_or(|sf| match e {
                MappyError::CapacityExceeded { severity, .. }
                | MappyError::CollisionLimitExceeded { severity, .. }
                | MappyError::ResizeFailed { severity, .. }
                | MappyError::SerializationError { severity, .. }
                | MappyError::Internal { severity, .. }
                | MappyError::OneSidedError { severity, .. } => format!("{severity:?}") == sf,
            });

            let time_match = match e {
                MappyError::CapacityExceeded { timestamp, .. }
                | MappyError::CollisionLimitExceeded { timestamp, .. }
                | MappyError::ResizeFailed { timestamp, .. }
                | MappyError::SerializationError { timestamp, .. }
                | MappyError::Internal { timestamp, .. }
                | MappyError::OneSidedError { timestamp, .. } => *timestamp >= since,
            };

            type_match && severity_match && time_match
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();

    Json(ErrorsResponse {
        total_count: error_log.len(),
        filtered_count: filtered.len(),
        errors: filtered,
    })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "error and key counts are far below 2^52, so the usize -> f64 conversion is exact"
)]
async fn get_error_stats(State(state): State<AppState>) -> Json<ErrorStatsResponse> {
    let error_log = state.error_log.read().await;

    let mut by_type = HashMap::new();
    let mut by_severity = HashMap::new();
    let mut collision_count = 0;
    let mut max_chain_length = 0;

    for error in error_log.iter() {
        match error {
            MappyError::CapacityExceeded { .. } => {
                *by_type.entry("CapacityExceeded".to_string()).or_insert(0) += 1;
            }
            MappyError::CollisionLimitExceeded {
                chain_length,
                severity,
                ..
            } => {
                *by_type
                    .entry("CollisionLimitExceeded".to_string())
                    .or_insert(0) += 1;
                collision_count += 1;
                max_chain_length = max_chain_length.max(*chain_length);
                *by_severity.entry(format!("{severity:?}")).or_insert(0) += 1;
            }
            MappyError::ResizeFailed { severity, .. } => {
                *by_type.entry("ResizeFailed".to_string()).or_insert(0) += 1;
                *by_severity.entry(format!("{severity:?}")).or_insert(0) += 1;
            }
            MappyError::SerializationError { severity, .. } => {
                *by_type.entry("SerializationError".to_string()).or_insert(0) += 1;
                *by_severity.entry(format!("{severity:?}")).or_insert(0) += 1;
            }
            MappyError::Internal { severity, .. } => {
                *by_type.entry("Internal".to_string()).or_insert(0) += 1;
                *by_severity.entry(format!("{severity:?}")).or_insert(0) += 1;
            }
            MappyError::OneSidedError { severity, .. } => {
                *by_type.entry("OneSidedError".to_string()).or_insert(0) += 1;
                *by_severity.entry(format!("{severity:?}")).or_insert(0) += 1;
            }
        }
    }

    let total_errors = error_log.len();
    drop(error_log);
    let error_rate = if total_errors > 0 {
        (by_type.get("CollisionLimitExceeded").unwrap_or(&0)
            + by_type.get("CapacityExceeded").unwrap_or(&0)) as f64
            / total_errors as f64
    } else {
        0.0
    };

    Json(ErrorStatsResponse {
        total_errors,
        by_type,
        by_severity,
        error_rate,
        collision_count,
        max_chain_length,
    })
}

async fn get_one_sided_errors(State(state): State<AppState>) -> Json<OneSidedErrorsResponse> {
    let violations: Vec<MappyError> = state
        .error_log
        .read()
        .await
        .iter()
        .filter(|e| matches!(e, MappyError::OneSidedError { .. }))
        .cloned()
        .collect();

    let total_violations = violations.len();

    let prob_exceed_chain = if total_violations > 0 {
        0.01_f64.powi(i32::try_from(total_violations).unwrap_or(i32::MAX))
    } else {
        0.0
    };

    Json(OneSidedErrorsResponse {
        violations,
        total_violations,
        prob_exceed_chain,
    })
}

async fn clear_errors(State(state): State<AppState>) -> StatusCode {
    let mut error_log = state.error_log.write().await;
    error_log.clear();
    StatusCode::OK
}

#[expect(
    clippy::cast_precision_loss,
    reason = "error and key counts are far below 2^52, so the usize -> f64 conversion is exact"
)]
async fn get_metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    let (total_keys, memory_usage, load_factor) = {
        let engine_guard = state.engine.read().await;
        if let Some(engine) = &*engine_guard {
            let engine_stats = engine.stats().await.unwrap();
            (
                engine_stats.maplet_stats.len,
                engine_stats.maplet_stats.memory_usage,
                engine_stats.maplet_stats.load_factor,
            )
        } else {
            (0, 0, 0.0)
        }
    };

    let total_errors = state.error_log.read().await.len();
    let error_rate = if total_keys > 0 {
        total_errors as f64 / total_keys as f64
    } else {
        0.0
    };

    let uptime = state.start_time.elapsed().as_secs();

    Json(MetricsResponse {
        uptime,
        total_operations: total_keys as u64,
        total_keys,
        memory_usage,
        disk_usage: memory_usage,
        operations_per_second: 100.0,
        error_rate,
        load_factor,
        response_time_ms: 2.5,
    })
}

async fn errors_stream(
    State(state): State<AppState>,
) -> Sse<impl futures::Stream<Item = Result<Event, axum::Error>>> {
    let mut rx = state.error_broadcast.subscribe();

    let stream = async_stream::try_stream! {
        while let Ok(error) = rx.recv().await {
            let json = serde_json::to_string(&error).unwrap();
            yield Event::default().data(json);
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keepalive"),
    )
}

// Adapter to make UnixStream work with hyper/axum
struct UnixStreamAdapter(tokio::net::UnixStream);

impl AsyncRead for UnixStreamAdapter {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStreamAdapter {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}

// Use hyper_util for Unix socket support

async fn serve_unix_socket(listener: UnixListener, app: Router) -> anyhow::Result<()> {
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use hyper_util::server::conn::auto::Builder;

    loop {
        let (stream, _) = listener.accept().await?;
        let app = app.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(UnixStreamAdapter(stream));
            let svc = service_fn(move |req| {
                let mut app = app.clone();
                async move {
                    app.call(req)
                        .await
                        .map_err(|e| std::io::Error::other(format!("{e}")))
                }
            });

            let builder = Builder::new(hyper_util::rt::TokioExecutor::new());

            if let Err(e) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving Unix socket connection: {}", e);
            }
        });
    }
}

// Use hyper_util for HTTP socket support
async fn serve_http_socket(listener: TcpListener, app: Router) -> anyhow::Result<()> {
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use hyper_util::server::conn::auto::Builder;

    loop {
        let (stream, _) = listener.accept().await?;
        let app = app.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let mut app = app.clone();
                async move {
                    app.call(req)
                        .await
                        .map_err(|e| std::io::Error::other(format!("{e}")))
                }
            });

            let builder = Builder::new(hyper_util::rt::TokioExecutor::new());

            if let Err(e) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving HTTP connection: {}", e);
            }
        });
    }
}

/// Runtime configuration read from the environment.
#[cfg(feature = "tokio-runtime")]
struct ServerConfig {
    socket_path: String,
    enable_http: bool,
    enable_socket: bool,
    port: u16,
    host: String,
    capacity: usize,
    false_positive_rate: f64,
    data_dir: String,
    persistence_mode: String,
}

#[cfg(feature = "tokio-runtime")]
impl ServerConfig {
    /// Read every knob from the environment, applying the defaults the server
    /// has always used when a variable is unset.
    fn from_env() -> anyhow::Result<Self> {
        fn env(name: &str, default: &str) -> String {
            std::env::var(name).unwrap_or_else(|_| default.to_string())
        }

        let enable_http = env("MAPPY_ENABLE_HTTP", "false").parse().unwrap_or(false);
        let enable_socket = env("MAPPY_ENABLE_SOCKET", "true").parse().unwrap_or(true);

        Ok(Self {
            socket_path: env("MAPPY_SOCKET_PATH", "/var/run/reynard/mappy.sock"),
            enable_http,
            enable_socket,
            port: env("MAPPY_PORT", "8003").parse()?,
            host: env("MAPPY_HOST", "0.0.0.0"),
            capacity: env("MAPPY_CAPACITY", "10000").parse()?,
            false_positive_rate: env("MAPPY_FALSE_POSITIVE_RATE", "0.01").parse()?,
            data_dir: env("MAPPY_DATA_DIR", "./data/mappy"),
            persistence_mode: env("MAPPY_PERSISTENCE_MODE", "memory"),
        })
    }

    fn persistence(&self) -> PersistenceMode {
        match self.persistence_mode.as_str() {
            "disk" => PersistenceMode::Disk,
            "hybrid" => PersistenceMode::Hybrid,
            _ => PersistenceMode::Memory,
        }
    }

    fn engine_config(&self) -> EngineConfig {
        EngineConfig {
            persistence_mode: self.persistence(),
            data_dir: Some(self.data_dir.clone()),
            maplet: mappy_core::types::MapletConfig {
                capacity: self.capacity,
                false_positive_rate: self.false_positive_rate,
                max_load_factor: 0.95,
                auto_resize: true,
                enable_deletion: true,
                enable_merging: true,
            },
            storage: mappy_core::storage::StorageConfig::default(),
            ttl: mappy_core::ttl::TTLConfig::default(),
        }
    }
}

#[cfg(feature = "tokio-runtime")]
fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get_route(health))
        .route("/status", get_route(status))
        .route("/set", post(set))
        .route("/get/{key}", get_route(get_value))
        .route("/api/errors", get_route(get_errors))
        .route("/api/errors", delete(clear_errors))
        .route("/api/errors/stats", get_route(get_error_stats))
        .route("/api/errors/one-sided", get_route(get_one_sided_errors))
        .route("/api/metrics", get_route(get_metrics))
        .route("/api/errors/stream", get_route(errors_stream))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

/// Bind the Unix socket, removing a stale socket file and creating the parent
/// directory if the deployment has not already done so.
#[cfg(feature = "tokio-runtime")]
fn prepare_socket(path: &str) -> anyhow::Result<UnixListener> {
    let socket_path = StdPath::new(path);
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }
    if let Some(parent) = socket_path.parent() {
        let _ = fs::create_dir_all(parent); // Non-fatal - deployment script should create it
        // Directory permissions are set by deployment scripts, not at runtime
    }
    let listener = UnixListener::bind(socket_path)?;
    set_socket_permissions(socket_path)?;
    Ok(listener)
}

/// Bind whatever the configuration enabled and drive it to completion.
#[cfg(feature = "tokio-runtime")]
async fn serve(cfg: ServerConfig, app: Router) -> anyhow::Result<()> {
    if cfg.enable_http && cfg.enable_socket {
        let unix_listener = prepare_socket(&cfg.socket_path)?;
        info!("Mappy Server Unix socket listening on {}", cfg.socket_path);

        let http_listener = TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)).await?;
        info!("Mappy Server HTTP listening on {}:{}", cfg.host, cfg.port);

        let app_clone = app.clone();
        let socket_handle =
            tokio::spawn(async move { serve_unix_socket(unix_listener, app_clone).await });
        let http_handle = tokio::spawn(async move { serve_http_socket(http_listener, app).await });

        tokio::select! {
            result = socket_handle => {
                if let Err(e) = result? {
                    return Err(anyhow::anyhow!("Unix socket server error: {e}"));
                }
            }
            result = http_handle => {
                if let Err(e) = result? {
                    return Err(anyhow::anyhow!("HTTP server error: {e}"));
                }
            }
        }
    } else if cfg.enable_http {
        let http_listener = TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)).await?;
        info!("Mappy Server HTTP listening on {}:{}", cfg.host, cfg.port);
        serve_http_socket(http_listener, app).await?;
    } else if cfg.enable_socket {
        let unix_listener = prepare_socket(&cfg.socket_path)?;
        info!("Mappy Server Unix socket listening on {}", cfg.socket_path);
        serve_unix_socket(unix_listener, app).await?;
    } else {
        return Err(anyhow::anyhow!(
            "At least one of HTTP or socket must be enabled"
        ));
    }

    Ok(())
}

#[cfg(feature = "tokio-runtime")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Mappy Server");

    dotenvy::EnvLoader::new().load().ok();

    let cfg = ServerConfig::from_env()?;
    info!(
        "Configuration: socket_path={}, enable_http={}, port={}, host={}, capacity={}, false_positive_rate={}, data_dir={}, persistence={}",
        cfg.socket_path,
        cfg.enable_http,
        cfg.port,
        cfg.host,
        cfg.capacity,
        cfg.false_positive_rate,
        cfg.data_dir,
        cfg.persistence_mode
    );

    let engine = Engine::new(cfg.engine_config()).await?;
    let (error_broadcast, _) = broadcast::channel(1000);
    let state = AppState {
        engine: Arc::new(RwLock::new(Some(engine))),
        error_log: Arc::new(RwLock::new(Vec::new())),
        error_broadcast: Arc::new(error_broadcast),
        start_time: Arc::new(Instant::now()),
    };

    serve(cfg, build_router(state)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let config = EngineConfig {
            persistence_mode: PersistenceMode::Memory,
            data_dir: None,
            maplet: mappy_core::types::MapletConfig {
                capacity: 1000,
                false_positive_rate: 0.01,
                max_load_factor: 0.95,
                auto_resize: true,
                enable_deletion: true,
                enable_merging: true,
            },
            storage: mappy_core::storage::StorageConfig::default(),
            ttl: mappy_core::ttl::TTLConfig::default(),
        };

        let engine = Engine::new(config).await.unwrap();
        let (error_broadcast, _) = broadcast::channel(1000);
        let start_time = Arc::new(Instant::now());
        let state = AppState {
            engine: Arc::new(RwLock::new(Some(engine))),
            error_log: Arc::new(RwLock::new(Vec::new())),
            error_broadcast: Arc::new(error_broadcast),
            start_time,
        };

        Router::new()
            .route("/health", get_route(health))
            .route("/status", get_route(status))
            .route("/set", post(set))
            .route("/get/{key}", get_route(get_value))
            .route("/api/errors", get_route(get_errors))
            .route("/api/errors/stats", get_route(get_error_stats))
            .route("/api/errors/one-sided", get_route(get_one_sided_errors))
            .route("/api/errors", delete(clear_errors))
            .route("/api/metrics", get_route(get_metrics))
            .route("/api/errors/stream", get_route(errors_stream))
            .with_state(state)
    }

    fn create_test_app_no_engine() -> Router {
        let (error_broadcast, _) = broadcast::channel(1000);
        let state = AppState {
            engine: Arc::new(RwLock::new(None)),
            error_log: Arc::new(RwLock::new(Vec::new())),
            error_broadcast: Arc::new(error_broadcast),
            start_time: Arc::new(Instant::now()),
        };

        Router::new()
            .route("/health", get_route(health))
            .route("/status", get_route(status))
            .route("/set", post(set))
            .route("/get/{key}", get_route(get_value))
            .route("/api/errors", get_route(get_errors))
            .route("/api/errors/stats", get_route(get_error_stats))
            .route("/api/errors/one-sided", get_route(get_one_sided_errors))
            .route("/api/errors", delete(clear_errors))
            .route("/api/metrics", get_route(get_metrics))
            .route("/api/errors/stream", get_route(errors_stream))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.service, "Mappy");
    }

    #[tokio::test]
    async fn test_status_endpoint_with_engine() {
        let app = create_test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(status.status, "running");
        assert!(status.engine_ready);
    }

    #[tokio::test]
    async fn test_status_endpoint_without_engine() {
        let app = create_test_app_no_engine();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(status.status, "running");
        assert!(!status.engine_ready);
    }

    #[tokio::test]
    async fn test_set_endpoint_success() {
        let app = create_test_app().await;
        let request_body = serde_json::json!({
            "key": "test_key",
            "value": "test_value"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_set_endpoint_no_engine() {
        let app = create_test_app_no_engine();
        let request_body = serde_json::json!({
            "key": "test_key",
            "value": "test_value"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_endpoint_found() {
        let app = create_test_app().await;

        // First set a value
        let set_body = serde_json::json!({
            "key": "test_key",
            "value": "test_value"
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&set_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then get it
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/test_key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_response: GetResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(get_response.key, "test_key");
        assert!(get_response.found);
        assert_eq!(get_response.value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_get_endpoint_not_found() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/nonexistent_key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_response: GetResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(get_response.key, "nonexistent_key");
        assert!(!get_response.found);
        assert_eq!(get_response.value, None);
    }

    #[tokio::test]
    async fn test_get_endpoint_no_engine() {
        let app = create_test_app_no_engine();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/test_key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_set_get_roundtrip() {
        let app = create_test_app().await;

        // Set multiple values
        let keys_values = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

        for (key, value) in &keys_values {
            let request_body = serde_json::json!({
                "key": key,
                "value": value
            });

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/set")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }

        // Get all values
        for (key, expected_value) in &keys_values {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/get/{key}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let get_response: GetResponse = serde_json::from_slice(&body).unwrap();
            assert_eq!(get_response.key, *key);
            assert!(get_response.found);
            assert_eq!(get_response.value, Some(expected_value.to_string()));
        }
    }

    #[tokio::test]
    async fn test_set_with_special_characters() {
        let app = create_test_app().await;

        // Use a key without slashes (slashes break URL paths)
        let request_body = serde_json::json!({
            "key": "special_key@123",
            "value": "value with spaces and\nnewlines"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Get it back
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/special_key@123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_response: GetResponse = serde_json::from_slice(&body).unwrap();
        assert!(get_response.found);
        assert_eq!(
            get_response.value,
            Some("value with spaces and\nnewlines".to_string())
        );
    }

    #[tokio::test]
    async fn test_set_empty_value() {
        let app = create_test_app().await;

        let request_body = serde_json::json!({
            "key": "empty_key",
            "value": ""
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Get it back
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/empty_key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_response: GetResponse = serde_json::from_slice(&body).unwrap();
        assert!(get_response.found);
        assert_eq!(get_response.value, Some(String::new()));
    }

    #[test]
    fn test_set_socket_permissions_nonexistent_file() {
        let temp_path = StdPath::new("/tmp/nonexistent_test_socket_12345.sock");
        let result = set_socket_permissions(temp_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "Mappy".to_string(),
        };
        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(serialized.contains("healthy"));
        assert!(serialized.contains("Mappy"));
    }

    #[test]
    fn test_status_response_serialization() {
        let response = StatusResponse {
            status: "running".to_string(),
            engine_ready: true,
        };
        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(serialized.contains("running"));
    }

    #[test]
    fn test_set_request_deserialization() {
        let json = r#"{"key": "test_key", "value": "test_value"}"#;
        let request: SetRequest = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(request.key, "test_key");
        assert_eq!(request.value, "test_value");
    }

    #[test]
    fn test_get_response_serialization() {
        let response = GetResponse {
            key: "test_key".to_string(),
            value: Some("test_value".to_string()),
            found: true,
        };
        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(serialized.contains("test_key"));
        assert!(serialized.contains("test_value"));
    }
}
