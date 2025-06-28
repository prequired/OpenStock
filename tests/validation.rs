use inventory::validation::*;

#[test]
fn test_validate_title_ebay_valid() {
    let result = validate_title_ebay("Valid Title").unwrap();
    assert!(result.is_valid(), "Valid title should pass validation");
}

#[test]
fn test_validate_title_ebay_too_long() {
    let long_title = "a".repeat(81);
    let result = validate_title_ebay(&long_title).unwrap();
    assert!(!result.is_valid(), "Title over 80 chars should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "title");
    assert!(result.errors[0].message.contains("80-character limit"));
}

#[test]
fn test_validate_title_ebay_empty() {
    let result = validate_title_ebay("").unwrap();
    assert!(!result.is_valid(), "Empty title should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "title");
    assert!(result.errors[0].message.contains("cannot be empty"));
}

#[test]
fn test_validate_price_valid() {
    let result = validate_price(10.99).unwrap();
    assert!(result.is_valid(), "Valid price should pass validation");
}

#[test]
fn test_validate_price_negative() {
    let result = validate_price(-1.0).unwrap();
    assert!(!result.is_valid(), "Negative price should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "price");
    assert!(result.errors[0].message.contains("non-negative"));
}

#[test]
fn test_validate_price_too_high() {
    let result = validate_price(1000000.0).unwrap();
    assert!(!result.is_valid(), "Price over limit should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "price");
    assert!(result.errors[0].message.contains("maximum allowed"));
}

#[test]
fn test_validate_quantity_valid() {
    let result = validate_quantity(5).unwrap();
    assert!(result.is_valid(), "Valid quantity should pass validation");
}

#[test]
fn test_validate_quantity_negative() {
    let result = validate_quantity(-1).unwrap();
    assert!(!result.is_valid(), "Negative quantity should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "quantity");
    assert!(result.errors[0].message.contains("non-negative"));
}

#[test]
fn test_validate_upc_stockx_valid() {
    let result = validate_upc_stockx("123456789012").unwrap();
    assert!(result.is_valid(), "Valid 12-digit UPC should pass");
}

#[test]
fn test_validate_upc_stockx_13_digits() {
    let result = validate_upc_stockx("1234567890123").unwrap();
    assert!(result.is_valid(), "Valid 13-digit UPC should pass");
}

#[test]
fn test_validate_upc_stockx_empty() {
    let result = validate_upc_stockx("").unwrap();
    assert!(!result.is_valid(), "Empty UPC should fail for StockX");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "upc");
    assert!(result.errors[0].message.contains("required"));
}

#[test]
fn test_validate_upc_stockx_invalid_length() {
    let result = validate_upc_stockx("12345").unwrap();
    assert!(!result.is_valid(), "Invalid UPC length should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "upc");
    assert!(result.errors[0].message.contains("12 or 13 digits"));
}

#[test]
fn test_validate_upc_ebay_empty() {
    let result = validate_upc_ebay("").unwrap();
    assert!(result.is_valid(), "Empty UPC should be valid for eBay");
}

#[test]
fn test_validate_upc_ebay_invalid() {
    let result = validate_upc_ebay("12345").unwrap();
    assert!(!result.is_valid(), "Invalid UPC should fail for eBay");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "upc");
}

#[test]
fn test_validate_size_poshmark_empty() {
    let result = validate_size_poshmark("").unwrap();
    assert!(!result.is_valid(), "Empty size should fail for Poshmark");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "size");
    assert!(result.errors[0].message.contains("required"));
}

#[test]
fn test_validate_size_poshmark_valid() {
    let result = validate_size_poshmark("M").unwrap();
    assert!(result.is_valid(), "Valid size should pass for Poshmark");
}

#[test]
fn test_validate_condition_valid() {
    let valid_conditions = ["new", "used", "deadstock", "like new", "good", "fair"];
    
    for condition in valid_conditions.iter() {
        let result = validate_condition(condition).unwrap();
        assert!(result.is_valid(), "Valid condition '{}' should pass", condition);
    }
}

#[test]
fn test_validate_condition_invalid() {
    let result = validate_condition("invalid").unwrap();
    assert!(!result.is_valid(), "Invalid condition should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "condition");
    assert!(result.errors[0].message.contains("Invalid condition"));
}

#[test]
fn test_validate_category_empty() {
    let result = validate_category("").unwrap();
    assert!(!result.is_valid(), "Empty category should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "category");
    assert!(result.errors[0].message.contains("cannot be empty"));
}

#[test]
fn test_validate_category_valid() {
    let result = validate_category("sneakers").unwrap();
    assert!(result.is_valid(), "Valid category should pass");
}

#[test]
fn test_validate_brand_valid() {
    let result = validate_brand("Nike").unwrap();
    assert!(result.is_valid(), "Valid brand should pass");
}

#[test]
fn test_validate_brand_empty() {
    let result = validate_brand("").unwrap();
    assert!(result.is_valid(), "Empty brand should be valid");
}

#[test]
fn test_validate_brand_too_long() {
    let long_brand = "a".repeat(101);
    let result = validate_brand(&long_brand).unwrap();
    assert!(!result.is_valid(), "Brand over 100 chars should fail");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "brand");
    assert!(result.errors[0].message.contains("100 character limit"));
}

#[test]
fn test_validate_item_ebay_valid() {
    let result = validate_item_ebay(
        "Valid Title",
        10.99,
        5,
        "sneakers",
        "new",
        Some("Nike"),
        Some("123456789012"),
    ).unwrap();
    
    assert!(result.is_valid(), "Valid eBay item should pass validation");
}

#[test]
fn test_validate_item_ebay_multiple_errors() {
    let result = validate_item_ebay(
        "",  // Empty title
        -1.0,  // Negative price
        5,
        "sneakers",
        "new",
        Some("Nike"),
        Some("123456789012"),
    ).unwrap();
    
    assert!(!result.is_valid(), "Item with multiple errors should fail");
    assert!(result.errors.len() >= 2, "Should have at least 2 errors");
    
    let error_fields: Vec<&str> = result.errors.iter().map(|e| e.field.as_str()).collect();
    assert!(error_fields.contains(&"title"), "Should have title error");
    assert!(error_fields.contains(&"price"), "Should have price error");
}

#[test]
fn test_validate_item_stockx_valid() {
    let result = validate_item_stockx(
        "Valid Title",
        10.99,
        5,
        "sneakers",
        "new",
        Some("Nike"),
        "123456789012",
        "M",
    ).unwrap();
    
    assert!(result.is_valid(), "Valid StockX item should pass validation");
}

#[test]
fn test_validate_item_stockx_missing_required() {
    let result = validate_item_stockx(
        "Valid Title",
        10.99,
        5,
        "sneakers",
        "new",
        Some("Nike"),
        "",  // Empty UPC
        "",  // Empty size
    ).unwrap();
    
    assert!(!result.is_valid(), "StockX item missing required fields should fail");
    assert!(result.errors.len() >= 2, "Should have at least 2 errors");
    
    let error_fields: Vec<&str> = result.errors.iter().map(|e| e.field.as_str()).collect();
    assert!(error_fields.contains(&"upc"), "Should have UPC error");
    assert!(error_fields.contains(&"size"), "Should have size error");
}

#[test]
fn test_validation_result_json_output() {
    let mut result = ValidationResult::new();
    result.add_error("title", "Test error", Some(1), Some("test value"));
    
    let json = result.to_json().unwrap();
    assert!(json.contains("title"), "JSON should contain field name");
    assert!(json.contains("Test error"), "JSON should contain error message");
    assert!(json.contains("test value"), "JSON should contain value");
}

#[test]
fn test_validation_result_is_valid() {
    let mut result = ValidationResult::new();
    assert!(result.is_valid(), "Empty result should be valid");
    
    result.add_error("test", "error", None, None);
    assert!(!result.is_valid(), "Result with errors should not be valid");
} 