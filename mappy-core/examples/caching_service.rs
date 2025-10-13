//! Real-world caching service example
//! 
//! This example demonstrates how to use Mappy as a caching layer for a web service.
//! It includes cache invalidation, TTL management, and performance monitoring.

use mappy_core::{Engine, EngineConfig, PersistenceMode};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author_id: u32,
    created_at: String,
}

/// Simulated database operations
struct Database {
    users: HashMap<u32, User>,
    posts: HashMap<u32, Post>,
}

impl Database {
    fn new() -> Self {
        let mut users = HashMap::new();
        let mut posts = HashMap::new();
        
        // Add some sample data
        for i in 1..=100 {
            users.insert(i, User {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            });
            
            posts.insert(i, Post {
                id: i,
                title: format!("Post {}", i),
                content: format!("This is the content of post {}", i),
                author_id: i,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            });
        }
        
        Self { users, posts }
    }
    
    /// Simulate slow database query
    async fn get_user(&self, id: u32) -> Option<User> {
        // Simulate database latency
        sleep(Duration::from_millis(100)).await;
        self.users.get(&id).cloned()
    }
    
    /// Simulate slow database query
    async fn get_post(&self, id: u32) -> Option<Post> {
        // Simulate database latency
        sleep(Duration::from_millis(150)).await;
        self.posts.get(&id).cloned()
    }
    
    /// Simulate database update
    async fn update_user(&self, user: User) -> User {
        // Simulate database latency
        sleep(Duration::from_millis(200)).await;
        user
    }
}

/// Caching service using Mappy
struct CachingService {
    cache: Engine,
    db: Database,
    cache_hits: u64,
    cache_misses: u64,
}

impl CachingService {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = EngineConfig {
            persistence_mode: PersistenceMode::Memory, // Use memory for fast access
            ttl: mappy_core::TTLConfig {
                enable_background_cleanup: true,
                cleanup_interval_secs: 1, // Cleanup every second
                max_cleanup_batch_size: 1000,
            },
            ..Default::default()
        };
        
        let cache = Engine::new(config).await?;
        let db = Database::new();
        
        Ok(Self {
            cache,
            db,
            cache_hits: 0,
            cache_misses: 0,
        })
    }
    
    /// Get user with caching
    async fn get_user(&mut self, id: u32) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let cache_key = format!("user:{}", id);
        
        // Try to get from cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await? {
            self.cache_hits += 1;
            let user: User = serde_json::from_slice(&cached_data)?;
            println!("  Cache HIT for user:{}", id);
            return Ok(Some(user));
        }
        
        // Cache miss - get from database
        self.cache_misses += 1;
        println!("  Cache MISS for user:{}", id);
        
        if let Some(user) = self.db.get_user(id).await {
            // Store in cache with TTL
            let user_data = serde_json::to_vec(&user)?;
            self.cache.set(cache_key, user_data).await?;
            self.cache.expire(&format!("user:{}", id), 300).await?; // 5 minutes TTL
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    /// Get post with caching
    async fn get_post(&mut self, id: u32) -> Result<Option<Post>, Box<dyn std::error::Error>> {
        let cache_key = format!("post:{}", id);
        
        // Try to get from cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await? {
            self.cache_hits += 1;
            let post: Post = serde_json::from_slice(&cached_data)?;
            println!("  Cache HIT for post:{}", id);
            return Ok(Some(post));
        }
        
        // Cache miss - get from database
        self.cache_misses += 1;
        println!("  Cache MISS for post:{}", id);
        
        if let Some(post) = self.db.get_post(id).await {
            // Store in cache with TTL
            let post_data = serde_json::to_vec(&post)?;
            self.cache.set(cache_key, post_data).await?;
            self.cache.expire(&format!("post:{}", id), 600).await?; // 10 minutes TTL
            
            Ok(Some(post))
        } else {
            Ok(None)
        }
    }
    
    /// Update user and invalidate cache
    async fn update_user(&mut self, user: User) -> Result<User, Box<dyn std::error::Error>> {
        // Update in database
        let updated_user = self.db.update_user(user.clone()).await;
        
        // Invalidate cache
        let cache_key = format!("user:{}", user.id);
        self.cache.delete(&cache_key).await?;
        
        // Store updated user in cache
        let user_data = serde_json::to_vec(&updated_user)?;
        self.cache.set(cache_key, user_data).await?;
        self.cache.expire(&format!("user:{}", user.id), 300).await?; // 5 minutes TTL
        
        println!("  Updated and cached user:{}", user.id);
        Ok(updated_user)
    }
    
    /// Get cache statistics
    async fn get_cache_stats(&self) -> Result<CacheStats, Box<dyn std::error::Error>> {
        let engine_stats = self.cache.stats().await?;
        let total_requests = self.cache_hits + self.cache_misses;
        let hit_rate = if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };
        
        Ok(CacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate,
            total_keys: engine_stats.storage_stats.total_keys,
            memory_usage: self.cache.memory_usage().await?,
            uptime_seconds: engine_stats.uptime_seconds,
        })
    }
    
    /// Clear all cache
    async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.cache.clear().await?;
        println!("  Cache cleared");
        Ok(())
    }
}

