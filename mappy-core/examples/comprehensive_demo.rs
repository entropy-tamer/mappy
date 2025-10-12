//! Comprehensive demonstration of mappy features
//! 
//! This example showcases all major features of the mappy key-value store:
//! - Basic operations (get, set, delete)
//! - TTL management
//! - Different storage backends
//! - Statistics and monitoring
//! - Concurrent operations

use mappy_core::{
    Engine, EngineConfig, PersistenceMode
};
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Mappy Comprehensive Demo");
    println!("==========================\n");

    // Demo 1: Basic Operations
    demo_basic_operations().await?;
    
    // Demo 2: TTL Management
    demo_ttl_management().await?;
    
    // Demo 3: Different Storage Backends
    demo_storage_backends().await?;
    
    // Demo 4: Statistics and Monitoring
    demo_statistics().await?;
    
    // Demo 5: Concurrent Operations
    demo_concurrent_operations().await?;
    
    // Demo 6: Real-world Use Cases
    demo_real_world_use_cases().await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

/// Demo 1: Basic key-value operations
async fn demo_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: Basic Operations");
    println!("---------------------------");
    
    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;
    
    // Set some key-value pairs
    engine.set("user:1".to_string(), b"Alice".to_vec()).await?;
    engine.set("user:2".to_string(), b"Bob".to_vec()).await?;
    engine.set("user:3".to_string(), b"Charlie".to_vec()).await?;
    
    // Get values
    let user1 = engine.get("user:1").await?;
    println!("  user:1 = {:?}", String::from_utf8_lossy(&user1.unwrap()));
    
    // Check existence
    let exists = engine.exists("user:2").await?;
    println!("  user:2 exists: {}", exists);
    
    // Get all keys
    let keys = engine.keys().await?;
    println!("  All keys: {:?}", keys);
    
    // Delete a key
    let deleted = engine.delete("user:3").await?;
    println!("  Deleted user:3: {}", deleted);
    
    // Final state
    let final_keys = engine.keys().await?;
    println!("  Final keys: {:?}", final_keys);
    
    engine.close().await?;
    println!("  ✅ Basic operations demo completed\n");
    Ok(())
}

/// Demo 2: TTL (Time-To-Live) management
async fn demo_ttl_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏰ Demo 2: TTL Management");
    println!("-------------------------");
    
    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;
    
    // Set a key with TTL
    engine.set("session:abc123".to_string(), b"user_data".to_vec()).await?;
    engine.expire("session:abc123", 2).await?; // Expires in 2 seconds
    
    // Check TTL
    let ttl = engine.ttl("session:abc123").await?;
    println!("  session:abc123 TTL: {:?} seconds", ttl);
    
    // Set another key with longer TTL
    engine.set("cache:data".to_string(), b"cached_data".to_vec()).await?;
    engine.expire("cache:data", 10).await?; // Expires in 10 seconds
    
    // Wait for first key to expire
    println!("  Waiting for session:abc123 to expire...");
    sleep(Duration::from_millis(2100)).await;
    
    // Check if expired
    let expired_value = engine.get("session:abc123").await?;
    println!("  session:abc123 after expiration: {:?}", expired_value);
    
    // Check TTL of second key
    let remaining_ttl = engine.ttl("cache:data").await?;
    println!("  cache:data remaining TTL: {:?} seconds", remaining_ttl);
    
    // Remove TTL (make persistent)
    let had_ttl = engine.persist("cache:data").await?;
    println!("  Removed TTL from cache:data: {}", had_ttl);
    
    let final_ttl = engine.ttl("cache:data").await?;
    println!("  cache:data TTL after persist: {:?}", final_ttl);
    
    engine.close().await?;
    println!("  ✅ TTL management demo completed\n");
    Ok(())
}

/// Demo 3: Different storage backends
async fn demo_storage_backends() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Demo 3: Storage Backends");
    println!("---------------------------");
    
    // Memory storage
    let memory_config = EngineConfig {
        persistence_mode: PersistenceMode::Memory,
        ..Default::default()
    };
    let memory_engine = Engine::new(memory_config).await?;
    memory_engine.set("memory_key".to_string(), b"memory_value".to_vec()).await?;
    let memory_value = memory_engine.get("memory_key").await?;
    println!("  Memory storage: {:?}", String::from_utf8_lossy(&memory_value.unwrap()));
    memory_engine.close().await?;
    
    // Disk storage (using temp directory)
    let temp_dir = std::env::temp_dir().join("mappy_demo");
    std::fs::create_dir_all(&temp_dir)?;
    
    let disk_config = EngineConfig {
        persistence_mode: PersistenceMode::Disk,
        data_dir: Some(temp_dir.to_string_lossy().to_string()),
        ..Default::default()
    };
    let disk_engine = Engine::new(disk_config).await?;
    disk_engine.set("disk_key".to_string(), b"disk_value".to_vec()).await?;
    let disk_value = disk_engine.get("disk_key").await?;
    println!("  Disk storage: {:?}", String::from_utf8_lossy(&disk_value.unwrap()));
    disk_engine.close().await?;
    
    // AOF storage
    let aof_dir = std::env::temp_dir().join("mappy_aof_demo");
    std::fs::create_dir_all(&aof_dir)?;
    
    let aof_config = EngineConfig {
        persistence_mode: PersistenceMode::AOF,
        data_dir: Some(aof_dir.to_string_lossy().to_string()),
        ..Default::default()
    };
    let aof_engine = Engine::new(aof_config).await?;
    aof_engine.set("aof_key".to_string(), b"aof_value".to_vec()).await?;
    let aof_value = aof_engine.get("aof_key").await?;
    println!("  AOF storage: {:?}", String::from_utf8_lossy(&aof_value.unwrap()));
    aof_engine.close().await?;
    
    println!("  ✅ Storage backends demo completed\n");
    Ok(())
}

