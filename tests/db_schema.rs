use rusqlite::Connection;
use inventory::db::schema::CREATE_ITEMS_TABLE_SQL;
use inventory::db::queries::{insert_item, NewItem, count_items};

fn setup_in_memory_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(CREATE_ITEMS_TABLE_SQL, []).unwrap();
    conn
}

#[test]
fn test_schema_creation() {
    let conn = setup_in_memory_db();
    // Table should exist, count should be 0
    let count = count_items(&conn).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_insert_valid_item() {
    let conn = setup_in_memory_db();
    let item = NewItem {
        title: "Valid Title",
        description: Some("desc"),
        price: 10.0,
        quantity: 5,
        photos: None,
        category: "shoes",
        condition: "new",
        brand: Some("Nike"),
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
    let rows = insert_item(&conn, &item).unwrap();
    assert_eq!(rows, 1);
    let count = count_items(&conn).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_title_length_constraint() {
    let conn = setup_in_memory_db();
    let long_title = "a".repeat(81);
    let item = NewItem {
        title: &long_title,
        description: None,
        price: 10.0,
        quantity: 1,
        photos: None,
        category: "shoes",
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
    let result = insert_item(&conn, &item);
    assert!(result.is_err(), "Should fail due to title length constraint");
}

#[test]
fn test_price_nonnegative_constraint() {
    let conn = setup_in_memory_db();
    let item = NewItem {
        title: "Test",
        description: None,
        price: -1.0,
        quantity: 1,
        photos: None,
        category: "shoes",
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
    let result = insert_item(&conn, &item);
    assert!(result.is_err(), "Should fail due to price >= 0 constraint");
}

#[test]
fn test_quantity_nonnegative_constraint() {
    let conn = setup_in_memory_db();
    let item = NewItem {
        title: "Test",
        description: None,
        price: 1.0,
        quantity: -5,
        photos: None,
        category: "shoes",
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
    let result = insert_item(&conn, &item);
    assert!(result.is_err(), "Should fail due to quantity >= 0 constraint");
}

#[test]
fn test_status_constraint() {
    let conn = setup_in_memory_db();
    let item = NewItem {
        title: "Test",
        description: None,
        price: 1.0,
        quantity: 1,
        photos: None,
        category: "shoes",
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
        status: "invalid_status",
    };
    let result = insert_item(&conn, &item);
    assert!(result.is_err(), "Should fail due to status constraint");
} 