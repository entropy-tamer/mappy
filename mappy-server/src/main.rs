//! Mappy Server - Network server for the Mappy service

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get as get_route, post},
};
use mappy_core::{Engine, EngineConfig, PersistenceMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::RwLock;
use tower::Service;
use tracing::info;

#[derive(Clone)]
struct AppState {
    engine: Arc<RwLock<Option<Engine>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    engine_ready: bool,
}

#[derive(Deserialize)]
struct SetRequest {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct GetResponse {
    key: String,
    value: Option<String>,
    found: bool,
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
    if let Some(ref engine) = *engine_guard {
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
    if let Some(ref engine) = *engine_guard {
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
                    app.call(req).await.map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))
                    })
                }
            });

            let builder = Builder::new(hyper_util::rt::TokioExecutor::new());

            if let Err(e) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving Unix socket connection: {}", e);
            }
        });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Mappy Server");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let socket_path = std::env::var("MAPPY_SOCKET_PATH")
        .unwrap_or_else(|_| "/var/run/reynard/mappy.sock".to_string());
    let enable_http = std::env::var("MAPPY_ENABLE_HTTP")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    let enable_socket = std::env::var("MAPPY_ENABLE_SOCKET")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let port = std::env::var("MAPPY_PORT")
        .unwrap_or_else(|_| "8003".to_string())
        .parse::<u16>()?;
    let host = std::env::var("MAPPY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let capacity = std::env::var("MAPPY_CAPACITY")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<usize>()?;
    let false_positive_rate = std::env::var("MAPPY_FALSE_POSITIVE_RATE")
        .unwrap_or_else(|_| "0.01".to_string())
        .parse::<f64>()?;
    let data_dir = std::env::var("MAPPY_DATA_DIR").unwrap_or_else(|_| "./data/mappy".to_string());
    let persistence_mode =
        std::env::var("MAPPY_PERSISTENCE_MODE").unwrap_or_else(|_| "memory".to_string());

    info!(
        "Configuration: socket_path={}, enable_http={}, port={}, host={}, capacity={}, false_positive_rate={}, data_dir={}, persistence={}",
        socket_path,
        enable_http,
        port,
        host,
        capacity,
        false_positive_rate,
        data_dir,
        persistence_mode
    );

    // Initialize engine
    let persistence = match persistence_mode.as_str() {
        "disk" => PersistenceMode::Disk,
        "hybrid" => PersistenceMode::Hybrid,
        _ => PersistenceMode::Memory,
    };

    let config = EngineConfig {
        persistence_mode: persistence,
        data_dir: Some(data_dir),
        maplet: mappy_core::types::MapletConfig {
            capacity,
            false_positive_rate,
            max_load_factor: 0.95,
            auto_resize: true,
            enable_deletion: true,
            enable_merging: true,
        },
        storage: mappy_core::storage::StorageConfig::default(),
        ttl: mappy_core::ttl::TTLConfig::default(),
    };

    let engine = Engine::new(config).await?;
    let state = AppState {
        engine: Arc::new(RwLock::new(Some(engine))),
    };

    // Build router
    let app = Router::new()
        .route("/health", get_route(health))
        .route("/status", get_route(status))
        .route("/set", post(set))
        .route("/get/{key}", get_route(get_value))
        .with_state(state);

    // Start servers based on configuration
    if enable_http && enable_socket {
        // Both HTTP and socket
        let socket_path_std = StdPath::new(&socket_path);
        if socket_path_std.exists() {
            fs::remove_file(socket_path_std)?;
        }

        if let Some(parent) = socket_path_std.parent() {
            fs::create_dir_all(parent)?;
            let mut dir_perms = fs::metadata(parent)?.permissions();
            dir_perms.set_mode(0o775);
            fs::set_permissions(parent, dir_perms)?;
        }

        let unix_listener = UnixListener::bind(socket_path_std)?;
        let mut perms = fs::metadata(socket_path_std)?.permissions();
        perms.set_mode(0o664);
        fs::set_permissions(socket_path_std, perms)?;
        info!("Mappy Server Unix socket listening on {}", socket_path);

        let http_listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!("Mappy Server HTTP listening on {}:{}", host, port);

        let app_clone = app.clone();
        let socket_handle =
            tokio::spawn(async move { serve_unix_socket(unix_listener, app_clone).await });

        let http_handle = tokio::spawn(async move { axum::serve(http_listener, app).await });

        tokio::select! {
            result = socket_handle => {
                if let Err(e) = result? {
                    return Err(anyhow::anyhow!("Unix socket server error: {}", e));
                }
            }
            result = http_handle => {
                if let Err(e) = result? {
                    return Err(anyhow::anyhow!("HTTP server error: {}", e));
                }
            }
        }
    } else if enable_http {
        // HTTP only
        let http_listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!("Mappy Server HTTP listening on {}:{}", host, port);
        axum::serve(http_listener, app).await?;
    } else if enable_socket {
        // Socket only
        let socket_path_std = StdPath::new(&socket_path);
        if socket_path_std.exists() {
            fs::remove_file(socket_path_std)?;
        }

        if let Some(parent) = socket_path_std.parent() {
            fs::create_dir_all(parent)?;
            let mut dir_perms = fs::metadata(parent)?.permissions();
            dir_perms.set_mode(0o775);
            fs::set_permissions(parent, dir_perms)?;
        }

        let unix_listener = UnixListener::bind(socket_path_std)?;
        let mut perms = fs::metadata(socket_path_std)?.permissions();
        perms.set_mode(0o664);
        fs::set_permissions(socket_path_std, perms)?;
        info!("Mappy Server Unix socket listening on {}", socket_path);
        serve_unix_socket(unix_listener, app).await?;
    } else {
        return Err(anyhow::anyhow!(
            "At least one of HTTP or socket must be enabled"
        ));
    }

    Ok(())
}
