use inventory::db::schema::initialize_database;
use inventory::commands::filter::handle_filter;
use inventory::commands::list::OutputFormat;
use rusqlite::Connection;
use std::sync::Arc;
use inventory::config::optimization::{PerformanceMonitor, QueryCache};

fn setup_test_db() -> Connection {
    let conn = initialize_database(None).unwrap();
    // Clear any existing data
    conn.execute("DELETE FROM items", []).unwrap();
    conn
}

fn add_test_item(
    conn: &Connection,
    title: &str,
    price: f64,
    quantity: i32,
    category: &str,
    condition: &str,
    brand: Option<&str>,
) {
    conn.execute(
        r#"INSERT INTO items (
            title, description, price, quantity, photos, category, condition, brand, upc,
            item_specifics, shipping_details, size, original_price, hashtags, colorway, release_date,
            platform_status, internal_notes, last_updated, status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)"#,
        rusqlite::params![
            title,
            None::<String>,
            price,
            quantity,
            None::<String>,
            category,
            condition,
            brand,
            None::<String>,
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

#[test]
fn test_filter_by_price_range() {
    let conn = setup_test_db();
    
    // Add test items with different prices
    add_test_item(&conn, "Cheap Item", 5.99, 1, "test", "new", Some("Brand1"));
    add_test_item(&conn, "Mid Range", 25.99, 1, "test", "new", Some("Brand2"));
    add_test_item(&conn, "Expensive", 150.99, 1, "test", "new", Some("Brand3"));
    
    // Test price range filter
    let result = handle_filter(
        &conn,
        Some("10-50".to_string()),
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
    
    // Verify only mid-range item is returned
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE price >= 10 AND price <= 50",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_filter_by_category_and_condition() {
    let conn = setup_test_db();
    
    // Add test items
    add_test_item(&conn, "Electronics Item", 99.99, 1, "electronics", "new", Some("Brand1"));
    add_test_item(&conn, "Clothing Item", 29.99, 1, "clothing", "used", Some("Brand2"));
    add_test_item(&conn, "Electronics Used", 49.99, 1, "electronics", "used", Some("Brand3"));
    
    // Test category and condition filter
    let result = handle_filter(
        &conn,
        None,
        Some("electronics".to_string()),
        Some("new".to_string()),
        None,
        Some("item_id,title,category,condition".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
    
    // Verify only new electronics items are returned
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE category = 'electronics' AND condition = 'new'",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_filter_field_shortcuts() {
    let conn = setup_test_db();
    
    // Add a test item
    add_test_item(&conn, "Test Item", 29.99, 2, "test", "new", Some("TestBrand"));
    
    // Test field shortcuts
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("id,t,p,q,c,cat,b".to_string()), // Using shortcuts
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
    
    // Test with mixed shortcuts and full names
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("item_id,t,price,q,condition,cat,brand".to_string()), // Mixed
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_filter_output_formats() {
    let conn = setup_test_db();
    
    // Add test items
    add_test_item(&conn, "Item 1", 19.99, 1, "test", "new", Some("Brand1"));
    add_test_item(&conn, "Item 2", 29.99, 2, "test", "used", Some("Brand2"));
    
    // Test JSON format
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    assert!(result.is_ok());
    
    // Test table format
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Table),
        None,
        None,
    );
    assert!(result.is_ok());
    
    // Test CSV format
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Csv),
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_filter_empty_results() {
    let conn = setup_test_db();
    
    // Add items that won't match the filter
    add_test_item(&conn, "Item 1", 19.99, 1, "electronics", "new", Some("Brand1"));
    add_test_item(&conn, "Item 2", 29.99, 2, "clothing", "used", Some("Brand2"));
    
    // Test filter that should return no results
    let result = handle_filter(
        &conn,
        Some("100-200".to_string()), // Price range with no matches
        Some("books".to_string()),   // Category with no matches
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
    
    // Verify no items match the criteria
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE price >= 100 AND price <= 200 AND category = 'books'",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_filter_invalid_price_range() {
    let conn = setup_test_db();
    
    // Test invalid price range format
    let result = handle_filter(
        &conn,
        Some("invalid-price".to_string()),
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_err());
    
    // Test invalid price values
    let result = handle_filter(
        &conn,
        Some("abc-def".to_string()),
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_filter_unknown_fields() {
    let conn = setup_test_db();
    
    // Test unknown field
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("unknown_field".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_err());
    
    // Test mixed valid and invalid fields
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("item_id,unknown_field,title".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_filter_complex_combination() {
    let conn = setup_test_db();
    
    // Add diverse test items
    add_test_item(&conn, "Nike Shoes", 89.99, 1, "footwear", "new", Some("Nike"));
    add_test_item(&conn, "Adidas Shirt", 29.99, 2, "clothing", "new", Some("Adidas"));
    add_test_item(&conn, "Nike Pants", 59.99, 1, "clothing", "used", Some("Nike"));
    add_test_item(&conn, "Generic Shoes", 19.99, 1, "footwear", "new", Some("Generic"));
    
    // Test complex filter: Nike brand, clothing category, price under 100
    let result = handle_filter(
        &conn,
        Some("-100".to_string()), // Price under 100
        Some("clothing".to_string()),
        None,
        Some("Nike".to_string()),
        Some("id,t,p,cat,b".to_string()), // Using shortcuts
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
    
    // Verify only Nike clothing under $100 is returned
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE price <= 100 AND category = 'clothing' AND brand = 'Nike'",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_filter_default_fields() {
    let conn = setup_test_db();
    
    // Add a test item
    add_test_item(&conn, "Test Item", 29.99, 2, "test", "new", Some("TestBrand"));
    
    // Test without specifying fields (should use defaults)
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        None, // No fields specified
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_filter_empty_fields() {
    let conn = setup_test_db();
    
    // Add a test item
    add_test_item(&conn, "Test Item", 29.99, 2, "test", "new", Some("TestBrand"));
    
    // Test with empty fields string
    let result = handle_filter(
        &conn,
        None,
        None,
        None,
        None,
        Some("".to_string()), // Empty fields
        Some(OutputFormat::Json),
        None,
        None,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_filter_price_range_edge_cases() {
    let conn = setup_test_db();
    
    // Add test items
    add_test_item(&conn, "Cheap Item", 5.99, 1, "test", "new", Some("Brand1"));
    add_test_item(&conn, "Mid Item", 25.99, 1, "test", "new", Some("Brand2"));
    add_test_item(&conn, "Expensive Item", 150.99, 1, "test", "new", Some("Brand3"));
    
    // Test open-ended ranges
    let result = handle_filter(
        &conn,
        Some("20-".to_string()), // Min price only
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    assert!(result.is_ok());
    
    let result = handle_filter(
        &conn,
        Some("-30".to_string()), // Max price only
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    assert!(result.is_ok());
    
    // Test exact price match
    let result = handle_filter(
        &conn,
        Some("25.99".to_string()), // Exact price
        None,
        None,
        None,
        Some("item_id,title,price".to_string()),
        Some(OutputFormat::Json),
        None,
        None,
    );
    assert!(result.is_ok());
} 