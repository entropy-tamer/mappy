//! # Mappy Client
//!
//! Client library for mappy maplet data structures with network support.

pub use mappy_core::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use mappy_core::types::MapletConfig;
    pub use mappy_core::{
        CounterOperator, Maplet, MapletError, MapletResult, MapletStats, MaxOperator, MinOperator,
        SetOperator,
    };
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Unix socket path (default: /var/run/reynard/mappy.sock)
    pub socket_path: Option<String>,
    /// HTTP URL (e.g., http://localhost:8003)
    pub http_url: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            socket_path: Some("/var/run/reynard/mappy.sock".to_string()),
            http_url: None,
        }
    }
}

impl ClientConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let socket_path = std::env::var("MAPPY_SOCKET_PATH")
            .ok()
            .filter(|s| !s.is_empty());
        let http_url = std::env::var("MAPPY_HTTP_URL")
            .ok()
            .filter(|s| !s.is_empty());

        Self {
            socket_path: socket_path.or_else(|| Some("/var/run/reynard/mappy.sock".to_string())),
            http_url,
        }
    }

    /// Use Unix socket
    pub fn with_socket<P: AsRef<Path>>(path: P) -> Self {
        Self {
            socket_path: Some(path.as_ref().to_string_lossy().to_string()),
            http_url: None,
        }
    }

    /// Use HTTP
    pub fn with_http(url: impl Into<String>) -> Self {
        Self {
            socket_path: None,
            http_url: Some(url.into()),
        }
    }
}

/// Network client for Mappy server
pub struct Client {
    config: ClientConfig,
}

#[derive(Serialize)]
struct SetRequest {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct GetResponse {
    value: Option<String>,
}

#[derive(Deserialize)]
pub struct HealthResponse;

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Create a client from environment variables
    pub fn from_env() -> Self {
        Self {
            config: ClientConfig::from_env(),
        }
    }

