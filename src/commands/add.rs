// Add item command implementation
// TODO: Implement add-item subcommand 

use anyhow::Result;
use rusqlite::Connection;
use crate::db::schema::initialize_database;
use crate::db::queries::{insert_item, NewItem};
use crate::validation::{validate_item_ebay, validate_item_stockx, validate_item_poshmark, validate_item_mercari};

pub fn handle_add_item(
    title: String,
    price: f64,
    quantity: i32,
    category: String,
    condition: String,
    brand: Option<String>,
    description: Option<String>,
    upc: Option<String>,
    size: Option<String>,
    original_price: Option<f64>,
    hashtags: Option<String>,
    colorway: Option<String>,
    release_date: Option<String>,
    internal_notes: Option<String>,
) -> Result<()> {
    // Initialize database connection
    let conn = initialize_database(None)?;
    
    // Validate the item for all platforms (we'll use eBay as default for now)
    let validation_result = validate_item_ebay(
        &title,
        price,
        quantity,
        &category,
        &condition,
        brand.as_deref(),
        upc.as_deref(),
    )?;
    
    // If validation fails, output JSON errors and return
    if !validation_result.is_valid() {
        let error_json = validation_result.to_json()?;
        eprintln!("{}", error_json);
        return Ok(());
    }
    
    // Create the item for database insertion
    let item = NewItem {
        title: &title,
        description: description.as_deref(),
        price,
        quantity,
        photos: None, // TODO: Add photo handling
        category: &category,
        condition: &condition,
        brand: brand.as_deref(),
        upc: upc.as_deref(),
        item_specifics: None, // TODO: Add item specifics handling
        shipping_details: None, // TODO: Add shipping details handling
        size: size.as_deref(),
        original_price,
        hashtags: hashtags.as_deref(),
        colorway: colorway.as_deref(),
        release_date: release_date.as_deref(),
        platform_status: None, // TODO: Add platform status handling
        internal_notes: internal_notes.as_deref(),
        status: "active",
    };
    
    // Insert the item into the database
    let rows_affected = insert_item(&conn, &item)?;
    
    if rows_affected == 1 {
        println!("Successfully added item: {} (${:.2}, qty: {})", title, price, quantity);
        println!("Category: {}, Condition: {}", category, condition);
        
        if let Some(brand_name) = brand {
            println!("Brand: {}", brand_name);
        }
        
        if let Some(desc) = description {
            println!("Description: {}", desc);
        }
        
        if let Some(upc_code) = upc {
            println!("UPC: {}", upc_code);
        }
        
        if let Some(size_value) = size {
            println!("Size: {}", size_value);
        }
    } else {
        eprintln!("Error: Failed to insert item into database");
    }
    
    Ok(())
} 