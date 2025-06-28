use anyhow::{Result, anyhow};
use chrono::Utc;
use csv::ReaderBuilder;
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const SCHEMA_FIELDS: &[&str] = &[
    "item_id", "title", "description", "price", "quantity", "upc", "category", "condition", "brand"
];

fn failed_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".inventory/failed")
}

fn failed_path() -> PathBuf {
    let ts = Utc::now().format("%Y%m%dT%H%M%S");
    failed_dir().join(format!("validate_{}.json", ts))
}

fn validate_row(row: &csv::StringRecord, row_num: usize) -> Vec<serde_json::Value> {
    let mut errors = Vec::new();
    let get = |field| row.get(field).unwrap_or("");
    // Title: required, <= 80 chars
    let title = get(1);
    if title.is_empty() {
        errors.push(json!({"field": "title", "message": "Title is required", "row": row_num, "value": title}));
    } else if title.len() > 80 {
        errors.push(json!({"field": "title", "message": "Exceeds eBay's 80-character limit", "row": row_num, "value": title}));
    }
    // Price: required, non-negative
    let price = get(3);
    if price.is_empty() {
        errors.push(json!({"field": "price", "message": "Price is required", "row": row_num, "value": price}));
    } else if let Ok(p) = price.parse::<f64>() {
        if p < 0.0 {
            errors.push(json!({"field": "price", "message": "Price must be non-negative", "row": row_num, "value": price}));
        }
    } else {
        errors.push(json!({"field": "price", "message": "Invalid price value", "row": row_num, "value": price}));
    }
    // Quantity: required, non-negative integer
    let quantity = get(4);
    if quantity.is_empty() {
        errors.push(json!({"field": "quantity", "message": "Quantity is required", "row": row_num, "value": quantity}));
    } else if let Ok(q) = quantity.parse::<i32>() {
        if q < 0 {
            errors.push(json!({"field": "quantity", "message": "Quantity must be non-negative", "row": row_num, "value": quantity}));
        }
    } else {
        errors.push(json!({"field": "quantity", "message": "Invalid quantity value", "row": row_num, "value": quantity}));
    }
    // Category: required
    let category = get(6);
    if category.is_empty() {
        errors.push(json!({"field": "category", "message": "Category is required", "row": row_num, "value": category}));
    }
    // Condition: required
    let condition = get(7);
    if condition.is_empty() {
        errors.push(json!({"field": "condition", "message": "Condition is required", "row": row_num, "value": condition}));
    }
    // Platform-specific: StockX requires upc
    let upc = get(5);
    if category == "stockx" && upc.is_empty() {
        errors.push(json!({"field": "upc", "message": "UPC required for StockX", "row": row_num, "value": upc}));
    }
    errors
}

pub fn handle_validate(file: &str) -> Result<()> {
    let mut rdr = ReaderBuilder::new().flexible(true).from_path(file)
        .map_err(|e| anyhow!("Failed to open CSV: {}", e))?;
    let headers = rdr.headers()?.clone();
    // Check schema
    for (i, field) in SCHEMA_FIELDS.iter().enumerate() {
        if headers.get(i).unwrap_or("") != *field {
            return Err(anyhow!("CSV schema mismatch at column {}: expected '{}', found '{}'", i+1, field, headers.get(i).unwrap_or("")));
        }
    }
    let mut all_errors = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        let row = result?;
        let errors = validate_row(&row, i+2); // +2 for header and 1-based
        all_errors.extend(errors);
    }
    if !all_errors.is_empty() {
        fs::create_dir_all(failed_dir())?;
        let path = failed_path();
        let mut f = File::create(&path)?;
        let json = json!({"errors": all_errors});
        f.write_all(serde_json::to_string_pretty(&json)?.as_bytes())?;
        eprintln!("Validation failed. Errors saved to {}", path.display());
        return Err(anyhow!("Validation failed. See {}", path.display()));
    } else {
        println!("Validation successful. No errors found.");
    }
    Ok(())
} 