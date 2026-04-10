use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use mappy_client::Client;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Clone)]
pub struct ProxyState {
    pub socket_path: Arc<String>,
}

pub async fn proxy_handler(
    State(state): State<ProxyState>,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    info!("Received request: {} {}", method, uri);
    info!("Socket path: {}", state.socket_path);

    match forward_to_mappy(state.socket_path.as_ref(), req).await {
        Ok(response) => {
            info!("Request successful: {} {}", method, path);
            response
        },
        Err(e) => {
            error!("Proxy error for {} {}: {}", method, path, e);
            error!("Error details: {:?}", e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn forward_to_mappy(
    socket_path: &str,
    req: Request,
) -> Result<Response, anyhow::Error> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let headers = req.headers().clone();

    info!("Creating mappy-client with socket: {}", socket_path);

    let client = Client::with_config(
        mappy_client::ClientConfig::with_socket(socket_path)
    );

    info!("Reading request body...");

    let body_bytes = http_body_util::BodyExt::collect(req.into_body())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read request body: {}", e))?
        .to_bytes();
    let body = if body_bytes.is_empty() { None } else { Some(body_bytes.as_ref()) };

    info!("Sending request to mappy: {} {} (body: {} bytes)", method, path, body_bytes.len());

    let response_bytes = client.request(method.as_str(), &path, body).await
        .map_err(|e| {
            error!("Mappy client request failed: {}", e);
            anyhow::anyhow!("Mappy client request failed: {}", e)
        })?;

    info!("Received response from mappy: {} bytes", response_bytes.len());

    let response_str = String::from_utf8_lossy(&response_bytes);
    
    let mut response_builder = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json");

    for (name, value) in headers.iter() {
        if name.as_str() == "accept" {
            response_builder = response_builder.header(name, value);
        }
    }

    response_builder.body(Body::from(response_str.to_string()))
        .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))
}

pub fn create_app(state: ProxyState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", any(proxy_handler))
        .route("/{*path}", any(proxy_handler))
        .layer(cors)
        .with_state(state)
}
