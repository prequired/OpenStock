// Import command implementation
// Implements: inventory import --file items.csv

use anyhow::{Result, Context};
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::Utc;
use csv::ReaderBuilder;
use dirs::home_dir;
use serde_json;
use crate::db::queries;
use crate::validation::{validate_item_ebay, ValidationResult, ValidationError};
use rusqlite::Connection;

/// Fixed CSV schema as per specification
const REQUIRED_FIELDS: [&str; 9] = [
    "item_id", "title", "description", "price", "quantity", 
    "upc", "category", "condition", "brand"
];

fn failed_imports_path() -> PathBuf {
    let mut dir = home_dir().expect("Could not determine home directory");
    dir.push(".inventory/failed");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S");
    dir.push(format!("failed_import_{}.json", timestamp));
    dir
}

/// Validate CSV headers against fixed schema
fn validate_headers(headers: &csv::StringRecord) -> Result<()> {
    let header_fields: Vec<&str> = headers.iter().collect();
    
    for required_field in &REQUIRED_FIELDS {
        if !header_fields.contains(required_field) {
            anyhow::bail!("Missing required field: {}", required_field);
        }
    }
    
    Ok(())
}

/// Interactive prompt for correcting invalid data
fn prompt_for_correction(field: &str, current_value: &str, error_message: &str, row: usize) -> Result<Option<String>> {
    println!("\nRow {}: Invalid {} - {}", row, field, error_message);
    println!("Current value: '{}'", current_value);
    print!("Enter new value (or press Enter to skip this row): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();
    
    if input.is_empty() {
        Ok(None) // Skip row
    } else {
        Ok(Some(input))
    }
}

/// Process a single CSV row with interactive correction or non-interactive skip
fn process_row(
    record: &csv::StringRecord,
    headers: &csv::StringRecord,
    row_idx: usize,
    conn: &Connection,
    non_interactive: bool,
) -> Result<bool> {
    // Extract fields from CSV
    let get_field = |field: &str| -> String {
        headers.iter()
            .position(|h| h == field)
            .and_then(|pos| record.get(pos))
            .unwrap_or("")
            .to_string()
    };
    
    let mut title = get_field("title");
    let description = get_field("description");
    let mut price_str = get_field("price");
    let mut quantity_str = get_field("quantity");
    let mut upc = get_field("upc");
    let mut category = get_field("category");
    let mut condition = get_field("condition");
    let mut brand = get_field("brand");
    
    // Parse numeric fields
    let mut price = price_str.parse::<f64>().unwrap_or(-1.0);
    let mut quantity = quantity_str.parse::<i32>().unwrap_or(-1);
    
    // Validate the row
    let mut validation = validate_item_ebay(
        &title,
        price,
        quantity,
        &category,
        &condition,
        if brand.is_empty() { None } else { Some(&brand) },
        if upc.is_empty() { None } else { Some(&upc) },
    )?;
    
    // Add row information to errors
    for err in &mut validation.errors {
        err.row = Some(row_idx + 1);
        err.value = match err.field.as_str() {
            "title" => Some(title.clone()),
            "price" => Some(price_str.clone()),
            "quantity" => Some(quantity_str.clone()),
            "category" => Some(category.clone()),
            "condition" => Some(condition.clone()),
            "brand" => Some(brand.clone()),
            "upc" => Some(upc.clone()),
            _ => None,
        };
    }
    
    // If validation fails, prompt for corrections (unless non_interactive)
    if !validation.is_valid() {
        if non_interactive {
            // In non-interactive mode, always skip invalid rows
            return Ok(false);
        }
        println!("\n=== Row {} has validation errors ===", row_idx + 1);
        
        // Show all errors first
        for err in &validation.errors {
            println!("- {}: {} (value: '{}')", 
                err.field, err.message, err.value.as_deref().unwrap_or(""));
        }
        
        // Prompt for corrections for each error
        for err in &validation.errors {
            let current_value = err.value.as_deref().unwrap_or("");
            let correction = prompt_for_correction(&err.field, current_value, &err.message, row_idx + 1)?;
            
            if let Some(new_value) = correction {
                // Update the corresponding field
                match err.field.as_str() {
                    "title" => title = new_value,
                    "price" => {
                        price = new_value.parse().unwrap_or(price);
                    },
                    "quantity" => {
                        quantity = new_value.parse().unwrap_or(quantity);
                    },
                    "category" => category = new_value,
                    "condition" => condition = new_value,
                    "brand" => brand = new_value,
                    "upc" => upc = new_value,
                    _ => {}
                }
            } else {
                // User chose to skip this row
                return Ok(false);
            }
        }
        
        // Re-validate after corrections
        let revalidation = validate_item_ebay(
            &title,
            price,
            quantity,
            &category,
            &condition,
            if brand.is_empty() { None } else { Some(&brand) },
            if upc.is_empty() { None } else { Some(&upc) },
        )?;
        
        if !revalidation.is_valid() {
            println!("Row {} still has validation errors after correction. Skipping.", row_idx + 1);
            return Ok(false);
        }
    }
    
    // Insert into database
    let item = queries::NewItem {
        title: &title,
        description: if description.is_empty() { None } else { Some(&description) },
        price,
        quantity,
        photos: None,
        category: &category,
        condition: &condition,
        brand: if brand.is_empty() { None } else { Some(&brand) },
        upc: if upc.is_empty() { None } else { Some(&upc) },
        item_specifics: None,
        shipping_details: None,
        size: None,
        original_price: None,
        hashtags: None,
        colorway: None,
        release_date: None,
        platform_status: None,
        internal_notes: None,
        status: "active",
    };
    
    match queries::insert_item(conn, &item) {
        Ok(_) => {
            println!("✓ Row {} imported successfully", row_idx + 1);
            Ok(true)
        },
        Err(e) => {
            println!("✗ Row {} database error: {}", row_idx + 1, e);
            Ok(false)
        }
    }
}

pub fn handle_import(file: String, conn: &Connection, non_interactive: bool) -> Result<()> {
    let file_path = PathBuf::from(&file);
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file);
    }
    
    println!("Importing from: {}", file);
    
    // Read CSV file
    let mut rdr = ReaderBuilder::new()
        .flexible(true)
        .from_path(&file_path)
        .with_context(|| format!("Failed to open CSV file: {}", file))?;
    
    // Validate headers
    let headers = rdr.headers()?.clone();
    validate_headers(&headers)?;
    
    println!("CSV schema validated. Starting import...");
    
    let mut failed_rows: Vec<ValidationError> = Vec::new();
    let mut imported = 0;
    let mut skipped = 0;
    
    // Process each row
    for (row_idx, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                let error = ValidationError {
                    field: "csv_parse".to_string(),
                    message: format!("CSV parse error: {}", e),
                    row: Some(row_idx + 1),
                    value: None,
                };
                failed_rows.push(error);
                println!("✗ Row {}: CSV parse error - {}", row_idx + 1, e);
                skipped += 1;
                continue;
            }
        };
        
        match process_row(&record, &headers, row_idx, conn, non_interactive) {
            Ok(true) => imported += 1,
            Ok(false) => {
                // Add validation errors for this row
                let validation = validate_item_ebay(
                    record.get(headers.iter().position(|h| h == "title").unwrap_or(usize::MAX)).unwrap_or(""),
                    record.get(headers.iter().position(|h| h == "price").unwrap_or(usize::MAX)).unwrap_or("").parse().unwrap_or(-1.0),
                    record.get(headers.iter().position(|h| h == "quantity").unwrap_or(usize::MAX)).unwrap_or("").parse().unwrap_or(-1),
                    record.get(headers.iter().position(|h| h == "category").unwrap_or(usize::MAX)).unwrap_or(""),
                    record.get(headers.iter().position(|h| h == "condition").unwrap_or(usize::MAX)).unwrap_or(""),
                    record.get(headers.iter().position(|h| h == "brand").unwrap_or(usize::MAX)),
                    record.get(headers.iter().position(|h| h == "upc").unwrap_or(usize::MAX)),
                )?;
                
                for mut err in validation.errors {
                    err.row = Some(row_idx + 1);
                    failed_rows.push(err);
                }
                skipped += 1;
            }
            Err(e) => {
                println!("✗ Row {}: Processing error - {}", row_idx + 1, e);
                skipped += 1;
            }
        }
    }
    
    // Save failed rows if any
    if !failed_rows.is_empty() {
        let path = failed_imports_path();
        let json = serde_json::to_string_pretty(&ValidationResult { errors: failed_rows })?;
        std::fs::write(&path, json)?;
        println!("\nFailed rows saved to: {}", path.display());
    }
    
    println!("\n=== Import Summary ===");
    println!("Successfully imported: {} items", imported);
    println!("Skipped/Failed: {} items", skipped);
    println!("Total processed: {} rows", imported + skipped);
    
    Ok(())
} 