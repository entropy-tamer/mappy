use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::task;

async fn start_mock_unix_server(socket_path: String) -> task::JoinHandle<()> {
    task::spawn(async move {
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind Unix socket");
        
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let mut buffer = [0u8; 1024];
                    match stream.read(&mut buffer).await {
                        Ok(n) => {
                            let request = String::from_utf8_lossy(&buffer[..n]);
                            
                            let response = if request.contains("GET") {
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}"
                            } else if request.contains("POST") {
                                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\n\r\n{\"status\":\"created\"}"
                            } else {
                                "HTTP/1.1 404 Not Found\r\n\r\n"
                            };
                            
                            if let Err(e) = stream.write_all(response.as_bytes()).await {
                                eprintln!("Failed to write response: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Failed to read: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to accept: {}", e),
            }
        }
    })
}

#[tokio::test]
async fn test_forward_to_socket_get() {
    let socket_path = "/tmp/test-mappy-proxy-forward.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server_handle = start_mock_unix_server(socket_path.to_string()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let request = axum::http::Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .expect("Failed to build request");
    
    let result = mappy_proxy::forward_to_socket(socket_path, request).await;
    
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.status(), 200);
    
    let _ = std::fs::remove_file(socket_path);
    server_handle.abort();
}

#[tokio::test]
async fn test_forward_to_socket_post() {
    let socket_path = "/tmp/test-mappy-proxy-forward-post.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server_handle = start_mock_unix_server(socket_path.to_string()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/")
        .body(axum::body::Body::from("{\"test\":\"data\"}"))
        .expect("Failed to build request");
    
    let result = mappy_proxy::forward_to_socket(socket_path, request).await;
    
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.status(), 201);
    
    let _ = std::fs::remove_file(socket_path);
    server_handle.abort();
}

#[tokio::test]
async fn test_forward_to_socket_connection_error() {
    let socket_path = "/tmp/nonexistent.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let request = axum::http::Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .expect("Failed to build request");
    
    let result = mappy_proxy::forward_to_socket(socket_path, request).await;
    
    assert!(result.is_err());
}
