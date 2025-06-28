use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use chrono::{Utc, DateTime};
use std::fs;
use std::io::Write;
use std::path::Path;
use serde_json;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: String,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
}

#[derive(Debug)]
pub struct PerformanceMonitor {
    pub query_times: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    pub cache_hits: Arc<Mutex<u64>>,
    pub cache_misses: Arc<Mutex<u64>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            query_times: Arc::new(Mutex::new(HashMap::new())),
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        }
    }

    pub fn record_query(&self, query_name: &str, duration: Duration) {
        if let Ok(mut times) = self.query_times.lock() {
            times.entry(query_name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    pub fn record_cache_hit(&self) {
        if let Ok(mut hits) = self.cache_hits.lock() {
            *hits += 1;
        }
    }

    pub fn record_cache_miss(&self) {
        if let Ok(mut misses) = self.cache_misses.lock() {
            *misses += 1;
        }
    }

    pub fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if let Ok(times) = self.query_times.lock() {
            for (query, durations) in times.iter() {
                if !durations.is_empty() {
                    let avg = durations.iter().map(|d| d.as_millis() as f64).sum::<f64>() / durations.len() as f64;
                    stats.insert(format!("{}_avg_ms", query), avg);
                    stats.insert(format!("{}_count", query), durations.len() as f64);
                }
            }
        }

        if let Ok(hits) = self.cache_hits.lock() {
            stats.insert("cache_hits".to_string(), *hits as f64);
        }

        if let Ok(misses) = self.cache_misses.lock() {
            stats.insert("cache_misses".to_string(), *misses as f64);
        }

        stats
    }

    /// Write performance stats to a file in the 'logs' directory as JSON
    pub fn write_performance_report(&self, file_name: &str) -> Result<()> {
        let stats = self.get_stats();
        let logs_dir = Path::new("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(logs_dir)?;
        }
        let file_path = logs_dir.join(file_name);
        let json = serde_json::to_string_pretty(&stats)?;
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct QueryCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    monitor: Arc<PerformanceMonitor>,
}

impl QueryCache {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            monitor,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(key) {
                if Utc::now() < entry.timestamp + chrono::Duration::from_std(entry.ttl).unwrap() {
                    self.monitor.record_cache_hit();
                    return Some(entry.data.clone());
                } else {
                    // Expired entry, remove it
                    drop(cache);
                    if let Ok(mut cache) = self.cache.lock() {
                        cache.remove(key);
                    }
                }
            }
        }
        self.monitor.record_cache_miss();
        None
    }

    pub fn set(&self, key: String, data: String, ttl: Duration) {
        let entry = CacheEntry {
            data,
            timestamp: Utc::now(),
            ttl,
        };
        
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, entry);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

pub fn create_database_indexes(conn: &Connection) -> Result<()> {
    // Create indexes for frequently queried fields
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_items_price ON items(price)",
        "CREATE INDEX IF NOT EXISTS idx_items_category ON items(category)",
        "CREATE INDEX IF NOT EXISTS idx_items_condition ON items(condition)",
        "CREATE INDEX IF NOT EXISTS idx_items_brand ON items(brand)",
        "CREATE INDEX IF NOT EXISTS idx_items_status ON items(status)",
        "CREATE INDEX IF NOT EXISTS idx_items_price_category ON items(price, category)",
        "CREATE INDEX IF NOT EXISTS idx_items_category_condition ON items(category, condition)",
    ];

    for index_sql in &indexes {
        conn.execute(index_sql, [])?;
    }

    Ok(())
}

pub fn optimize_database(conn: &Connection) -> Result<()> {
    // Create indexes
    create_database_indexes(conn)?;
    
    // Use execute_batch for ANALYZE and PRAGMA statements
    conn.execute_batch(
        "ANALYZE;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = 10000;
        PRAGMA temp_store = MEMORY;"
    )?;
    
    Ok(())
}

pub fn measure_query_performance<F, T>(monitor: &PerformanceMonitor, query_name: &str, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let start = Instant::now();
    let result = f()?;
    let duration = start.elapsed();
    
    monitor.record_query(query_name, duration);
    
    Ok(result)
}

pub fn generate_cache_key(operation: &str, params: &HashMap<String, String>) -> String {
    let mut key = operation.to_string();
    let mut sorted_params: Vec<_> = params.iter().collect();
    sorted_params.sort_by(|a, b| a.0.cmp(b.0));
    
    for (k, v) in sorted_params {
        key.push_str(&format!("_{}={}", k, v));
    }
    
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_database;

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        // Record some query times
        monitor.record_query("test_query", Duration::from_millis(100));
        monitor.record_query("test_query", Duration::from_millis(200));
        
        let stats = monitor.get_stats();
        assert!(stats.contains_key("test_query_avg_ms"));
        assert_eq!(stats["test_query_count"], 2.0);
    }

    #[test]
    fn test_query_cache() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let cache = QueryCache::new(monitor.clone());
        
        // Test cache set/get
        cache.set("test_key".to_string(), "test_value".to_string(), Duration::from_secs(60));
        
        let value = cache.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
        
        // Test cache miss
        let value = cache.get("nonexistent_key");
        assert_eq!(value, None);
    }

    #[test]
    fn test_database_indexes() {
        let conn = initialize_database(None).unwrap();
        let result = create_database_indexes(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_key_generation() {
        let mut params = HashMap::new();
        params.insert("price".to_string(), "10-50".to_string());
        params.insert("category".to_string(), "electronics".to_string());
        
        let key = generate_cache_key("filter", &params);
        assert!(key.contains("filter"));
        assert!(key.contains("price=10-50"));
        assert!(key.contains("category=electronics"));
    }
} 