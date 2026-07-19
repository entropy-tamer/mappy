//! mappy-proxy: a smol TCP→Unix byte pipe for mappy-daemon.
//!
//! Listens on a TCP port and bidirectionally forwards bytes to the mappy
//! Unix socket. Allows remote machines on the mesh to access the local
//! mappy-daemon over TCP without any protocol translation.
//!
//! ## Usage
//!
//! ```sh
//! MAPPY_PROXY_LISTEN=0.0.0.0:6380 MAPPY_SOCKET_PATH=/run/user/1000/mappy/mappy.sock mappy-proxy
//! ```
//!
//! Remote clients connect to `host:6380` and speak the mappy line protocol
//! directly (PING, SET, GET, DEL, KEYS, STATS).

use anyhow::Context as _;
use async_net::unix::UnixStream;
use async_net::TcpListener;
use futures_lite::io::copy;
use std::net::SocketAddr;

fn listen_addr() -> SocketAddr {
    std::env::var("MAPPY_PROXY_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:6380".to_string())
        .parse()
        .expect("MAPPY_PROXY_LISTEN must be a valid socket address (e.g. 0.0.0.0:6380)")
}

fn unix_socket_path() -> String {
    std::env::var("MAPPY_SOCKET_PATH").unwrap_or_else(|_| {
        let runtime =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        format!("{runtime}/mappy/mappy.sock")
    })
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mappy_proxy=info".into()),
        )
        .init();

    let addr = listen_addr();
    let socket_path = unix_socket_path();

    smol::block_on(async {
                let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener on {addr}"))?;

        tracing::info!("mappy-proxy listening on tcp://{addr} → unix://{socket_path}");

        loop {
            let (tcp_stream, peer) = listener.accept().await?;
            let socket_path = socket_path.clone();

            smol::spawn(async move {
                tracing::debug!("connection from {peer}");

                let unix_stream = match UnixStream::connect(&socket_path).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("failed to connect to mappy socket: {e}");
                        return;
                    }
                };

                // Bidirectional copy: TCP↔Unix.
                let (mut tcp_read, mut tcp_write) = (tcp_stream.clone(), tcp_stream);
                let (mut unix_read, mut unix_write) = (unix_stream.clone(), unix_stream);

                let client_to_server = async {
                    let _ = copy(&mut tcp_read, &mut unix_write).await;
                };
                let server_to_client = async {
                    let _ = copy(&mut unix_read, &mut tcp_write).await;
                };

                // Run both directions concurrently; when either finishes, drop the other.
                futures_lite::future::zip(client_to_server, server_to_client).await;

                tracing::debug!("connection from {peer} closed");
            })
            .detach();
        }
    })
}
