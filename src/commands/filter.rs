// Filter command implementation
// Implements: inventory filter --price 10-50 --category clothing --condition new --brand nike -f id,title,price --format json

use anyhow::{Result, anyhow};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::commands::list::OutputFormat;
use crate::config::optimization::{PerformanceMonitor, QueryCache, measure_query_performance, generate_cache_key};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteredItem {
    pub item_id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub quantity: Option<i32>,
    pub category: Option<String>,
    pub condition: Option<String>,
    pub brand: Option<String>,
    pub upc: Option<String>,
}

#[derive(Debug)]
pub struct PriceRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl PriceRange {
    fn parse(price_str: &str) -> Result<Self> {
        if price_str.contains('-') {
            let parts: Vec<&str> = price_str.split('-').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid price range format. Use 'min-max' or 'min-' or '-max'"));
            }
            
            let min = if parts[0].is_empty() {
                None
            } else {
                Some(parts[0].parse::<f64>().map_err(|_| anyhow!("Invalid minimum price"))?)
            };
            
            let max = if parts[1].is_empty() {
                None
            } else {
                Some(parts[1].parse::<f64>().map_err(|_| anyhow!("Invalid maximum price"))?)
            };
            
            Ok(PriceRange { min, max })
        } else {
            // Single price value
            let price = price_str.parse::<f64>().map_err(|_| anyhow!("Invalid price value"))?;
            Ok(PriceRange { min: Some(price), max: Some(price) })
        }
    }
}

fn get_field_shortcuts() -> std::collections::HashMap<&'static str, &'static str> {
    let mut shortcuts = std::collections::HashMap::new();
    shortcuts.insert("id", "item_id");
    shortcuts.insert("t", "title");
    shortcuts.insert("d", "description");
    shortcuts.insert("p", "price");
    shortcuts.insert("q", "quantity");
    shortcuts.insert("c", "condition");
    shortcuts.insert("cat", "category");
    shortcuts.insert("b", "brand");
    shortcuts.insert("u", "upc");
    shortcuts
}

fn expand_field_shortcuts(fields: &str) -> Result<Vec<String>> {
    let shortcuts = get_field_shortcuts();
    let field_list: Vec<&str> = fields.split(',').map(|s| s.trim()).collect();
    let mut expanded_fields = Vec::new();
    
    for field in field_list {
        if field.is_empty() {
            continue;
        }
        
        let expanded = if shortcuts.contains_key(field) {
            shortcuts[field].to_string()
        } else {
            field.to_string()
        };
        
        expanded_fields.push(expanded);
    }
    
    if expanded_fields.is_empty() {
        return Err(anyhow!("No valid fields specified"));
    }
    
    Ok(expanded_fields)
}

fn validate_fields(fields: &[String]) -> Result<()> {
    let valid_fields = [
        "item_id", "title", "description", "price", "quantity", 
        "category", "condition", "brand", "upc"
    ];
    
    for field in fields {
        if !valid_fields.contains(&field.as_str()) {
            return Err(anyhow!("Unknown field: {}", field));
        }
    }
    
    Ok(())
}

fn build_filter_query(
    price_range: Option<&PriceRange>,
    category: Option<&str>,
    condition: Option<&str>,
    brand: Option<&str>,
    fields: &[String],
) -> Result<(String, Vec<rusqlite::types::Value>)> {
    let mut conditions = Vec::new();
    let mut params = Vec::new();
    
    // Price range filter
    if let Some(range) = price_range {
        if let Some(min) = range.min {
            conditions.push("price >= ?");
            params.push(rusqlite::types::Value::Real(min));
        }
        if let Some(max) = range.max {
            conditions.push("price <= ?");
            params.push(rusqlite::types::Value::Real(max));
        }
    }
    
    // Category filter
    if let Some(cat) = category {
        conditions.push("category = ?");
        params.push(rusqlite::types::Value::Text(cat.to_string()));
    }
    
    // Condition filter
    if let Some(cond) = condition {
        conditions.push("condition = ?");
        params.push(rusqlite::types::Value::Text(cond.to_string()));
    }
    
    // Brand filter
    if let Some(brand_name) = brand {
        conditions.push("brand = ?");
        params.push(rusqlite::types::Value::Text(brand_name.to_string()));
    }
    
    // Build SELECT clause
    let select_clause = fields.join(", ");
    
    // Build WHERE clause
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    
    let query = format!("SELECT {} FROM items {}", select_clause, where_clause);
    
    Ok((query, params))
}

