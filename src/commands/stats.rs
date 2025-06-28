// Statistics command implementation
// Implements: inventory stats [--format {json,table}]

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::commands::list::OutputFormat;
use crate::config::optimization::{PerformanceMonitor, QueryCache, measure_query_performance, generate_cache_key};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryStats {
    pub total_items: i64,
    pub total_value: f64,
    pub average_price: f64,
    pub categories: Vec<CategoryStats>,
    pub conditions: Vec<ConditionStats>,
    pub brands: Vec<BrandStats>,
    pub price_ranges: PriceRangeStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: i64,
    pub total_value: f64,
    pub average_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionStats {
    pub condition: String,
    pub count: i64,
    pub total_value: f64,
    pub average_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrandStats {
    pub brand: String,
    pub count: i64,
    pub total_value: f64,
    pub average_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceRangeStats {
    pub under_10: i64,
    pub under_25: i64,
    pub under_50: i64,
    pub under_100: i64,
    pub under_250: i64,
    pub over_250: i64,
}

fn get_total_items(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?;
    Ok(count)
}

fn get_total_value(conn: &Connection) -> Result<f64> {
    let total: f64 = conn.query_row(
        "SELECT COALESCE(SUM(price * quantity), 0.0) FROM items",
        [],
        |row| row.get(0)
    )?;
    Ok(total)
}

fn get_average_price(conn: &Connection) -> Result<f64> {
    let avg: f64 = conn.query_row(
        "SELECT COALESCE(AVG(price), 0.0) FROM items",
        [],
        |row| row.get(0)
    )?;
    Ok(avg)
}

fn get_category_stats(conn: &Connection) -> Result<Vec<CategoryStats>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            category,
            COUNT(*) as count,
            COALESCE(SUM(price * quantity), 0.0) as total_value,
            COALESCE(AVG(price), 0.0) as average_price
        FROM items 
        GROUP BY category 
        ORDER BY count DESC
        "#
    )?;
    
    let stats = stmt.query_map([], |row| {
        Ok(CategoryStats {
            category: row.get(0)?,
            count: row.get(1)?,
            total_value: row.get(2)?,
            average_price: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, rusqlite::Error>>()
    .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    
    Ok(stats)
}

fn get_condition_stats(conn: &Connection) -> Result<Vec<ConditionStats>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            condition,
            COUNT(*) as count,
            COALESCE(SUM(price * quantity), 0.0) as total_value,
            COALESCE(AVG(price), 0.0) as average_price
        FROM items 
        GROUP BY condition 
        ORDER BY count DESC
        "#
    )?;
    
    let stats = stmt.query_map([], |row| {
        Ok(ConditionStats {
            condition: row.get(0)?,
            count: row.get(1)?,
            total_value: row.get(2)?,
            average_price: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, rusqlite::Error>>()
    .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    
    Ok(stats)
}

fn get_brand_stats(conn: &Connection) -> Result<Vec<BrandStats>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            COALESCE(brand, 'Unknown') as brand,
            COUNT(*) as count,
            COALESCE(SUM(price * quantity), 0.0) as total_value,
            COALESCE(AVG(price), 0.0) as average_price
        FROM items 
        GROUP BY brand 
        ORDER BY count DESC
        LIMIT 10
        "#
    )?;
    
    let stats = stmt.query_map([], |row| {
        Ok(BrandStats {
            brand: row.get(0)?,
            count: row.get(1)?,
            total_value: row.get(2)?,
            average_price: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, rusqlite::Error>>()
    .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    
    Ok(stats)
}

fn get_price_range_stats(conn: &Connection) -> Result<PriceRangeStats> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN price < 10 THEN 1 ELSE 0 END), 0) as under_10,
            COALESCE(SUM(CASE WHEN price < 25 THEN 1 ELSE 0 END), 0) as under_25,
            COALESCE(SUM(CASE WHEN price < 50 THEN 1 ELSE 0 END), 0) as under_50,
            COALESCE(SUM(CASE WHEN price < 100 THEN 1 ELSE 0 END), 0) as under_100,
            COALESCE(SUM(CASE WHEN price < 250 THEN 1 ELSE 0 END), 0) as under_250,
            COALESCE(SUM(CASE WHEN price >= 250 THEN 1 ELSE 0 END), 0) as over_250
        FROM items
        "#
    )?;
    
    let stats = stmt.query_row([], |row| {
        Ok(PriceRangeStats {
            under_10: row.get(0)?,
            under_25: row.get(1)?,
            under_50: row.get(2)?,
            under_100: row.get(3)?,
            under_250: row.get(4)?,
            over_250: row.get(5)?,
        })
    })?;
    
    Ok(stats)
}

fn format_table(stats: &InventoryStats) -> String {
    let mut output = String::new();
    
    // Overall statistics
    output.push_str("=== INVENTORY STATISTICS ===\n\n");
    output.push_str(&format!("Total Items: {}\n", stats.total_items));
    output.push_str(&format!("Total Value: ${:.2}\n", stats.total_value));
    output.push_str(&format!("Average Price: ${:.2}\n\n", stats.average_price));
    
    // Categories
    if !stats.categories.is_empty() {
        output.push_str("=== BY CATEGORY ===\n");
        output.push_str("Category          | Count | Total Value | Avg Price\n");
        output.push_str("------------------|-------|-------------|----------\n");
        for cat in &stats.categories {
            output.push_str(&format!("{:<16} | {:<5} | ${:<10.2} | ${:.2}\n", 
                cat.category, cat.count, cat.total_value, cat.average_price));
        }
        output.push_str("\n");
    }
    
    // Conditions
    if !stats.conditions.is_empty() {
        output.push_str("=== BY CONDITION ===\n");
        output.push_str("Condition         | Count | Total Value | Avg Price\n");
        output.push_str("------------------|-------|-------------|----------\n");
        for cond in &stats.conditions {
            output.push_str(&format!("{:<16} | {:<5} | ${:<10.2} | ${:.2}\n", 
                cond.condition, cond.count, cond.total_value, cond.average_price));
        }
        output.push_str("\n");
    }
    
    // Top Brands
    if !stats.brands.is_empty() {
        output.push_str("=== TOP BRANDS ===\n");
        output.push_str("Brand             | Count | Total Value | Avg Price\n");
        output.push_str("------------------|-------|-------------|----------\n");
        for brand in &stats.brands {
            output.push_str(&format!("{:<16} | {:<5} | ${:<10.2} | ${:.2}\n", 
                brand.brand, brand.count, brand.total_value, brand.average_price));
        }
        output.push_str("\n");
    }
    
    // Price Ranges
    output.push_str("=== PRICE RANGES ===\n");
    output.push_str("Range             | Count\n");
    output.push_str("------------------|-------\n");
    output.push_str(&format!("Under $10        | {}\n", stats.price_ranges.under_10));
    output.push_str(&format!("Under $25        | {}\n", stats.price_ranges.under_25));
    output.push_str(&format!("Under $50        | {}\n", stats.price_ranges.under_50));
    output.push_str(&format!("Under $100       | {}\n", stats.price_ranges.under_100));
    output.push_str(&format!("Under $250       | {}\n", stats.price_ranges.under_250));
    output.push_str(&format!("$250 and over    | {}\n", stats.price_ranges.over_250));
    
    output
}

pub fn handle_stats(
    conn: &Connection, 
    format: Option<OutputFormat>,
    monitor: Option<Arc<PerformanceMonitor>>,
    cache: Option<Arc<QueryCache>>,
) -> Result<()> {
    let format = format.unwrap_or(OutputFormat::Table);
    
    // Check cache first if available
    if let Some(cache) = &cache {
        let mut params = HashMap::new();
        params.insert("format".to_string(), format!("{:?}", format));
        
        let cache_key = generate_cache_key("stats", &params);
        
        if let Some(cached_result) = cache.get(&cache_key) {
            print!("{}", cached_result);
            return Ok(());
        }
    }
    
    // Gather all statistics with performance monitoring
    let stats = if let Some(monitor) = &monitor {
        measure_query_performance(monitor, "stats_query", || {
            Ok(InventoryStats {
                total_items: get_total_items(conn)?,
                total_value: get_total_value(conn)?,
                average_price: get_average_price(conn)?,
                categories: get_category_stats(conn)?,
                conditions: get_condition_stats(conn)?,
                brands: get_brand_stats(conn)?,
                price_ranges: get_price_range_stats(conn)?,
            })
        })?
    } else {
        InventoryStats {
            total_items: get_total_items(conn)?,
            total_value: get_total_value(conn)?,
            average_price: get_average_price(conn)?,
            categories: get_category_stats(conn)?,
            conditions: get_condition_stats(conn)?,
            brands: get_brand_stats(conn)?,
            price_ranges: get_price_range_stats(conn)?,
        }
    };
    
    // Format output
    let output = match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&stats)?
        }
        OutputFormat::Table => {
            format_table(&stats)
        }
        OutputFormat::Csv => {
            // For CSV, we'll output a simplified version with key metrics
            let mut csv = String::new();
            csv.push_str("metric,value\n");
            csv.push_str(&format!("total_items,{}\n", stats.total_items));
            csv.push_str(&format!("total_value,{:.2}\n", stats.total_value));
            csv.push_str(&format!("average_price,{:.2}\n", stats.average_price));
            csv.push_str(&format!("categories_count,{}\n", stats.categories.len()));
            csv.push_str(&format!("conditions_count,{}\n", stats.conditions.len()));
            csv.push_str(&format!("brands_count,{}\n", stats.brands.len()));
            csv
        }
    };
    
    // Cache the result if cache is available
    if let Some(cache) = &cache {
        let mut params = HashMap::new();
        params.insert("format".to_string(), format!("{:?}", format));
        
        let cache_key = generate_cache_key("stats", &params);
        cache.set(cache_key, output.clone(), Duration::from_secs(600)); // 10 minute TTL for stats
    }
    
    print!("{}", output);
    Ok(())
} 