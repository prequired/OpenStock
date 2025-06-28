use inventory::db::schema::initialize_database;
use inventory::commands::import::handle_import;
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_csv(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", content).unwrap();
    file
}

fn setup_test_db() -> Connection {
    let conn = initialize_database(None).unwrap();
    // Clear any existing data
    conn.execute("DELETE FROM items", []).unwrap();
    conn
}

#[test]
fn test_import_valid_csv() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Test Item,Test Description,29.99,5,123456789012,electronics,new,TestBrand
2,Another Item,Another Description,15.50,3,123456789013,clothing,used,AnotherBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    // Mock stdin for non-interactive test
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Verify items were imported
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 2);
    
    // Verify first item
    let mut stmt = conn.prepare("SELECT title, price, quantity, category, condition, brand FROM items WHERE item_id = 1").unwrap();
    let row = stmt.query_row([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    }).unwrap();
    
    assert_eq!(row.0, "Test Item");
    assert_eq!(row.1, 29.99);
    assert_eq!(row.2, 5);
    assert_eq!(row.3, "electronics");
    assert_eq!(row.4, "new");
    assert_eq!(row.5, Some("TestBrand".to_string()));
}

#[test]
fn test_import_missing_required_fields() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,price,quantity
1,Test Item,29.99,5"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing required field"));
}

#[test]
fn test_import_invalid_price() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Test Item,Test Description,-10.00,5,123456789012,electronics,new,TestBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed rows due to negative price
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_invalid_quantity() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Test Item,Test Description,29.99,-5,123456789012,electronics,new,TestBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed rows due to negative quantity
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_invalid_condition() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Test Item,Test Description,29.99,5,123456789012,electronics,invalid_condition,TestBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed rows due to invalid condition
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_empty_title() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,,Test Description,29.99,5,123456789012,electronics,new,TestBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed rows due to empty title
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_title_too_long() {
    let conn = setup_test_db();
    
    let long_title = "A".repeat(81); // 81 characters, exceeds eBay's 80-char limit
    let csv_content = format!(r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,{},Test Description,29.99,5,123456789012,electronics,new,TestBrand"#, long_title);
    
    let csv_file = create_test_csv(&csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed rows due to title too long
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_mixed_valid_invalid() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Valid Item,Valid Description,29.99,5,123456789012,electronics,new,TestBrand
2,Invalid Item,Invalid Description,-10.00,5,123456789013,electronics,new,TestBrand
3,Another Valid,Another Description,15.50,3,123456789014,clothing,used,AnotherBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have imported 2 valid items, skipped 1 invalid
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 2);
    
    // Verify valid items were imported
    let mut stmt = conn.prepare("SELECT title FROM items ORDER BY item_id").unwrap();
    let titles: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
        .map(|r| r.unwrap()).collect();
    
    assert_eq!(titles, vec!["Valid Item", "Another Valid"]);
}

#[test]
fn test_import_csv_parse_error() {
    let conn = setup_test_db();
    
    // Malformed CSV with unclosed quote - this will definitely cause a parse error
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,"Unclosed quote,Test Description,29.99,5,123456789012,electronics,new,TestBrand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have failed due to CSV parse error
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_empty_csv() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should have imported 0 items
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_import_file_not_found() {
    let conn = setup_test_db();
    
    let result = handle_import("nonexistent_file.csv".to_string(), &conn, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File not found"));
}

#[test]
fn test_import_with_optional_fields_empty() {
    let conn = setup_test_db();
    
    let csv_content = r#"item_id,title,description,price,quantity,upc,category,condition,brand
1,Test Item,Test Description,29.99,5,,electronics,new,"#;
    
    let csv_file = create_test_csv(csv_content);
    let file_path = csv_file.path().to_str().unwrap();
    
    let result = handle_import(file_path.to_string(), &conn, true);
    assert!(result.is_ok());
    
    // Should import successfully with empty optional fields
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);
    
    // Verify optional fields are NULL
    let mut stmt = conn.prepare("SELECT upc, brand FROM items WHERE item_id = 1").unwrap();
    let row = stmt.query_row([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
        ))
    }).unwrap();
    
    assert_eq!(row.0, None); // upc should be NULL
    assert_eq!(row.1, None); // brand should be NULL
} 