fn execute_filter_query(
    conn: &Connection,
    query: &str,
    params: &[rusqlite::types::Value],
) -> Result<Vec<FilteredItem>> {
    let mut stmt = conn.prepare(query)?;
    
    let items = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        let mut item = FilteredItem {
            item_id: None,
            title: None,
            description: None,
            price: None,
            quantity: None,
            category: None,
            condition: None,
            brand: None,
            upc: None,
        };
        
        // Map fields based on their position in the SELECT clause
        let mut col_idx = 0;
        
        for field in query.split("SELECT ").nth(1).unwrap().split(" FROM").next().unwrap().split(", ") {
            match field.trim() {
                "item_id" => item.item_id = row.get(col_idx).ok(),
                "title" => item.title = row.get(col_idx).ok(),
                "description" => item.description = row.get(col_idx).ok(),
                "price" => item.price = row.get(col_idx).ok(),
                "quantity" => item.quantity = row.get(col_idx).ok(),
                "category" => item.category = row.get(col_idx).ok(),
                "condition" => item.condition = row.get(col_idx).ok(),
                "brand" => item.brand = row.get(col_idx).ok(),
                "upc" => item.upc = row.get(col_idx).ok(),
                _ => {}
            }
            col_idx += 1;
        }
        
        Ok(item)
    })?
    .collect::<Result<Vec<_>, rusqlite::Error>>()
    .map_err(|e| anyhow!("Database error: {}", e))?;
    
    Ok(items)
}

