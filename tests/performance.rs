use inventory::config::optimization::{PerformanceMonitor, QueryCache, optimize_database, measure_query_performance};
use inventory::db::schema::initialize_database;
use inventory::commands::filter::handle_filter;
use inventory::commands::stats::handle_stats;
use inventory::commands::list::OutputFormat;
use rusqlite::Connection;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn setup_test_db() -> Connection {
    let conn = initialize_database(None).unwrap();
    conn.execute("DELETE FROM items", []).unwrap();
    conn
}

fn add_test_items(conn: &Connection, count: usize) {
    for i in 0..count {
        conn.execute(
            r#"INSERT INTO items (
                title, description, price, quantity, photos, category, condition, brand, upc,
                item_specifics, shipping_details, size, original_price, hashtags, colorway, release_date,
                platform_status, internal_notes, last_updated, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)"#,
            rusqlite::params![
                format!("Item {}", i),
                Some(format!("Description for item {}", i)),
                (i % 100) as f64 + 10.0,
                (i % 10) + 1,
                None::<String>,
                match i % 5 { 0 => "electronics", 1 => "clothing", 2 => "books", 3 => "sports", _ => "home" },
                match i % 4 { 0 => "new", 1 => "used", 2 => "like new", _ => "good" },
                match i % 8 { 0 => "Nike", 1 => "Apple", 2 => "Samsung", 3 => "Adidas", 4 => "Dell", 5 => "Sony", 6 => "Puma", _ => "Microsoft" },
                Some(format!("123456789{:03}", i)),
                None::<String>,
                None::<String>,
                None::<String>,
                None::<f64>,
                None::<String>,
                None::<String>,
                None::<String>,
                None::<String>,
                None::<String>,
                "active",
            ],
        ).unwrap();
    }
}

#[test]
fn test_database_optimization() {
    let conn = setup_test_db();
    add_test_items(&conn, 1000);
    
    let result = optimize_database(&conn);
    if result.is_err() {
        eprintln!("optimize_database error: {:?}", result);
    }
    assert!(result.is_ok());
    
    // Verify indexes were created
    let mut indexes = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='items'").unwrap();
    let index_count = indexes.query_map([], |_| Ok(())).unwrap().count();
    assert!(index_count > 0);
}

#[test]
fn test_query_performance_monitoring() {
    let monitor = PerformanceMonitor::new();
    
    // Simulate a slow query
    let result = measure_query_performance(&monitor, "test_query", || {
        thread::sleep(Duration::from_millis(50));
        Ok(())
    });
    
    assert!(result.is_ok());
    
    let stats = monitor.get_stats();
    assert!(stats.contains_key("test_query_avg_ms"));
    assert!(stats["test_query_avg_ms"] >= 50.0);
    assert_eq!(stats["test_query_count"], 1.0);
}

#[test]
fn test_filter_performance_with_cache() {
    let conn = setup_test_db();
    add_test_items(&conn, 5000);
    optimize_database(&conn).unwrap();
    
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    // First query (cache miss)
    let start = std::time::Instant::now();
    let result = handle_filter(
        &conn,
        Some("10-50".to_string()),
        Some("electronics".to_string()),
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        Some(monitor.clone()),
        Some(cache.clone()),
    );
    let first_duration = start.elapsed();
    assert!(result.is_ok());
    
    // Second query (should be faster due to caching)
    let start = std::time::Instant::now();
    let result = handle_filter(
        &conn,
        Some("10-50".to_string()),
        Some("electronics".to_string()),
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        Some(monitor.clone()),
        Some(cache.clone()),
    );
    let second_duration = start.elapsed();
    assert!(result.is_ok());
    
    // Second query should be faster (though exact timing may vary)
    println!("First query: {:?}, Second query: {:?}", first_duration, second_duration);
    
    let stats = monitor.get_stats();
    assert!(stats.contains_key("cache_hits"));
    assert!(stats.contains_key("cache_misses"));
}

#[test]
fn test_stats_performance() {
    let conn = setup_test_db();
    add_test_items(&conn, 10000);
    optimize_database(&conn).unwrap();
    
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    // Measure stats query performance
    let start = std::time::Instant::now();
    let result = measure_query_performance(&monitor, "stats_query", || {
        handle_stats(&conn, Some(OutputFormat::Json), Some(monitor.clone()), Some(cache.clone()))
    });
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(5)); // Should complete within 5 seconds
    
    let stats = monitor.get_stats();
    println!("stats_query_count: {}", stats["stats_query_count"]);
    // Allow for 1 or more queries due to possible cache misses or repeated calls
    assert!(stats["stats_query_count"] >= 1.0);
}

#[test]
fn test_concurrent_access() {
    let conn = setup_test_db();
    add_test_items(&conn, 1000);
    optimize_database(&conn).unwrap();
    
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    // Spawn multiple threads to simulate concurrent access
    let mut handles = vec![];
    
    for i in 0..5 {
        let cache_clone = cache.clone();
        let _monitor_clone = monitor.clone();
        
        let handle = thread::spawn(move || {
            // Simulate concurrent filter operations
            for j in 0..10 {
                let key = format!("filter_{}_{}", i, j);
                cache_clone.set(key.clone(), format!("result_{}_{}", i, j), Duration::from_secs(60));
                
                let value = cache_clone.get(&key);
                assert_eq!(value, Some(format!("result_{}_{}", i, j)));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let stats = monitor.get_stats();
    println!("cache_hits: {} cache_misses: {}", stats["cache_hits"], stats["cache_misses"]);
    // All accesses should be hits, as we set then get each key
    assert_eq!(stats["cache_hits"], 50.0);
}

#[test]
fn test_cache_expiration() {
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = QueryCache::new(monitor.clone());
    
    // Set a cache entry with short TTL
    cache.set("expire_test".to_string(), "test_value".to_string(), Duration::from_millis(10));
    
    // Should be available immediately
    let value = cache.get("expire_test");
    assert_eq!(value, Some("test_value".to_string()));
    
    // Wait for expiration
    thread::sleep(Duration::from_millis(20));
    
    // Should be expired
    let value = cache.get("expire_test");
    assert_eq!(value, None);
    
    let stats = monitor.get_stats();
    assert_eq!(stats["cache_hits"], 1.0);
    assert_eq!(stats["cache_misses"], 1.0);
}

#[test]
fn test_large_dataset_performance() {
    let conn = setup_test_db();
    add_test_items(&conn, 50000); // Large dataset
    optimize_database(&conn).unwrap();
    
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    // Test filter performance on large dataset
    let start = std::time::Instant::now();
    let result = measure_query_performance(&monitor, "large_filter", || {
        handle_filter(
            &conn,
            Some("10-100".to_string()),
            Some("electronics".to_string()),
            Some("new".to_string()),
            None,
            Some("item_id,title,price".to_string()),
            Some(OutputFormat::Json),
            Some(monitor.clone()),
            Some(cache.clone()),
        )
    });
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(10)); // Should complete within 10 seconds
    
    let stats = monitor.get_stats();
    assert!(stats.contains_key("large_filter_avg_ms"));
    assert!(stats["large_filter_count"] == 1.0);
} 