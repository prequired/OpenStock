use std::process::Command;
use rusqlite::Connection;
use inventory::db::schema::{CREATE_ITEMS_TABLE_SQL, default_db_path};
use inventory::db::queries::{insert_item, NewItem, count_items};
use std::fs;

fn setup_test_db() -> Connection {
    // Use the actual database file for testing
    let db_path = default_db_path();
    let conn = Connection::open(&db_path).unwrap();
    
    // Clear existing data
    conn.execute("DELETE FROM items", []).unwrap();
    
    conn
}

fn insert_test_item(conn: &Connection, title: &str, price: f64, quantity: i32, category: &str) -> i32 {
    let item = NewItem {
        title,
        description: None,
        price,
        quantity,
        photos: None,
        category,
        condition: "new",
        brand: None,
        upc: None,
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
    
    insert_item(conn, &item).unwrap();
    
    // Get the inserted item's ID
    let mut stmt = conn.prepare("SELECT item_id FROM items WHERE title = ?").unwrap();
    let id: i32 = stmt.query_row([title], |row| row.get(0)).unwrap();
    id
}

#[test]
fn test_delete_item_nonexistent() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "delete-item",
            "--id", "999"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Command should fail with error
    assert!(!output.status.success(), "Command should fail for non-existent item");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for error message
    assert!(stderr.contains("does not exist"), "Should show item does not exist error");
    assert!(stderr.contains("999"), "Should mention the non-existent ID");
}

#[test]
fn test_delete_item_invalid_id() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "delete-item",
            "--id", "invalid"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Command should fail with error
    assert!(!output.status.success(), "Command should fail for invalid ID");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for error message
    assert!(stderr.contains("error"), "Should show error for invalid ID");
}

#[test]
fn test_delete_item_zero_id() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "delete-item",
            "--id", "0"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Command should fail with error
    assert!(!output.status.success(), "Command should fail for zero ID");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for error message
    assert!(stderr.contains("does not exist"), "Should show item does not exist error");
}

#[test]
fn test_delete_item_negative_id() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "delete-item",
            "--id=-1"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Command should fail with error
    assert!(!output.status.success(), "Command should fail for negative ID");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for error message
    assert!(stderr.contains("does not exist"), "Should show item does not exist error");
}

#[test]
fn test_delete_item_database_integration() {
    let conn = setup_test_db();
    
    // Insert a test item
    let item_id = insert_test_item(&conn, "Test Item for DB Integration", 10.99, 5, "sneakers");
    
    // Verify item exists
    let count_before = count_items(&conn).unwrap();
    assert_eq!(count_before, 1, "Should have one item before deletion");
    
    // Test that the item can be found in the database
    let mut stmt = conn.prepare("SELECT title FROM items WHERE item_id = ?").unwrap();
    let title: String = stmt.query_row([item_id], |row| row.get(0)).unwrap();
    assert_eq!(title, "Test Item for DB Integration", "Should find the inserted item");
    
    // Test database deletion directly (bypassing CLI for non-interactive testing)
    let rows_affected = conn.execute("DELETE FROM items WHERE item_id = ?", [item_id]).unwrap();
    assert_eq!(rows_affected, 1, "Should delete exactly one row");
    
    // Verify item is deleted
    let count_after = count_items(&conn).unwrap();
    assert_eq!(count_after, 0, "Should have no items after deletion");
    
    // Verify item no longer exists
    let result: Result<String, _> = stmt.query_row([item_id], |row| row.get(0));
    assert!(result.is_err(), "Item should no longer exist in database");
} 