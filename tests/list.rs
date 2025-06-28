use inventory::db::schema::initialize_database;
use inventory::db::queries::{insert_item, NewItem, get_all_items};
use inventory::output::format::{format_json, format_csv, format_table, InventoryItem};

#[test]
fn test_list_inventory_empty_database() {
    let conn = initialize_database(None).unwrap();
    
    let items = get_all_items(&conn).unwrap();
    assert_eq!(items.len(), 0);
    
    // Test JSON formatting for empty list
    let json_output = format_json(&items).unwrap();
    assert_eq!(json_output, "[]");
    
    // Test CSV formatting for empty list
    let csv_output = format_csv(&items).unwrap();
    assert_eq!(csv_output, "item_id,title,price,quantity,condition,category,brand\n");
    
    // Test table formatting for empty list
    let table_output = format_table(&items).unwrap();
    assert!(table_output.contains("ID  | Title"));
    assert!(table_output.contains("----|"));
}

#[test]
fn test_list_inventory_with_items() {
    let conn = initialize_database(None).unwrap();
    
    // Insert test items
    let item1 = NewItem {
        title: "Air Jordan 1 Retro High OG",
        description: Some("Classic sneaker in Chicago colorway"),
        price: 150.0,
        quantity: 2,
        photos: None,
        category: "sneakers",
        condition: "new",
        brand: Some("Nike"),
        upc: Some("123456789012"),
        item_specifics: None,
        shipping_details: None,
        size: Some("10"),
        original_price: Some(170.0),
        hashtags: Some("#jordan #sneakers"),
        colorway: Some("Chicago"),
        release_date: Some("2022-10-29"),
        platform_status: None,
        internal_notes: None,
        status: "active",
    };
    
    let item2 = NewItem {
        title: "Vintage Denim Jacket",
        description: Some("Classic 90s denim jacket"),
        price: 45.0,
        quantity: 1,
        photos: None,
        category: "clothing",
        condition: "used",
        brand: Some("Levi's"),
        upc: None,
        item_specifics: None,
        shipping_details: None,
        size: Some("M"),
        original_price: None,
        hashtags: Some("#vintage #denim"),
        colorway: None,
        release_date: None,
        platform_status: None,
        internal_notes: None,
        status: "active",
    };
    
    insert_item(&conn, &item1).unwrap();
    insert_item(&conn, &item2).unwrap();
    
    let items = get_all_items(&conn).unwrap();
    assert_eq!(items.len(), 2);
    
    // Verify item data
    assert_eq!(items[0].item_id, 1);
    assert_eq!(items[0].title, "Air Jordan 1 Retro High OG");
    assert_eq!(items[0].price, 150.0);
    assert_eq!(items[0].quantity, 2);
    assert_eq!(items[0].condition, "new");
    assert_eq!(items[0].category, "sneakers");
    assert_eq!(items[0].brand, Some("Nike".to_string()));
    
    assert_eq!(items[1].item_id, 2);
    assert_eq!(items[1].title, "Vintage Denim Jacket");
    assert_eq!(items[1].price, 45.0);
    assert_eq!(items[1].quantity, 1);
    assert_eq!(items[1].condition, "used");
    assert_eq!(items[1].category, "clothing");
    assert_eq!(items[1].brand, Some("Levi's".to_string()));
}

#[test]
fn test_json_formatting() {
    let items = vec![
        InventoryItem::new(1, "Test Item 1".to_string(), 100.0, 2, "new".to_string(), "test".to_string(), Some("Brand1".to_string())),
        InventoryItem::new(2, "Test Item 2".to_string(), 50.0, 1, "used".to_string(), "test".to_string(), None),
    ];
    
    let json_output = format_json(&items).unwrap();
    println!("JSON OUTPUT: {}", json_output);
    // Verify JSON structure
    assert!(json_output.contains("\"item_id\": 1"));
    assert!(json_output.contains("\"title\": \"Test Item 1\""));
    assert!(json_output.contains("\"price\": 100.0"));
    assert!(json_output.contains("\"quantity\": 2"));
    assert!(json_output.contains("\"condition\": \"new\""));
    assert!(json_output.contains("\"category\": \"test\""));
    assert!(json_output.contains("\"brand\": \"Brand1\""));
    
    assert!(json_output.contains("\"item_id\": 2"));
    assert!(json_output.contains("\"title\": \"Test Item 2\""));
    assert!(json_output.contains("\"price\": 50.0"));
    assert!(json_output.contains("\"quantity\": 1"));
    assert!(json_output.contains("\"condition\": \"used\""));
    assert!(json_output.contains("\"brand\": null"));
}

#[test]
fn test_csv_formatting() {
    let items = vec![
        InventoryItem::new(1, "Test Item 1".to_string(), 100.0, 2, "new".to_string(), "test".to_string(), Some("Brand1".to_string())),
        InventoryItem::new(2, "Test Item 2".to_string(), 50.0, 1, "used".to_string(), "test".to_string(), None),
    ];
    
    let csv_output = format_csv(&items).unwrap();
    let lines: Vec<&str> = csv_output.lines().collect();
    
    // Verify header
    assert_eq!(lines[0], "item_id,title,price,quantity,condition,category,brand");
    
    // Verify data rows
    assert_eq!(lines[1], "1,Test Item 1,100.00,2,new,test,Brand1");
    assert_eq!(lines[2], "2,Test Item 2,50.00,1,used,test,");
}

#[test]
fn test_table_formatting() {
    let items = vec![
        InventoryItem::new(1, "Short Title".to_string(), 100.0, 2, "new".to_string(), "test".to_string(), Some("Brand1".to_string())),
        InventoryItem::new(2, "This is a very long title that should be truncated at 50 characters".to_string(), 50.0, 1, "used".to_string(), "test".to_string(), None),
    ];
    
    let table_output = format_table(&items).unwrap();
    let lines: Vec<&str> = table_output.lines().collect();
    println!("TABLE OUTPUT:\n{}", table_output);
    // Verify header
    assert!(lines[0].contains("ID  | Title"));
    assert!(lines[0].contains("Price   | Qty | Condition | Category | Brand"));
    assert!(lines[1].contains("----|"));
    
    // Verify short title (not truncated)
    assert!(lines[2].contains("1    | Short Title"));
    assert!(lines[2].contains("$100.00"));
    assert!(lines[2].contains("2   | new"));
    
    // Verify long title (truncated)
    assert!(lines[3].contains("2    | This is a very long title that should be trunca..."));
    assert!(lines[3].contains("$50.00"));
    assert!(lines[3].contains("1   | used"));
}

#[test]
fn test_csv_escaping() {
    let items = vec![
        InventoryItem::new(1, "Item with, comma".to_string(), 100.0, 2, "new".to_string(), "test".to_string(), Some("Brand with \"quotes\"".to_string())),
    ];
    
    let csv_output = format_csv(&items).unwrap();
    let lines: Vec<&str> = csv_output.lines().collect();
    
    // Verify escaped fields
    assert_eq!(lines[1], "1,\"Item with, comma\",100.00,2,new,test,\"Brand with \"\"quotes\"\"\"");
} 