fn format_filtered_items_table(items: &[FilteredItem], fields: &[String]) -> String {
    if items.is_empty() {
        return "No items found matching the filter criteria.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Header
    let header: Vec<String> = fields.iter().map(|f| f.to_uppercase()).collect();
    output.push_str(&header.join(" | "));
    output.push('\n');
    
    // Separator
    let separator: Vec<String> = fields.iter().map(|_| "---".to_string()).collect();
    output.push_str(&separator.join(" | "));
    output.push('\n');
    
    // Rows
    for item in items {
        let mut row = Vec::new();
        
        for field in fields {
            let value = match field.as_str() {
                "item_id" => item.item_id.map(|v| v.to_string()).unwrap_or_default(),
                "title" => item.title.clone().unwrap_or_default(),
                "description" => item.description.clone().unwrap_or_default(),
                "price" => item.price.map(|v| format!("${:.2}", v)).unwrap_or_default(),
                "quantity" => item.quantity.map(|v| v.to_string()).unwrap_or_default(),
                "category" => item.category.clone().unwrap_or_default(),
                "condition" => item.condition.clone().unwrap_or_default(),
                "brand" => item.brand.clone().unwrap_or_default(),
                "upc" => item.upc.clone().unwrap_or_default(),
                _ => String::new(),
            };
            
            // Truncate long values
            let truncated = if value.len() > 30 {
                format!("{}...", &value[..27])
            } else {
                value
            };
            
            row.push(truncated);
        }
        
        output.push_str(&row.join(" | "));
        output.push('\n');
    }
    
    output
}

fn format_filtered_items_csv(items: &[FilteredItem], fields: &[String]) -> String {
    if items.is_empty() {
        return "No items found matching the filter criteria.\n".to_string();
    }
    
    let mut output = String::new();
    
    // Header
    let header: Vec<String> = fields.iter().map(|f| f.to_uppercase()).collect();
    output.push_str(&header.join(","));
    output.push('\n');
    
    // Rows
    for item in items {
        let mut row = Vec::new();
        
        for field in fields {
            let value = match field.as_str() {
                "item_id" => item.item_id.map(|v| v.to_string()).unwrap_or_default(),
                "title" => item.title.clone().unwrap_or_default(),
                "description" => item.description.clone().unwrap_or_default(),
                "price" => item.price.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                "quantity" => item.quantity.map(|v| v.to_string()).unwrap_or_default(),
                "category" => item.category.clone().unwrap_or_default(),
                "condition" => item.condition.clone().unwrap_or_default(),
                "brand" => item.brand.clone().unwrap_or_default(),
                "upc" => item.upc.clone().unwrap_or_default(),
                _ => String::new(),
            };
            
            // Escape CSV values
            let escaped = if value.contains(',') || value.contains('"') || value.contains('\n') {
                format!("\"{}\"", value.replace("\"", "\"\""))
            } else {
                value
            };
            
            row.push(escaped);
        }
        
        output.push_str(&row.join(","));
        output.push('\n');
    }
    
    output
}

pub fn handle_filter(
    conn: &Connection,
    price: Option<String>,
    category: Option<String>,
    condition: Option<String>,
    brand: Option<String>,
    fields: Option<String>,
    format: Option<OutputFormat>,
    monitor: Option<Arc<PerformanceMonitor>>,
    cache: Option<Arc<QueryCache>>,
) -> Result<()> {
    let format = format.unwrap_or(OutputFormat::Json);
    
    // Parse price range
    let price_range = if let Some(ref price_str) = price {
        Some(PriceRange::parse(price_str)?)
    } else {
        None
    };
    
    // Parse and validate fields
    let fields_str = fields.unwrap_or_else(|| "item_id,title,price,quantity,category,condition,brand".to_string());
    let expanded_fields = expand_field_shortcuts(&fields_str)?;
    validate_fields(&expanded_fields)?;
    
    // Check cache first if available
    if let Some(cache) = &cache {
        let mut params = HashMap::new();
        if let Some(price_str) = &price { params.insert("price".to_string(), price_str.clone()); }
        if let Some(cat) = &category { params.insert("category".to_string(), cat.clone()); }
        if let Some(cond) = &condition { params.insert("condition".to_string(), cond.clone()); }
        if let Some(brand_name) = &brand { params.insert("brand".to_string(), brand_name.clone()); }
        params.insert("fields".to_string(), fields_str.clone());
        params.insert("format".to_string(), format!("{:?}", format));
        
        let cache_key = generate_cache_key("filter", &params);
        
        if let Some(cached_result) = cache.get(&cache_key) {
            print!("{}", cached_result);
            return Ok(());
        }
    }
    
    // Build and execute query with performance monitoring
    let items = if let Some(monitor) = &monitor {
        measure_query_performance(monitor, "filter_query", || {
            let (query, params) = build_filter_query(
                price_range.as_ref(),
                category.as_deref(),
                condition.as_deref(),
                brand.as_deref(),
                &expanded_fields,
            )?;
            
            execute_filter_query(conn, &query, &params)
        })?
    } else {
        let (query, params) = build_filter_query(
            price_range.as_ref(),
            category.as_deref(),
            condition.as_deref(),
            brand.as_deref(),
            &expanded_fields,
        )?;
        
        execute_filter_query(conn, &query, &params)?
    };
    
    // Format output
    let output = match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&items)?
        }
        OutputFormat::Table => {
            format_filtered_items_table(&items, &expanded_fields)
        }
        OutputFormat::Csv => {
            format_filtered_items_csv(&items, &expanded_fields)
        }
    };
    
    // Cache the result if cache is available
    if let Some(cache) = &cache {
        let mut params = HashMap::new();
        if let Some(price_str) = &price { params.insert("price".to_string(), price_str.clone()); }
        if let Some(cat) = &category { params.insert("category".to_string(), cat.clone()); }
        if let Some(cond) = &condition { params.insert("condition".to_string(), cond.clone()); }
        if let Some(brand_name) = &brand { params.insert("brand".to_string(), brand_name.clone()); }
        params.insert("fields".to_string(), fields_str);
        params.insert("format".to_string(), format!("{:?}", format));
        
        let cache_key = generate_cache_key("filter", &params);
        cache.set(cache_key, output.clone(), Duration::from_secs(300)); // 5 minute TTL
    }
    
    print!("{}", output);
    Ok(())
} 