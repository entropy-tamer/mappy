use mappy_proxy::{ProxyState, create_app};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let socket_path = std::env::var("MAPPY_SOCKET_PATH")
        .unwrap_or_else(|_| "/var/run/reynard/mappy.sock".to_string());
    let proxy_port_env = std::env::var("MAPPY_PROXY_PORT")
        .unwrap_or_else(|_| "8004".to_string());
    let proxy_port = proxy_port_env.parse::<u16>()
        .map_err(|e| anyhow::anyhow!("Invalid MAPPY_PROXY_PORT: {}", e))?;

    tracing::info!("Starting Mappy HTTP Proxy on port {}", proxy_port);
    tracing::info!("Forwarding to Unix socket: {}", socket_path);

    let state = ProxyState {
        socket_path: Arc::new(socket_path),
    };

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", proxy_port))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port {}: {}", proxy_port, e))?;
    tracing::info!("HTTP proxy listening on 0.0.0.0:{}", proxy_port);
    tracing::info!("Proxy is ready to accept connections");

    axum::serve(listener, app).await;
    Ok(())
}
