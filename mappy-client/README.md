# mappy-client

[![Crates.io](https://img.shields.io/crates/v/mappy-client.svg)](https://crates.io/crates/mappy-client)
[![Documentation](https://docs.rs/mappy-client/badge.svg)](https://docs.rs/mappy-client)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Client library for mappy maplet data structures** - High-level client interface for distributed maplet operations.

## Overview

The mappy-client provides a convenient, high-level interface for working with maplet data structures in distributed environments. It builds on top of `mappy-core` to provide client-server communication, connection pooling, and distributed operations.

## Key Features

- **Distributed Operations**: Work with maplets across multiple nodes
- **Connection Pooling**: Efficient connection management
- **Async Support**: Full async/await support with Tokio
- **Serialization**: Built-in JSON serialization for network communication
- **Error Handling**: Comprehensive error types and recovery strategies
- **Load Balancing**: Automatic load balancing across multiple servers
- **Retry Logic**: Configurable retry policies for network operations

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
mappy-client = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use mappy_client::{MapletClient, ClientConfig};
use mappy_core::CounterOperator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ClientConfig::new()
        .add_server("127.0.0.1:8080")
        .add_server("127.0.0.1:8081")
        .timeout(std::time::Duration::from_secs(5))
        .retry_attempts(3);

    // Create client
    let mut client = MapletClient::<String, u64, CounterOperator>::new(config).await?;

    // Insert data
    client.insert("user:123".to_string(), 42).await?;
    client.insert("user:456".to_string(), 17).await?;

    // Query data
    let count = client.query("user:123".to_string()).await?;
    println!("User 123 count: {:?}", count);

    // Check existence
    let exists = client.contains("user:123".to_string()).await?;
    println!("User 123 exists: {}", exists);

    // Get statistics
    let stats = client.stats().await?;
    println!("Server stats: {:?}", stats);

    Ok(())
}
```

## Client Configuration

```rust
use mappy_client::ClientConfig;

let config = ClientConfig::new()
    .add_server("server1:8080")
    .add_server("server2:8080")
    .add_server("server3:8080")
    .timeout(std::time::Duration::from_secs(10))
    .retry_attempts(5)
    .retry_delay(std::time::Duration::from_millis(100))
    .connection_pool_size(10)
    .load_balancing_strategy(LoadBalancingStrategy::RoundRobin);
```

## Distributed Operations

### Batch Operations

```rust
use mappy_client::BatchOperation;

// Batch insert
let mut batch = BatchOperation::new();
batch.insert("key1".to_string(), 10);
batch.insert("key2".to_string(), 20);
batch.insert("key3".to_string(), 30);

client.execute_batch(batch).await?;

// Batch query
let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
let results = client.batch_query(keys).await?;
```

### Distributed Merging

```rust
// Merge maplets from multiple servers
let server_ids = vec!["server1", "server2", "server3"];
let merged_result = client.distributed_merge(server_ids).await?;
```

## Error Handling

```rust
use mappy_client::{MapletClient, ClientError};

match client.query("key".to_string()).await {
    Ok(value) => println!("Value: {:?}", value),
    Err(ClientError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
        // Handle network issues
    }
    Err(ClientError::ServerError(e)) => {
        eprintln!("Server error: {}", e);
        // Handle server errors
    }
    Err(ClientError::SerializationError(e)) => {
        eprintln!("Serialization error: {}", e);
        // Handle serialization issues
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Connection Management

```rust
use mappy_client::{MapletClient, ConnectionPool};

// Create connection pool
let pool = ConnectionPool::new()
    .max_connections(20)
    .idle_timeout(std::time::Duration::from_secs(300))
    .connection_timeout(std::time::Duration::from_secs(10));

let client = MapletClient::with_pool(pool, config).await?;

// Health check
let is_healthy = client.health_check().await?;
if !is_healthy {
    eprintln!("Client is not healthy");
}
```

## Monitoring and Metrics

```rust
use mappy_client::ClientMetrics;

// Get client metrics
let metrics = client.get_metrics().await?;
println!("Total requests: {}", metrics.total_requests);
println!("Successful requests: {}", metrics.successful_requests);
println!("Failed requests: {}", metrics.failed_requests);
println!("Average latency: {:?}", metrics.average_latency);
```

## Advanced Features

### Custom Serialization

```rust
use mappy_client::{MapletClient, CustomSerializer};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CustomData {
    id: u64,
    name: String,
    metadata: std::collections::HashMap<String, String>,
}

let serializer = CustomSerializer::<CustomData>::new();
let client = MapletClient::with_serializer(serializer, config).await?;
```

### Circuit Breaker

```rust
use mappy_client::{MapletClient, CircuitBreakerConfig};

let circuit_breaker = CircuitBreakerConfig::new()
    .failure_threshold(5)
    .recovery_timeout(std::time::Duration::from_secs(30))
    .half_open_max_calls(3);

let client = MapletClient::with_circuit_breaker(circuit_breaker, config).await?;
```

## Performance Tuning

### Connection Pooling

```rust
let config = ClientConfig::new()
    .connection_pool_size(50)  // Increase pool size for high throughput
    .max_connections_per_server(10)
    .connection_timeout(std::time::Duration::from_millis(100));
```

### Batch Size Optimization

```rust
let config = ClientConfig::new()
    .batch_size(1000)  // Larger batches for better throughput
    .batch_timeout(std::time::Duration::from_millis(50));
```

## Examples

See the `examples/` directory for comprehensive usage examples:

- `basic_client.rs` - Basic client usage
- `distributed_counter.rs` - Distributed counting system
- `load_balancing.rs` - Load balancing demonstration
- `error_handling.rs` - Error handling patterns

## Testing

```bash
# Run tests
cargo test

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench
```

## Documentation

- **[API Documentation](https://docs.rs/mappy-client)** - Complete API reference
- **[Core Library](https://crates.io/crates/mappy-core)** - Underlying maplet implementation
- **[Main Repository](https://github.com/entropy-tamer/mappy)** - Source code and examples

## License

MIT License - see [LICENSE](../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/entropy-tamer/mappy) for contribution guidelines.