/// Demo 4: Statistics and monitoring
async fn demo_statistics() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 4: Statistics and Monitoring");
    println!("------------------------------------");
    
    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;
    
    // Perform some operations
    for i in 0..100 {
        engine.set(format!("key_{}", i), format!("value_{}", i).into_bytes()).await?;
    }
    
    // Set some TTLs
    for i in 0..10 {
        engine.expire(&format!("key_{}", i), 60).await?;
    }
    
    // Get statistics
    let stats = engine.stats().await?;
    println!("  Engine Statistics:");
    println!("    Uptime: {} seconds", stats.uptime_seconds);
    println!("    Total operations: {}", stats.total_operations);
    println!("    Maplet stats: {:?}", stats.maplet_stats);
    println!("    Storage stats: {:?}", stats.storage_stats);
    println!("    TTL stats: {:?}", stats.ttl_stats);
    
    engine.close().await?;
    println!("  ✅ Statistics demo completed\n");
    Ok(())
}

/// Demo 5: Concurrent operations
async fn demo_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Demo 5: Concurrent Operations");
    println!("-------------------------------");
    
    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;
    let engine_arc = Arc::new(engine);
    
    // Spawn multiple concurrent tasks
    let handles: Vec<_> = (0..10).map(|i| {
        let engine = Arc::clone(&engine_arc);
        tokio::spawn(async move {
            for j in 0..100 {
                let key = format!("concurrent_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j);
                engine.set(key, value.into_bytes()).await.unwrap();
            }
        })
    }).collect();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }
    
    // Verify results
    let total_keys = engine_arc.keys().await?.len();
    println!("  Total keys after concurrent operations: {}", total_keys);
    
    // Test concurrent reads
    let read_handles: Vec<_> = (0..5).map(|i| {
        let engine = Arc::clone(&engine_arc);
        tokio::spawn(async move {
            for j in 0..50 {
                let key = format!("concurrent_{}_{}", i, j);
                let value = engine.get(&key).await.unwrap();
                assert!(value.is_some());
            }
        })
    }).collect();
    
    for handle in read_handles {
        handle.await?;
    }
    
    println!("  ✅ Concurrent operations completed successfully");
    
    engine_arc.close().await?;
    println!("  ✅ Concurrent operations demo completed\n");
    Ok(())
}

/// Demo 6: Real-world use cases
async fn demo_real_world_use_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Demo 6: Real-world Use Cases");
    println!("-------------------------------");
    
    let config = EngineConfig::default();
    let engine = Engine::new(config).await?;
    
    // Use case 1: Session management
    println!("  📱 Session Management:");
    engine.set("session:user123".to_string(), b"user_data".to_vec()).await?;
    engine.expire("session:user123", 3600).await?; // 1 hour
    let session_ttl = engine.ttl("session:user123").await?;
    println!("    Session TTL: {:?} seconds", session_ttl);
    
    // Use case 2: Caching
    println!("  🗄️  Caching:");
    engine.set("cache:expensive_calculation".to_string(), b"result_data".to_vec()).await?;
    engine.expire("cache:expensive_calculation", 300).await?; // 5 minutes
    let cache_value = engine.get("cache:expensive_calculation").await?;
    println!("    Cache hit: {:?}", String::from_utf8_lossy(&cache_value.unwrap()));
    
    // Use case 3: Rate limiting
    println!("  🚦 Rate Limiting:");
    let client_id = "client:192.168.1.1";
    engine.set(client_id.to_string(), b"1".to_vec()).await?;
    engine.expire(client_id, 60).await?; // 1 minute window
    
    // Simulate multiple requests
    for i in 2..=5 {
        let current = engine.get(client_id).await?;
        if let Some(count_bytes) = current {
            let count = String::from_utf8_lossy(&count_bytes).parse::<u32>().unwrap_or(0);
            if count < 3 { // Allow 3 requests per minute
                engine.set(client_id.to_string(), (count + 1).to_string().into_bytes()).await?;
                println!("    Request {}: Allowed (count: {})", i, count + 1);
            } else {
                println!("    Request {}: Rate limited (count: {})", i, count);
            }
        }
    }
    
    // Use case 4: Feature flags
    println!("  🚩 Feature Flags:");
    engine.set("feature:new_ui".to_string(), b"enabled".to_vec()).await?;
    engine.set("feature:beta_feature".to_string(), b"disabled".to_vec()).await?;
    
    let new_ui = engine.get("feature:new_ui").await?;
    let beta_feature = engine.get("feature:beta_feature").await?;
    println!("    New UI: {:?}", String::from_utf8_lossy(&new_ui.unwrap()));
    println!("    Beta Feature: {:?}", String::from_utf8_lossy(&beta_feature.unwrap()));
    
    engine.close().await?;
    println!("  ✅ Real-world use cases demo completed\n");
    Ok(())
}

