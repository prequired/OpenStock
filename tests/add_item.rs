use std::process::Command;
use rusqlite::Connection;
use inventory::db::schema::CREATE_ITEMS_TABLE_SQL;
use inventory::db::queries::count_items;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(CREATE_ITEMS_TABLE_SQL, []).unwrap();
    conn
}

#[test]
fn test_add_item_success() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Test Item",
            "--price", "10.99",
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "new",
            "--brand", "Nike",
            "--description", "Test description",
            "--upc", "123456789012",
            "--size", "M"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Check that the command succeeded
    assert!(output.status.success(), "Command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for success message
    assert!(stdout.contains("Successfully added item"), "Should show success message");
    assert!(stdout.contains("Test Item"), "Should show item title");
    assert!(stdout.contains("$10.99"), "Should show price");
    assert!(stdout.contains("qty: 5"), "Should show quantity");
    assert!(stdout.contains("sneakers"), "Should show category");
    assert!(stdout.contains("new"), "Should show condition");
    assert!(stdout.contains("Nike"), "Should show brand");
    assert!(stdout.contains("Test description"), "Should show description");
    assert!(stdout.contains("UPC: 123456789012"), "Should show UPC");
    assert!(stdout.contains("Size: M"), "Should show size");
    
    // Check that no validation errors were output (warnings are ok)
    assert!(!stderr.contains("errors"), "Should not output validation errors to stderr");
}

#[test]
fn test_add_item_validation_error_title_too_long() {
    let long_title = "a".repeat(81);
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", &long_title,
            "--price", "10.99",
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Command should still exit successfully (validation error is not a program error)
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("title"), "Should mention title field");
    assert!(stderr.contains("80-character limit"), "Should mention character limit");
    assert!(stderr.contains(&long_title), "Should include the invalid value");
}

#[test]
fn test_add_item_validation_error_negative_price() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Test Item",
            "--price=-1.0",
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("price"), "Should mention price field");
    assert!(stderr.contains("non-negative"), "Should mention non-negative requirement");
    assert!(stderr.contains("-1"), "Should include the invalid value");
}

#[test]
fn test_add_item_validation_error_invalid_condition() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Test Item",
            "--price", "10.99",
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "invalid_condition"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("condition"), "Should mention condition field");
    assert!(stderr.contains("Invalid condition"), "Should mention invalid condition");
}

#[test]
fn test_add_item_validation_error_empty_title() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "",
            "--price", "10.99",
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("title"), "Should mention title field");
    assert!(stderr.contains("cannot be empty"), "Should mention empty title");
}

#[test]
fn test_add_item_validation_error_empty_category() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Test Item",
            "--price", "10.99",
            "--quantity", "5",
            "--category", "",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("category"), "Should mention category field");
    assert!(stderr.contains("cannot be empty"), "Should mention empty category");
}

#[test]
fn test_add_item_validation_error_multiple_errors() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "",  // Empty title
            "--price=-1.0",  // Negative price
            "--quantity", "5",
            "--category", "sneakers",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should exit successfully even with validation errors");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check for JSON error output with multiple errors
    assert!(stderr.contains("errors"), "Should output JSON with errors");
    assert!(stderr.contains("title"), "Should mention title field");
    assert!(stderr.contains("price"), "Should mention price field");
    
    // Find the JSON output (it should be the last JSON object in stderr)
    if let Some(json_start) = stderr.rfind('{') {
        if let Some(json_end) = stderr.rfind('}') {
            let json_str = &stderr[json_start..=json_end];
            
            // Verify it's valid JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                assert!(parsed["errors"].is_array(), "Should have errors array");
                let errors = parsed["errors"].as_array().unwrap();
                assert!(errors.len() >= 2, "Should have at least 2 errors");
            }
        }
    }
}

#[test]
fn test_add_item_minimal_required_fields() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Minimal Item",
            "--price", "5.99",
            "--quantity", "1",
            "--category", "electronics",
            "--condition", "new"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should succeed with minimal fields");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully added item"), "Should show success message");
    assert!(stdout.contains("Minimal Item"), "Should show item title");
}

#[test]
fn test_add_item_with_optional_fields() {
    let output = Command::new("cargo")
        .args([
            "run", "--", "add-item",
            "--title", "Optional Fields Item",
            "--price", "15.50",
            "--quantity", "3",
            "--category", "clothing",
            "--condition", "used",
            "--brand", "Adidas",
            "--description", "A test item with optional fields",
            "--upc", "987654321098",
            "--size", "L",
            "--original-price", "25.00",
            "--hashtags", "#test #item",
            "--colorway", "Black/White",
            "--release-date", "2023-01-15",
            "--internal-notes", "Test notes for this item"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should succeed with all optional fields");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully added item"), "Should show success message");
    assert!(stdout.contains("Optional Fields Item"), "Should show item title");
    assert!(stdout.contains("Adidas"), "Should show brand");
    assert!(stdout.contains("A test item with optional fields"), "Should show description");
    assert!(stdout.contains("UPC: 987654321098"), "Should show UPC");
    assert!(stdout.contains("Size: L"), "Should show size");
} 