#[derive(Debug)]
struct CacheStats {
    cache_hits: u64,
    cache_misses: u64,
    hit_rate: f64,
    total_keys: u64,
    memory_usage: u64,
    uptime_seconds: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Caching Service Demo");
    println!("======================\n");
    
    let mut service = CachingService::new().await?;
    
    // Test 1: Initial cache misses
    println!("📝 Test 1: Initial Cache Misses");
    println!("{}", "-".repeat(40));
    
    let start = Instant::now();
    let user1 = service.get_user(1).await?;
    let user1_time = start.elapsed();
    
    println!("  Retrieved user 1: {:?}", user1);
    println!("  Time taken: {:?}", user1_time);
    
    // Test 2: Cache hits
    println!("\n📝 Test 2: Cache Hits");
    println!("{}", "-".repeat(40));
    
    let start = Instant::now();
    let user1_cached = service.get_user(1).await?;
    let user1_cached_time = start.elapsed();
    
    println!("  Retrieved user 1 (cached): {:?}", user1_cached);
    println!("  Time taken: {:?}", user1_cached_time);
    println!("  Speedup: {:.2}x", user1_time.as_millis() as f64 / user1_cached_time.as_millis() as f64);
    
    // Test 3: Multiple users
    println!("\n📝 Test 3: Multiple Users");
    println!("{}", "-".repeat(40));
    
    for i in 1..=5 {
        let start = Instant::now();
        let user = service.get_user(i).await?;
        let time = start.elapsed();
        println!("  User {}: {:?} (took {:?})", i, user.as_ref().map(|u| &u.name), time);
    }
    
    // Test 4: Posts
    println!("\n📝 Test 4: Posts");
    println!("{}", "-".repeat(40));
    
    for i in 1..=3 {
        let start = Instant::now();
        let post = service.get_post(i).await?;
        let time = start.elapsed();
        println!("  Post {}: {:?} (took {:?})", i, post.as_ref().map(|p| &p.title), time);
    }
    
    // Test 5: Cache statistics
    println!("\n📊 Test 5: Cache Statistics");
    println!("{}", "-".repeat(40));
    
    let stats = service.get_cache_stats().await?;
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("  Total keys: {}", stats.total_keys);
    println!("  Memory usage: {} bytes", stats.memory_usage);
    println!("  Uptime: {} seconds", stats.uptime_seconds);
    
    // Test 6: Cache invalidation
    println!("\n📝 Test 6: Cache Invalidation");
    println!("{}", "-".repeat(40));
    
    if let Some(mut user) = service.get_user(1).await? {
        user.name = "Updated User 1".to_string();
        let updated_user = service.update_user(user).await?;
        println!("  Updated user: {:?}", updated_user);
        
        // Verify cache was updated
        let cached_user = service.get_user(1).await?;
        println!("  Cached user after update: {:?}", cached_user);
    }
    
    // Test 7: TTL expiration
    println!("\n📝 Test 7: TTL Expiration");
    println!("{}", "-".repeat(40));
    
    // Set a short TTL for testing
    service.cache.expire("user:1", 2).await?; // 2 seconds
    println!("  Set TTL for user:1 to 2 seconds");
    
    // Wait for expiration
    println!("  Waiting for TTL to expire...");
    sleep(Duration::from_millis(2100)).await;
    
    // Try to get the expired user
    let expired_user = service.get_user(1).await?;
    println!("  User after TTL expiration: {:?}", expired_user);
    
    // Test 8: Final statistics
    println!("\n📊 Test 8: Final Statistics");
    println!("{}", "-".repeat(40));
    
    let final_stats = service.get_cache_stats().await?;
    println!("  Final cache hits: {}", final_stats.cache_hits);
    println!("  Final cache misses: {}", final_stats.cache_misses);
    println!("  Final hit rate: {:.2}%", final_stats.hit_rate * 100.0);
    println!("  Final memory usage: {} bytes", final_stats.memory_usage);
    
    // Test 9: Cache clearing
    println!("\n📝 Test 9: Cache Clearing");
    println!("{}", "-".repeat(40));
    
    service.clear_cache().await?;
    
    let cleared_stats = service.get_cache_stats().await?;
    println!("  Keys after clearing: {}", cleared_stats.total_keys);
    
    println!("\n✅ Caching service demo completed!");
    Ok(())
}
