use inventory::db::schema::initialize_database;
use inventory::commands::stats::handle_stats;
use inventory::commands::list::OutputFormat;
use rusqlite::Connection;
use std::io::Write;
use tempfile::NamedTempFile;
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
fn test_stats_empty_database() {
    let conn = setup_test_db();
    
    // Test table format (default)
    let result = handle_stats(&conn, None, None, None);
    assert!(result.is_ok());
    
    // Test JSON format
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Test CSV format
    let result = handle_stats(&conn, Some(OutputFormat::Csv), None, None);
    assert!(result.is_ok());
}

#[test]
fn test_stats_single_item() {
    let conn = setup_test_db();
    
    // Add a single item
    add_test_item(&conn, "Test Item", 29.99, 2, "electronics", "new", Some("TestBrand"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify the item was added
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);
    
    // Verify total value calculation (price * quantity)
    let total_value: f64 = conn.query_row(
        "SELECT SUM(price * quantity) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(total_value, 29.99 * 2.0);
}

#[test]
fn test_stats_multiple_categories() {
    let conn = setup_test_db();
    
    // Add items from different categories
    add_test_item(&conn, "Laptop", 999.99, 1, "electronics", "new", Some("Dell"));
    add_test_item(&conn, "T-Shirt", 19.99, 3, "clothing", "new", Some("Nike"));
    add_test_item(&conn, "Book", 12.99, 2, "books", "used", Some("Penguin"));
    add_test_item(&conn, "Phone", 599.99, 1, "electronics", "new", Some("Apple"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify we have 3 categories
    let category_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT category) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(category_count, 3);
    
    // Verify electronics has 2 items
    let electronics_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE category = 'electronics'",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(electronics_count, 2);
}

#[test]
fn test_stats_different_conditions() {
    let conn = setup_test_db();
    
    // Add items with different conditions
    add_test_item(&conn, "New Item", 100.0, 1, "test", "new", Some("Brand1"));
    add_test_item(&conn, "Used Item", 50.0, 1, "test", "used", Some("Brand2"));
    add_test_item(&conn, "Like New", 75.0, 1, "test", "like new", Some("Brand3"));
    add_test_item(&conn, "Good Item", 25.0, 1, "test", "good", Some("Brand4"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify we have 4 different conditions
    let condition_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT condition) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(condition_count, 4);
}

#[test]
fn test_stats_brands_with_nulls() {
    let conn = setup_test_db();
    
    // Add items with and without brands
    add_test_item(&conn, "Branded Item", 100.0, 1, "test", "new", Some("Nike"));
    add_test_item(&conn, "No Brand Item", 50.0, 1, "test", "new", None);
    add_test_item(&conn, "Another Branded", 75.0, 1, "test", "new", Some("Adidas"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify we have 3 brands (including "Unknown" for NULL)
    let brand_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT COALESCE(brand, 'Unknown')) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(brand_count, 3);
}

#[test]
fn test_stats_price_ranges() {
    let conn = setup_test_db();
    
    // Add items in different price ranges
    add_test_item(&conn, "Cheap Item", 5.99, 1, "test", "new", Some("Brand1")); // Under $10
    add_test_item(&conn, "Budget Item", 15.99, 1, "test", "new", Some("Brand2")); // Under $25
    add_test_item(&conn, "Mid Range", 35.99, 1, "test", "new", Some("Brand3")); // Under $50
    add_test_item(&conn, "Expensive", 150.99, 1, "test", "new", Some("Brand4")); // Under $250
    add_test_item(&conn, "Premium", 500.99, 1, "test", "new", Some("Brand5")); // Over $250
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify we have items in different price ranges
    let under_10: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE price < 10",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(under_10, 1);
    
    let over_250: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE price >= 250",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(over_250, 1);
}

#[test]
fn test_stats_average_price_calculation() {
    let conn = setup_test_db();
    
    // Add items with known prices
    add_test_item(&conn, "Item 1", 10.0, 1, "test", "new", Some("Brand1"));
    add_test_item(&conn, "Item 2", 20.0, 1, "test", "new", Some("Brand2"));
    add_test_item(&conn, "Item 3", 30.0, 1, "test", "new", Some("Brand3"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify average price calculation
    let avg_price: f64 = conn.query_row(
        "SELECT AVG(price) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(avg_price, 20.0); // (10 + 20 + 30) / 3 = 20
}

#[test]
fn test_stats_total_value_calculation() {
    let conn = setup_test_db();
    
    // Add items with quantities
    add_test_item(&conn, "Item 1", 10.0, 2, "test", "new", Some("Brand1")); // 10 * 2 = 20
    add_test_item(&conn, "Item 2", 15.0, 3, "test", "new", Some("Brand2")); // 15 * 3 = 45
    add_test_item(&conn, "Item 3", 25.0, 1, "test", "new", Some("Brand3")); // 25 * 1 = 25
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify total value calculation (price * quantity)
    let total_value: f64 = conn.query_row(
        "SELECT SUM(price * quantity) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(total_value, 90.0); // 20 + 45 + 25 = 90
}

#[test]
fn test_stats_table_format() {
    let conn = setup_test_db();
    
    // Add some test data
    add_test_item(&conn, "Test Item", 29.99, 2, "electronics", "new", Some("TestBrand"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Table), None, None);
    assert!(result.is_ok());
}

#[test]
fn test_stats_csv_format() {
    let conn = setup_test_db();
    
    // Add some test data
    add_test_item(&conn, "Test Item", 29.99, 2, "electronics", "new", Some("TestBrand"));
    
    let result = handle_stats(&conn, Some(OutputFormat::Csv), None, None);
    assert!(result.is_ok());
}

#[test]
fn test_stats_large_dataset() {
    let conn = setup_test_db();
    
    // Add many items to test performance
    for i in 0..1000 {
        conn.execute(
            r#"INSERT INTO items (
                title, description, price, quantity, photos, category, condition, brand, upc,
                item_specifics, shipping_details, size, original_price, hashtags, colorway, release_date,
                platform_status, internal_notes, last_updated, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)"#,
            rusqlite::params![
                format!("Item {}", i),
                format!("Description for item {}", i),
                (i % 100) as f64 + 10.0,
                (i % 10) + 1,
                None::<String>,
                match i % 5 { 0 => "electronics", 1 => "clothing", 2 => "books", 3 => "sports", _ => "home" },
                match i % 4 { 0 => "new", 1 => "used", 2 => "like new", _ => "good" },
                match i % 8 { 0 => "Nike", 1 => "Apple", 2 => "Samsung", 3 => "Adidas", 4 => "Dell", 5 => "Sony", 6 => "Puma", _ => "Microsoft" },
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
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), None, None);
    assert!(result.is_ok());
    
    // Verify we have 1000 items
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1000);
    
    // Verify we have 5 categories
    let category_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT category) FROM items",
        [],
        |row| row.get(0)
    ).unwrap();
    assert_eq!(category_count, 5);
}

#[test]
fn test_stats_with_performance_monitoring() {
    let conn = setup_test_db();
    
    // Add some test items with varied data
    let items = vec![
        ("iPhone 13", "Latest smartphone", 999.99, 5, "electronics", "new", "Apple"),
        ("Nike Air Max", "Running shoes", 129.99, 10, "clothing", "new", "Nike"),
        ("Python Programming", "Programming book", 49.99, 3, "books", "used", "O'Reilly"),
        ("Samsung TV", "4K Smart TV", 799.99, 2, "electronics", "like new", "Samsung"),
        ("Adidas T-Shirt", "Cotton t-shirt", 29.99, 15, "clothing", "new", "Adidas"),
        ("Coffee Maker", "Automatic coffee machine", 89.99, 4, "home", "used", "Breville"),
        ("Basketball", "Official size basketball", 39.99, 8, "sports", "new", "Spalding"),
        ("MacBook Pro", "Laptop computer", 1499.99, 1, "electronics", "like new", "Apple"),
        ("Running Shorts", "Athletic shorts", 24.99, 12, "clothing", "new", "Nike"),
        ("Cookbook", "Italian recipes", 34.99, 6, "books", "used", "Random House"),
    ];

    for (title, description, price, quantity, category, condition, brand) in items {
        conn.execute(
            r#"INSERT INTO items (
                title, description, price, quantity, photos, category, condition, brand, upc,
                item_specifics, shipping_details, size, original_price, hashtags, colorway, release_date,
                platform_status, internal_notes, last_updated, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)"#,
            rusqlite::params![
                title,
                description,
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
    
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    let result = handle_stats(&conn, Some(OutputFormat::Json), Some(monitor), Some(cache));
    assert!(result.is_ok());
} 