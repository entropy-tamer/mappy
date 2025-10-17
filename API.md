# Mappy API Documentation

## Overview

Mappy is a high-performance, space-efficient key-value store built on maplet technology. It provides approximate membership testing with configurable false positive rates, making it ideal for caching, session management, and other applications where space efficiency is critical.

## Core Concepts

### Maplets

A maplet is a space-efficient data structure that provides approximate key-value mappings. It uses quotient filters and perfect hashing to achieve:

- **Space efficiency**: Uses significantly less memory than traditional hash tables
- **Configurable accuracy**: Adjustable false positive rates
- **Fast operations**: O(1) average case for insert, query, and delete operations
- **Merge operations**: Built-in support for value merging (counters, sets, etc.)

### Storage Backends

Mappy supports multiple storage backends:

- **Memory**: In-memory only, fastest but not persistent
- **Disk**: Full persistence using Sled database
- **AOF**: Append-only file logging for durability
- **Hybrid**: Combines memory cache with AOF logging

## API Reference

### Engine

The `Engine` is the main interface for interacting with Mappy.

#### Creating an Engine

```rust
use mappy_core::{Engine, EngineConfig, PersistenceMode};

// Default configuration (memory storage)
let engine = Engine::new(EngineConfig::default()).await?;

// Custom configuration
let config = EngineConfig {
    persistence_mode: PersistenceMode::Disk,
    data_dir: Some("/path/to/data".to_string()),
    maplet: MapletConfig {
        capacity: 10000,
        false_positive_rate: 0.01,
        auto_resize: true,
    },
    storage: StorageConfig {
        memory_capacity: Some(1024 * 1024), // 1MB cache
        aof_sync_interval_ms: Some(1000),   // 1 second sync
    },
    ttl: TTLConfig {
        enabled: true,
        cleanup_interval_ms: 1000,
    },
};
let engine = Engine::new(config).await?;
```

#### Basic Operations

```rust
// Set a key-value pair
engine.set("user:123".to_string(), b"Alice".to_vec()).await?;

// Get a value
let value = engine.get("user:123").await?;
match value {
    Some(data) => println!("User: {}", String::from_utf8_lossy(&data)),
    None => println!("User not found"),
}

// Check if a key exists
let exists = engine.exists("user:123").await?;

// Delete a key
let deleted = engine.delete("user:123").await?;

// Get all keys
let keys = engine.keys().await?;

// Clear all data
engine.clear().await?;
```

#### TTL (Time-To-Live) Operations

```rust
// Set a key with TTL (expires in 60 seconds)
engine.set("session:abc".to_string(), b"session_data".to_vec()).await?;
engine.expire("session:abc", 60).await?;

// Check remaining TTL
let ttl = engine.ttl("session:abc").await?;
match ttl {
    Some(seconds) => println!("Expires in {} seconds", seconds),
    None => println!("No TTL set"),
}

// Remove TTL (make persistent)
let had_ttl = engine.persist("session:abc").await?;

// Set TTL for multiple keys
let keys = vec!["key1".to_string(), "key2".to_string()];
engine.expire_many(keys, 300).await?; // 5 minutes

// Get all keys with TTL
let ttl_keys = engine.keys_with_ttl().await?;
```

#### Statistics and Monitoring

```rust
// Get comprehensive statistics
let stats = engine.stats().await?;
println!("Uptime: {} seconds", stats.uptime_seconds);
println!("Total operations: {}", stats.total_operations);
println!("Maplet stats: {:?}", stats.maplet_stats);
println!("Storage stats: {:?}", stats.storage_stats);
println!("TTL stats: {:?}", stats.ttl_stats);

// Get memory usage
let memory_usage = engine.memory_usage().await?;
println!("Memory usage: {} bytes", memory_usage);
```

#### Cleanup and Maintenance

```rust
// Flush pending writes to disk
engine.flush().await?;

// Close the engine (cleanup resources)
engine.close().await?;
```