    /// Send a request via Unix socket
    async fn request_unix(
        &self,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> anyhow::Result<Vec<u8>> {
        let socket_path = self
            .config
            .socket_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Unix socket path not configured"))?;

        let mut stream = UnixStream::connect(socket_path)
            .await
            .with_context(|| format!("Failed to connect to Unix socket: {}", socket_path))?;

        // Build HTTP request
        let mut request = format!("{} {} HTTP/1.1\r\n", method, path);
        request.push_str("Host: localhost\r\n");
        request.push_str("Content-Type: application/json\r\n");

        if let Some(body) = body {
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        request.push_str("\r\n");

        // Send request
        stream.write_all(request.as_bytes()).await?;
        if let Some(body) = body {
            stream.write_all(body).await?;
        }
        stream.flush().await?;

        // Read response
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        // Parse HTTP response (simple parser)
        let response_str = String::from_utf8_lossy(&response);
        let body_start = response_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);

        Ok(response[body_start..].to_vec())
    }

    /// Send a request via HTTP
    async fn request_http(
        &self,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> anyhow::Result<Vec<u8>> {
        let url = self
            .config
            .http_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP URL not configured"))?;

        let client = reqwest::Client::new();
        let url = format!("{}{}", url.trim_end_matches('/'), path);

        let response = match method {
            "GET" => client.get(&url).send().await?,
            "POST" => {
                let mut req = client.post(&url);
                if let Some(body) = body {
                    req = req.body(body.to_vec());
                }
                req.send().await?
            }
            _ => return Err(anyhow::anyhow!("Unsupported method: {}", method)),
        };

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Send a request (auto-detects socket vs HTTP)
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> anyhow::Result<Vec<u8>> {
        if self.config.socket_path.is_some() {
            self.request_unix(method, path, body).await
        } else if self.config.http_url.is_some() {
            self.request_http(method, path, body).await
        } else {
            Err(anyhow::anyhow!("No connection method configured"))
        }
    }

    /// Check server health
    pub async fn health(&self) -> anyhow::Result<HealthResponse> {
        let response = self.request("GET", "/health", None).await?;
        let health: HealthResponse = serde_json::from_slice(&response)?;
        Ok(health)
    }

    /// Set a key-value pair
    pub async fn set(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> anyhow::Result<()> {
        let request = SetRequest {
            key: key.into(),
            value: value.into(),
        };
        let body = serde_json::to_vec(&request)?;
        self.request("POST", "/set", Some(&body)).await?;
        Ok(())
    }

    /// Get a value by key
    pub async fn get(&self, key: impl Into<String>) -> anyhow::Result<Option<String>> {
        let key = key.into();
        let path = format!("/get/{}", urlencoding::encode(&key));
        let response = self.request("GET", &path, None).await?;
        let get_response: GetResponse = serde_json::from_slice(&response)?;
        Ok(get_response.value)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use super::{Client, ClientConfig, GetResponse, SetRequest};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_client_basic_usage() {
        // Maplet's mutating/reading methods take `self: Arc<Self>`, so the handle
        // is an Arc and every call clones it. See mappy-core/src/maplet.rs:120.
        let maplet = Arc::new(
            Maplet::<String, u64, CounterOperator>::new(100, 0.01)
                .expect("Failed to create maplet"),
        );

        Arc::clone(&maplet)
            .insert("test".to_string(), 42)
            .await
            .expect("Failed to insert test key");
        assert_eq!(
            Arc::clone(&maplet).query("test".to_string()).await,
            Some(42)
        );
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(
            config.socket_path,
            Some("/var/run/reynard/mappy.sock".to_string())
        );
        assert_eq!(config.http_url, None);
    }

    #[test]
    fn test_client_config_from_env_empty() {
        unsafe { std::env::set_var("MAPPY_SOCKET_PATH", "") };
        unsafe { std::env::set_var("MAPPY_HTTP_URL", "") };
        let config = ClientConfig::from_env();
        assert_eq!(
            config.socket_path,
            Some("/var/run/reynard/mappy.sock".to_string())
        );
        assert_eq!(config.http_url, None);
        unsafe { std::env::remove_var("MAPPY_SOCKET_PATH") };
        unsafe { std::env::remove_var("MAPPY_HTTP_URL") };
    }

    #[test]
    fn test_client_config_with_socket() {
        let config = ClientConfig::with_socket("/tmp/custom.sock");
        assert_eq!(config.socket_path, Some("/tmp/custom.sock".to_string()));
        assert_eq!(config.http_url, None);
    }

    #[test]
    fn test_client_config_with_http() {
        let config = ClientConfig::with_http("http://example.com:8080");
        assert_eq!(config.socket_path, None);
        assert_eq!(config.http_url, Some("http://example.com:8080".to_string()));
    }

    #[test]
    fn test_client_new() {
        let client = Client::new();
        assert_eq!(
            client.config.socket_path,
            Some("/var/run/reynard/mappy.sock".to_string())
        );
        assert_eq!(client.config.http_url, None);
    }

    #[test]
    fn test_client_with_config() {
        let config = ClientConfig::with_socket("/tmp/test.sock");
        let client = Client::with_config(config);
        assert_eq!(
            client.config.socket_path,
            Some("/tmp/test.sock".to_string())
        );
    }

    #[test]
    fn test_client_default() {
        let client = Client::default();
        assert_eq!(
            client.config.socket_path,
            Some("/var/run/reynard/mappy.sock".to_string())
        );
    }

    #[test]
    fn test_set_request_serialization() {
        let request = SetRequest {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };
        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(serialized.contains("test_key"));
        assert!(serialized.contains("test_value"));
    }

    #[test]
    fn test_get_response_deserialization() {
        let json = r#"{"value": "test_value"}"#;
        let response: GetResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.value, Some("test_value".to_string()));
    }

    #[test]
    fn test_get_response_none() {
        let json = r#"{"value": null}"#;
        let response: GetResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.value, None);
    }

    #[tokio::test]
    async fn test_request_unix_no_socket() {
        let config = ClientConfig {
            socket_path: None,
            http_url: None,
        };
        let client = Client::with_config(config);
        let result = client.request_unix("GET", "/health", None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unix socket path not configured")
        );
    }

    #[tokio::test]
    async fn test_request_http_no_url() {
        let config = ClientConfig {
            socket_path: None,
            http_url: None,
        };
        let client = Client::with_config(config);
        let result = client.request_http("GET", "/health", None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("HTTP URL not configured")
        );
    }

    #[tokio::test]
    async fn test_request_no_config() {
        let config = ClientConfig {
            socket_path: None,
            http_url: None,
        };
        let client = Client::with_config(config);
        let result = client.request("GET", "/health", None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No connection method configured")
        );
    }

    #[tokio::test]
    async fn test_request_http_unsupported_method() {
        let config = ClientConfig {
            socket_path: None,
            http_url: Some("http://localhost:8003".to_string()),
        };
        let client = Client::with_config(config);
        let result = client.request_http("DELETE", "/health", None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported method")
        );
    }
}
