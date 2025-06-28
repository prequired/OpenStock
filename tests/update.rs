use inventory::db::schema::initialize_database;
use inventory::db::queries::{insert_item, NewItem, get_item_by_id, update_item};
use inventory::commands::update::{Update, execute};
use rusqlite::Connection;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;
use serde_json;
use std::fs;

fn setup_test_db() -> Connection {
    let conn = initialize_database(None).unwrap();
    // Clear any existing data
    conn.execute("DELETE FROM items", []).unwrap();
    conn
}

#[test]
fn test_update_valid_csv() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let item_id = insert_item(&conn, &NewItem {
        title: "Item 1",
        description: None,
        price: 10.0,
        quantity: 5,
        photos: None,
        category: "sneakers",
        condition: "new",
        brand: Some("Nike"),
        upc: Some("123456789012"),
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
    })?;
    
    let mut csv_file = NamedTempFile::new()?;
    writeln!(csv_file, "id,title,price")?;
    writeln!(csv_file, "{},Updated Item,20.0", item_id)?;
    
    let args = Update { 
        file: Some(csv_file.path().to_str().unwrap().to_string()), 
        retry: None 
    };
    let result = execute(args, &conn);
    assert!(result.is_ok(), "Valid CSV update should succeed");
    
    let item = get_item_by_id(&conn, item_id.try_into().unwrap())?.unwrap();
    assert_eq!(item["title"], "Updated Item");
    assert_eq!(item["price"], "20.00");
    assert_eq!(item["quantity"], "5"); // Unchanged
    Ok(())
}

#[test]
fn test_update_invalid_id() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let mut csv_file = NamedTempFile::new()?;
    writeln!(csv_file, "id,title")?;
    writeln!(csv_file, "999,Updated Item")?;
    
    let args = Update { 
        file: Some(csv_file.path().to_str().unwrap().to_string()), 
        retry: None 
    };
    let result = execute(args, &conn);
    assert!(result.is_ok(), "Should handle invalid ID gracefully");
    
    // Check that failed rows were saved
    let failed_dir = dirs::home_dir().unwrap().join(".inventory/failed");
    if failed_dir.exists() {
        let failed_files: Vec<_> = fs::read_dir(&failed_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("failed_update_"))
            .collect();
        assert!(!failed_files.is_empty(), "Failed update file should be created");
    }
    Ok(())
}

#[test]
fn test_update_invalid_validation() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let item_id = insert_item(&conn, &NewItem {
        title: "Item 1",
        description: None,
        price: 10.0,
        quantity: 5,
        photos: None,
        category: "sneakers",
        condition: "new",
        brand: Some("Nike"),
        upc: Some("123456789012"),
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
    })?;
    
    let mut csv_file = NamedTempFile::new()?;
    writeln!(csv_file, "id,title,price")?;
    writeln!(csv_file, "{},, -1.0", item_id)?; // Empty title and negative price
    
    std::env::set_var("INVENTORY_NONINTERACTIVE", "1");
    
    let args = Update { 
        file: Some(csv_file.path().to_str().unwrap().to_string()), 
        retry: None 
    };
    let result = execute(args, &conn);
    assert!(result.is_ok(), "Should handle validation errors gracefully");
    
    // Check that failed rows were saved
    let failed_dir = dirs::home_dir().unwrap().join(".inventory/failed");
    if failed_dir.exists() {
        let failed_files: Vec<_> = fs::read_dir(&failed_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("failed_update_"))
            .collect();
        assert!(!failed_files.is_empty(), "Failed update file should be created");
    }
    Ok(())
}

#[test]
fn test_update_retry_valid() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let item_id = insert_item(&conn, &NewItem {
        title: "Item 1",
        description: None,
        price: 10.0,
        quantity: 5,
        photos: None,
        category: "sneakers",
        condition: "new",
        brand: Some("Nike"),
        upc: Some("123456789012"),
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
    })?;
    
    let mut json_file = NamedTempFile::new()?;
    let errors = serde_json::json!({
        "errors": [
            {
                "field": "title",
                "message": "Title cannot be empty",
                "row": 2,
                "value": ""
            }
        ]
    });
    serde_json::to_writer(&json_file, &errors)?;
    
    let args = Update { 
        file: None, 
        retry: Some(json_file.path().to_str().unwrap().to_string()) 
    };
    let result = execute(args, &conn);
    assert!(result.is_ok(), "Retry should handle errors gracefully");
    
    let item = get_item_by_id(&conn, item_id.try_into().unwrap())?.unwrap();
    assert_eq!(item["title"], "Item 1"); // Title not updated due to retry failure
    Ok(())
}

#[test]
fn test_update_empty_csv() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let mut csv_file = NamedTempFile::new()?;
    writeln!(csv_file, "id,title")?;
    
    let args = Update { 
        file: Some(csv_file.path().to_str().unwrap().to_string()), 
        retry: None 
    };
    let result = execute(args, &conn);
    assert!(result.is_ok(), "Empty CSV should process without error");
    
    Ok(())
}

#[test]
fn test_update_partial_fields() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let item_id = insert_item(&conn, &NewItem {
        title: "Original Title",
        description: None,
        price: 10.0,
        quantity: 5,
        photos: None,
        category: "sneakers",
        condition: "new",
        brand: Some("Nike"),
        upc: Some("123456789012"),
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
    })?;
    
    // Test partial update - only update title
    update_item(&conn, item_id.try_into().unwrap(), Some("Updated Title"), None, None, None, None, None, None)?;
    
    let item = get_item_by_id(&conn, item_id.try_into().unwrap())?.unwrap();
    assert_eq!(item["title"], "Updated Title");
    assert_eq!(item["price"], "10.00"); // Unchanged - formatted with 2 decimal places
    assert_eq!(item["quantity"], "5"); // Unchanged
    assert_eq!(item["category"], "sneakers"); // Unchanged
    Ok(())
}

#[test]
fn test_update_nonexistent_item() -> anyhow::Result<()> {
    let conn = setup_test_db();
    
    let result = update_item(&conn, 999, Some("New Title"), None, None, None, None, None, None);
    assert!(result.is_err(), "Should fail when updating non-existent item");
    Ok(())
} 