### Configuration

#### EngineConfig

```rust
pub struct EngineConfig {
    /// Persistence mode for storage
    pub persistence_mode: PersistenceMode,
    /// Data directory for persistent storage
    pub data_dir: Option<String>,
    /// Maplet configuration
    pub maplet: MapletConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// TTL configuration
    pub ttl: TTLConfig,
}
```

#### MapletConfig

```rust
pub struct MapletConfig {
    /// Initial capacity
    pub capacity: usize,
    /// Target false positive rate (0.0 to 1.0)
    pub false_positive_rate: f64,
    /// Whether to auto-resize when capacity is exceeded
    pub auto_resize: bool,
}
```

#### StorageConfig

```rust
pub struct StorageConfig {
    /// Memory capacity for cache (bytes)
    pub memory_capacity: Option<usize>,
    /// AOF sync interval (milliseconds)
    pub aof_sync_interval_ms: Option<u64>,
}
```

#### TTLConfig

```rust
pub struct TTLConfig {
    /// Whether TTL is enabled
    pub enabled: bool,
    /// Cleanup interval (milliseconds)
    pub cleanup_interval_ms: u64,
}
```

### Persistence Modes

#### Memory

- **Use case**: Temporary data, testing, high-performance scenarios
- **Pros**: Fastest operations, no disk I/O
- **Cons**: Data lost on restart, limited by RAM

```rust
let config = EngineConfig {
    persistence_mode: PersistenceMode::Memory,
    ..Default::default()
};
```

#### Disk

- **Use case**: Persistent data with ACID guarantees
- **Pros**: Full durability, crash recovery
- **Cons**: Slower than memory, disk space usage

```rust
let config = EngineConfig {
    persistence_mode: PersistenceMode::Disk,
    data_dir: Some("/var/lib/mappy".to_string()),
    ..Default::default()
};
```

#### AOF (Append-Only File)

- **Use case**: Durability with good performance
- **Pros**: Fast writes, crash recovery, compact logs
- **Cons**: Slower reads, log file growth

```rust
let config = EngineConfig {
    persistence_mode: PersistenceMode::AOF,
    data_dir: Some("/var/lib/mappy".to_string()),
    storage: StorageConfig {
        aof_sync_interval_ms: Some(1000), // 1 second sync
        ..Default::default()
    },
    ..Default::default()
};
```

#### Hybrid

- **Use case**: Best of both worlds
- **Pros**: Fast reads from memory, durable writes
- **Cons**: More complex, higher memory usage

```rust
let config = EngineConfig {
    persistence_mode: PersistenceMode::Hybrid,
    data_dir: Some("/var/lib/mappy".to_string()),
    storage: StorageConfig {
        memory_capacity: Some(1024 * 1024), // 1MB cache
        aof_sync_interval_ms: Some(1000),
        ..Default::default()
    },
    ..Default::default()
};
```

## Error Handling

Mappy uses the `MapletError` enum for error handling:

