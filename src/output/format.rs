// Output formatting implementation
// TODO: Implement JSON, CSV, and table formatting 

use anyhow::Result;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryItem {
    pub item_id: i32,
    pub title: String,
    pub price: f64,
    pub quantity: i32,
    pub condition: String,
    pub category: String,
    pub brand: Option<String>,
}

impl InventoryItem {
    pub fn new(
        item_id: i32,
        title: String,
        price: f64,
        quantity: i32,
        condition: String,
        category: String,
        brand: Option<String>,
    ) -> Self {
        Self {
            item_id,
            title,
            price,
            quantity,
            condition,
            category,
            brand,
        }
    }
}

pub fn format_json(items: &[InventoryItem]) -> Result<String> {
    Ok(serde_json::to_string_pretty(items)?)
}

pub fn format_csv(items: &[InventoryItem]) -> Result<String> {
    let mut csv = String::new();
    
    // Header
    csv.push_str("item_id,title,price,quantity,condition,category,brand\n");
    
    // Data rows
    for item in items {
        csv.push_str(&format!(
            "{},{},{:.2},{},{},{},{}\n",
            item.item_id,
            escape_csv_field(&item.title),
            item.price,
            item.quantity,
            escape_csv_field(&item.condition),
            escape_csv_field(&item.category),
            item.brand.as_ref().map_or(String::new(), |b| escape_csv_field(b))
        ));
    }
    
    Ok(csv)
}

pub fn format_table(items: &[InventoryItem]) -> Result<String> {
    let mut table = String::new();
    
    // Header
    table.push_str("ID  | Title                                                | Price   | Qty | Condition | Category | Brand\n");
    table.push_str("----|------------------------------------------------------|---------|-----|-----------|----------|-------\n");
    
    // Data rows
    for item in items {
        let title = if item.title.len() > 50 {
            format!("{}...", &item.title[..47])
        } else {
            item.title.clone()
        };
        
        let brand = item.brand.as_deref().unwrap_or("");
        let brand_display = if brand.len() > 30 {
            format!("{}...", &brand[..27])
        } else {
            brand.to_string()
        };
        
        table.push_str(&format!(
            "{:<4} | {:<50} | ${:<7.2} | {:<3} | {:<9} | {:<8} | {}\n",
            item.item_id,
            title,
            item.price,
            item.quantity,
            item.condition,
            item.category,
            brand_display
        ));
    }
    
    Ok(table)
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace("\"", "\"\""))
    } else {
        field.to_string()
    }
}

pub fn format_items(items: &[InventoryItem], format: &str) -> Result<String> {
    match format.to_lowercase().as_str() {
        "json" => format_json(items),
        "csv" => format_csv(items),
        "table" => format_table(items),
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
} 