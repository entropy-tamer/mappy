//! Network routing table example
//! 
//! Demonstrates using maplets for network routing tables,
//! as described in Section 6 of the research paper.

use mappy_core::{Maplet, SetOperator};
use std::collections::{HashMap, HashSet};

/// Simulate network routing using a maplet
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Network Routing with Maplets");
    println!("============================");
    
    // Create a maplet for routing table (prefix -> set of next hops)
    let config = mappy_core::types::MapletConfig::new(1000, 0.01);
    let routing_table = Maplet::<String, HashSet<String>, SetOperator>::with_config(config)?;
    
    // Simulate network topology
    let routes = vec![
        ("192.168.1.0/24", vec!["router1", "router2"]),
        ("10.0.0.0/8", vec!["router3"]),
        ("172.16.0.0/12", vec!["router4", "router5", "router6"]),
        ("0.0.0.0/0", vec!["default_gateway"]), // Default route
    ];
    
    println!("Building routing table...");
    
    // Add routes to the maplet
    for (prefix, next_hops) in &routes {
        let mut hop_set = HashSet::new();
        for hop in next_hops {
            hop_set.insert(hop.to_string());
        }
        routing_table.insert(prefix.to_string(), hop_set).await?;
        println!("  Added route: {} -> {:?}", prefix, next_hops);
    }
    
    // Simulate route lookups
    let test_ips = vec![
        "192.168.1.100",
        "10.1.2.3", 
        "172.16.1.1",
        "8.8.8.8",
        "203.0.113.1",
    ];
    
    println!("\nRoute lookups:");
    for ip in &test_ips {
        let route = find_best_route(&routing_table, ip).await;
        println!("  {} -> {:?}", ip, route);
    }
    
    // Show statistics
    let stats = routing_table.stats().await;
    println!("\nRouting Table Statistics:");
    println!("  Capacity: {}", stats.capacity);
    println!("  Routes: {}", stats.len);
    println!("  Load factor: {:.2}%", stats.load_factor * 100.0);
    println!("  Memory usage: {} bytes", stats.memory_usage);
    
    // Compare with traditional HashMap
    let mut hashmap_routes: HashMap<String, HashSet<String>> = HashMap::new();
    for (prefix, next_hops) in &routes {
        let mut hop_set = HashSet::new();
        for hop in next_hops {
            hop_set.insert(hop.to_string());
        }
        hashmap_routes.insert(prefix.to_string(), hop_set);
    }
    
    println!("\nComparison with HashMap:");
    for ip in &test_ips {
        let maplet_route = find_best_route(&routing_table, ip).await;
        let hashmap_route = find_best_route_hashmap(&hashmap_routes, ip);
        println!("  {}: Maplet={:?}, HashMap={:?}", ip, maplet_route, hashmap_route);
    }
    
    Ok(())
}

/// Find the best route for an IP address using the maplet
async fn find_best_route(
    routing_table: &Maplet<String, HashSet<String>, SetOperator>,
    ip: &str,
) -> Option<HashSet<String>> {
    // Simple longest prefix match simulation
    let ip_parts: Vec<&str> = ip.split('.').collect();
    
    // Try different prefix lengths
    for prefix_len in (0..=4).rev() {
        if prefix_len == 0 {
            // Try default route
            if let Some(routes) = routing_table.query(&"0.0.0.0/0".to_string()).await {
                return Some(routes);
            }
        } else {
            let prefix = format!("{}/{}", 
                ip_parts[..prefix_len].join("."),
                prefix_len * 8
            );
            
            if let Some(routes) = routing_table.query(&prefix).await {
                return Some(routes);
            }
        }
    }
    
    None
}

/// Find the best route for an IP address using HashMap
fn find_best_route_hashmap(
    routing_table: &HashMap<String, HashSet<String>>,
    ip: &str,
) -> Option<HashSet<String>> {
    let ip_parts: Vec<&str> = ip.split('.').collect();
    
    for prefix_len in (0..=4).rev() {
        if prefix_len == 0 {
            if let Some(routes) = routing_table.get("0.0.0.0/0") {
                return Some(routes.clone());
            }
        } else {
            let prefix = format!("{}/{}", 
                ip_parts[..prefix_len].join("."),
                prefix_len * 8
            );
            
            if let Some(routes) = routing_table.get(&prefix) {
                return Some(routes.clone());
            }
        }
    }
    
    None
}