```rust
use mappy_core::MapletError;

match engine.set("key".to_string(), b"value".to_vec()).await {
    Ok(()) => println!("Success"),
    Err(MapletError::CapacityExceeded) => println!("Maplet is full"),
    Err(MapletError::InvalidConfig) => println!("Invalid configuration"),
    Err(MapletError::StorageError(msg)) => println!("Storage error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

### Common Error Types

- `CapacityExceeded`: Maplet has reached its capacity limit
- `InvalidConfig`: Configuration parameters are invalid
- `StorageError`: Storage backend error
- `TTLError`: TTL-related error
- `SerializationError`: Data serialization/deserialization error

## Performance Considerations

### Memory Usage

- Maplets use significantly less memory than traditional hash tables
- Memory usage scales with capacity and false positive rate
- Use `engine.memory_usage().await?` to monitor memory consumption

### False Positive Rate

- Lower false positive rates require more memory
- Typical values: 0.01 (1%) to 0.1 (10%)
- Use `engine.stats().await?` to monitor actual false positive rate

### Concurrent Access

- All operations are thread-safe and async
- Multiple readers can access simultaneously
- Writers are serialized for consistency

### Storage Backend Selection

- **Memory**: Use for temporary data or when persistence isn't needed
- **Disk**: Use when you need full ACID guarantees
- **AOF**: Use for good balance of performance and durability
- **Hybrid**: Use when you need both fast reads and durability

## Best Practices

### Key Naming

- Use hierarchical keys: `user:123`, `session:abc`, `cache:data`
- Avoid special characters that might cause issues
- Keep keys reasonably short for better performance

### TTL Management

- Set appropriate TTLs based on your use case
- Use `engine.keys_with_ttl().await?` to monitor TTL usage
- Consider cleanup intervals based on your data patterns

### Error Handling

- Always handle `MapletError` appropriately
- Use `engine.stats().await?` to monitor health
- Implement proper cleanup with `engine.close().await?`

### Configuration

- Start with default configuration and tune as needed
- Monitor memory usage and adjust capacity accordingly
- Choose persistence mode based on your durability requirements

## Examples

### Session Management

```rust
use mappy_core::{Engine, EngineConfig, PersistenceMode};

async fn create_session(engine: &Engine, user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let session_id = format!("session:{}", uuid::Uuid::new_v4());
    let session_data = format!("user_id:{}", user_id);

    engine.set(session_id.clone(), session_data.into_bytes()).await?;
    engine.expire(&session_id, 3600).await?; // 1 hour

    Ok(session_id)
}

async fn validate_session(engine: &Engine, session_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let exists = engine.exists(session_id).await?;
    Ok(exists)
}
```

### Caching

```rust
async fn get_cached_data(engine: &Engine, key: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let cache_key = format!("cache:{}", key);
    let data = engine.get(&cache_key).await?;
    Ok(data)
}

async fn set_cached_data(engine: &Engine, key: &str, data: Vec<u8>, ttl_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    let cache_key = format!("cache:{}", key);
    engine.set(cache_key.clone(), data).await?;
    engine.expire(&cache_key, ttl_seconds).await?;
    Ok(())
}
```

### Rate Limiting

```rust
async fn check_rate_limit(engine: &Engine, client_id: &str, max_requests: u32, window_seconds: u64) -> Result<bool, Box<dyn std::error::Error>> {
    let rate_key = format!("rate_limit:{}", client_id);

    let current_count = engine.get(&rate_key).await?;
    let count = match current_count {
        Some(data) => String::from_utf8_lossy(&data).parse::<u32>().unwrap_or(0),
        None => 0,
    };

    if count >= max_requests {
        return Ok(false); // Rate limited
    }

    // Increment counter
    engine.set(rate_key.clone(), (count + 1).to_string().into_bytes()).await?;
    engine.expire(&rate_key, window_seconds).await?;

    Ok(true) // Allowed
}
```

## Migration Guide

### From HashMap/BTreeMap

1. Replace direct hash map operations with engine operations
2. Handle the `MapletError` type for error handling
3. Use async/await for all operations
4. Consider TTL for temporary data

### From Redis

1. Replace Redis commands with engine methods
2. Use hierarchical key naming
3. Handle TTL operations explicitly
4. Monitor memory usage and false positive rates

## Troubleshooting

### High Memory Usage

- Check if auto-resize is enabled
- Monitor false positive rate
- Consider reducing capacity if not needed
- Use `engine.stats().await?` to analyze usage

### Performance Issues

- Choose appropriate persistence mode
- Monitor storage backend performance
- Consider using memory mode for temporary data
- Use `engine.stats().await?` to identify bottlenecks

### Data Loss

- Ensure proper persistence mode is selected
- Use `engine.flush().await?` before shutdown
- Check TTL settings
- Monitor storage backend health

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on contributing to Mappy.

## License

Mappy is licensed under the MIT License. See [LICENSE](LICENSE) for details.
