use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub row: Option<usize>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, field: &str, message: &str, row: Option<usize>, value: Option<&str>) {
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            row,
            value: value.map(|v| v.to_string()),
        });
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

// Platform-specific validation functions

pub fn validate_title_ebay(title: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if title.len() > 80 {
        result.add_error("title", "Exceeds eBay's 80-character limit", None, Some(title));
    }
    
    if title.trim().is_empty() {
        result.add_error("title", "Title cannot be empty", None, Some(title));
    }
    
    Ok(result)
}

pub fn validate_title_stockx(title: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if title.len() > 80 {
        result.add_error("title", "Exceeds StockX's 80-character limit", None, Some(title));
    }
    
    if title.trim().is_empty() {
        result.add_error("title", "Title cannot be empty", None, Some(title));
    }
    
    Ok(result)
}

pub fn validate_title_poshmark(title: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if title.len() > 80 {
        result.add_error("title", "Exceeds Poshmark's 80-character limit", None, Some(title));
    }
    
    if title.trim().is_empty() {
        result.add_error("title", "Title cannot be empty", None, Some(title));
    }
    
    Ok(result)
}

pub fn validate_title_mercari(title: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if title.len() > 80 {
        result.add_error("title", "Exceeds Mercari's 80-character limit", None, Some(title));
    }
    
    if title.trim().is_empty() {
        result.add_error("title", "Title cannot be empty", None, Some(title));
    }
    
    Ok(result)
}

pub fn validate_price(price: f64) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if price < 0.0 {
        result.add_error("price", "Price must be non-negative", None, Some(&price.to_string()));
    }
    
    if price > 999999.99 {
        result.add_error("price", "Price exceeds maximum allowed value", None, Some(&price.to_string()));
    }
    
    Ok(result)
}

pub fn validate_quantity(quantity: i32) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if quantity < 0 {
        result.add_error("quantity", "Quantity must be non-negative", None, Some(&quantity.to_string()));
    }
    
    if quantity > 999999 {
        result.add_error("quantity", "Quantity exceeds maximum allowed value", None, Some(&quantity.to_string()));
    }
    
    Ok(result)
}

pub fn validate_upc_stockx(upc: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if upc.trim().is_empty() {
        result.add_error("upc", "UPC is required for StockX listings", None, Some(upc));
        return Ok(result);
    }
    
    // Basic UPC validation (12 or 13 digits)
    let digits: Vec<char> = upc.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 12 && digits.len() != 13 {
        result.add_error("upc", "UPC must be 12 or 13 digits", None, Some(upc));
    }
    
    Ok(result)
}

pub fn validate_upc_ebay(upc: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if !upc.trim().is_empty() {
        // Basic UPC validation (12 or 13 digits)
        let digits: Vec<char> = upc.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 12 && digits.len() != 13 {
            result.add_error("upc", "UPC must be 12 or 13 digits", None, Some(upc));
        }
    }
    
    Ok(result)
}

pub fn validate_size_poshmark(size: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if size.trim().is_empty() {
        result.add_error("size", "Size is required for Poshmark clothing items", None, Some(size));
    }
    
    Ok(result)
}

pub fn validate_size_stockx(size: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if size.trim().is_empty() {
        result.add_error("size", "Size is required for StockX listings", None, Some(size));
    }
    
    Ok(result)
}

pub fn validate_condition(condition: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    let valid_conditions = ["new", "used", "deadstock", "like new", "good", "fair"];
    let condition_lower = condition.to_lowercase();
    
    if !valid_conditions.contains(&condition_lower.as_str()) {
        result.add_error(
            "condition", 
            &format!("Invalid condition. Must be one of: {}", valid_conditions.join(", ")), 
            None, 
            Some(condition)
        );
    }
    
    Ok(result)
}

pub fn validate_category(category: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if category.trim().is_empty() {
        result.add_error("category", "Category cannot be empty", None, Some(category));
    }
    
    Ok(result)
}

pub fn validate_brand(brand: &str) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    if !brand.trim().is_empty() && brand.len() > 100 {
        result.add_error("brand", "Brand name exceeds 100 character limit", None, Some(brand));
    }
    
    Ok(result)
}

// Platform-specific validation for complete items
pub fn validate_item_ebay(
    title: &str,
    price: f64,
    quantity: i32,
    category: &str,
    condition: &str,
    brand: Option<&str>,
    upc: Option<&str>,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Validate individual fields
    result.errors.extend(validate_title_ebay(title)?.errors);
    result.errors.extend(validate_price(price)?.errors);
    result.errors.extend(validate_quantity(quantity)?.errors);
    result.errors.extend(validate_category(category)?.errors);
    result.errors.extend(validate_condition(condition)?.errors);
    
    if let Some(brand_name) = brand {
        result.errors.extend(validate_brand(brand_name)?.errors);
    }
    
    if let Some(upc_code) = upc {
        result.errors.extend(validate_upc_ebay(upc_code)?.errors);
    }
    
    Ok(result)
}

pub fn validate_item_stockx(
    title: &str,
    price: f64,
    quantity: i32,
    category: &str,
    condition: &str,
    brand: Option<&str>,
    upc: &str,
    size: &str,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Validate individual fields
    result.errors.extend(validate_title_stockx(title)?.errors);
    result.errors.extend(validate_price(price)?.errors);
    result.errors.extend(validate_quantity(quantity)?.errors);
    result.errors.extend(validate_category(category)?.errors);
    result.errors.extend(validate_condition(condition)?.errors);
    result.errors.extend(validate_upc_stockx(upc)?.errors);
    result.errors.extend(validate_size_stockx(size)?.errors);
    
    if let Some(brand_name) = brand {
        result.errors.extend(validate_brand(brand_name)?.errors);
    }
    
    Ok(result)
}

pub fn validate_item_poshmark(
    title: &str,
    price: f64,
    quantity: i32,
    category: &str,
    condition: &str,
    brand: Option<&str>,
    size: &str,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Validate individual fields
    result.errors.extend(validate_title_poshmark(title)?.errors);
    result.errors.extend(validate_price(price)?.errors);
    result.errors.extend(validate_quantity(quantity)?.errors);
    result.errors.extend(validate_category(category)?.errors);
    result.errors.extend(validate_condition(condition)?.errors);
    result.errors.extend(validate_size_poshmark(size)?.errors);
    
    if let Some(brand_name) = brand {
        result.errors.extend(validate_brand(brand_name)?.errors);
    }
    
    Ok(result)
}

pub fn validate_item_mercari(
    title: &str,
    price: f64,
    quantity: i32,
    category: &str,
    condition: &str,
    brand: Option<&str>,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Validate individual fields
    result.errors.extend(validate_title_mercari(title)?.errors);
    result.errors.extend(validate_price(price)?.errors);
    result.errors.extend(validate_quantity(quantity)?.errors);
    result.errors.extend(validate_category(category)?.errors);
    result.errors.extend(validate_condition(condition)?.errors);
    
    if let Some(brand_name) = brand {
        result.errors.extend(validate_brand(brand_name)?.errors);
    }
    
    Ok(result)
} 