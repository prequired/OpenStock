use anyhow::{Result, Context};
use crate::validation::{validate_item_ebay, ValidationResult, ValidationError};
use crate::db::queries;
use clap::Parser;
use csv::ReaderBuilder;
use rusqlite::Connection;
use serde_json;
use std::fs::{self, File};
use std::io::{self, Write, IsTerminal};
use std::path::{Path, PathBuf};
use chrono::Utc;
use dirs::home_dir;

#[derive(Parser)]
pub struct Update {
    #[arg(long, short = 'f', help = "CSV file with updates")]
    pub file: Option<String>,
    #[arg(long, help = "JSON file with failed imports to retry")]
    pub retry: Option<String>,
}

pub fn execute(args: Update, conn: &Connection) -> anyhow::Result<()> {
    if args.file.is_some() && args.retry.is_some() {
        return Err(anyhow::anyhow!("Cannot specify both --file and --retry"));
    }

    if let Some(file) = args.file {
        update_from_csv(file, conn)?;
    } else if let Some(retry_file) = args.retry {
        update_from_retry(retry_file, conn)?;
    } else {
        return Err(anyhow::anyhow!("Must specify either --file or --retry"));
    }
    Ok(())
}

fn update_from_csv(file: String, conn: &Connection) -> anyhow::Result<()> {
    if !Path::new(&file).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&file)?;
    let headers = rdr.headers()?.clone();
    let mut failed_rows = Vec::new();
    let mut row_num = 1;

    for result in rdr.records() {
        row_num += 1;
        let record = result?;
        let row_data = UpdateRow::from_record(&record, &headers, row_num)?;

        // Check if item exists
        if queries::get_item_by_id(conn, row_data.id)?.is_none() {
            failed_rows.push(ValidationError {
                field: "id".to_string(),
                message: "Item not found".to_string(),
                row: Some(row_num),
                value: Some(row_data.id.to_string()),
            });
            continue;
        }

        // Get existing item data for validation
        let existing_item = queries::get_item_by_id(conn, row_data.id)?.unwrap();
        
        // Prepare data for validation (use existing values for fields not being updated)
        let title_for_validation = row_data.title.as_deref().unwrap_or(&existing_item["title"]);
        let price_for_validation = row_data.price.unwrap_or_else(|| existing_item["price"].parse().unwrap_or(0.0));
        let quantity_for_validation = row_data.quantity.unwrap_or_else(|| existing_item["quantity"].parse().unwrap_or(0));
        let category_for_validation = row_data.category.as_deref().unwrap_or(&existing_item["category"]);
        let condition_for_validation = row_data.condition.as_deref().unwrap_or(&existing_item["condition"]);
        let brand_for_validation = row_data.brand.as_deref().or_else(|| {
            let brand = &existing_item["brand"];
            if brand.is_empty() { None } else { Some(brand.as_str()) }
        });
        let upc_for_validation = row_data.upc.as_deref().or_else(|| {
            let upc = &existing_item["upc"];
            if upc.is_empty() { None } else { Some(upc.as_str()) }
        });

        // Validate the combined data
        let validation_result = validate_item_ebay(
            title_for_validation,
            price_for_validation,
            quantity_for_validation,
            category_for_validation,
            condition_for_validation,
            brand_for_validation,
            upc_for_validation,
        );

        let mut corrected = row_data.clone();
        if let Ok(validation) = validation_result {
            if !validation.errors.is_empty() {
                // Check if we're in an interactive terminal
                let noninteractive = std::env::var("INVENTORY_NONINTERACTIVE").is_ok();
                let is_interactive = io::stdin().is_terminal() && io::stdout().is_terminal() && !noninteractive;
                
                if is_interactive {
                    println!("Invalid row {}:", row_num);
                    for error in &validation.errors {
                        if !corrected.is_field_provided(&error.field) {
                            continue; // Skip validation for unprovided fields
                        }
                        print!("Invalid {} for row {}: {:?}. Enter new value or press Enter to skip: ",
                               error.field, row_num, error.value);
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let input = input.trim();

                        if input.is_empty() {
                            failed_rows.push(error.clone());
                        } else {
                            match error.field.as_str() {
                                "title" => corrected.title = Some(input.to_string()),
                                "price" => {
                                    if let Ok(price) = input.parse::<f64>() {
                                        corrected.price = Some(price);
                                    } else {
                                        failed_rows.push(ValidationError {
                                            field: "price".to_string(),
                                            message: "Invalid price format".to_string(),
                                            row: Some(row_num),
                                            value: Some(input.to_string()),
                                        });
                                    }
                                }
                                "quantity" => {
                                    if let Ok(quantity) = input.parse::<i32>() {
                                        corrected.quantity = Some(quantity);
                                    } else {
                                        failed_rows.push(ValidationError {
                                            field: "quantity".to_string(),
                                            message: "Invalid quantity format".to_string(),
                                            row: Some(row_num),
                                            value: Some(input.to_string()),
                                        });
                                    }
                                }
                                "condition" => corrected.condition = Some(input.to_string()),
                                "category" => corrected.category = Some(input.to_string()),
                                "brand" => corrected.brand = Some(input.to_string()),
                                "upc" => corrected.upc = Some(input.to_string()),
                                _ => {}
                            }
                        }
                    }
                } else {
                    // Non-interactive mode: add all validation errors to failed rows
                    for error in &validation.errors {
                        if corrected.is_field_provided(&error.field) {
                            failed_rows.push(error.clone());
                        }
                    }
                    continue; // Skip this row in non-interactive mode
                }

                // Re-validate corrected row
                let re_validation = validate_item_ebay(
                    corrected.title.as_deref().unwrap_or(title_for_validation),
                    corrected.price.unwrap_or(price_for_validation),
                    corrected.quantity.unwrap_or(quantity_for_validation),
                    corrected.category.as_deref().unwrap_or(category_for_validation),
                    corrected.condition.as_deref().unwrap_or(condition_for_validation),
                    corrected.brand.as_deref().or(brand_for_validation),
                    corrected.upc.as_deref().or(upc_for_validation),
                );

                if let Ok(re_validation) = re_validation {
                    if !re_validation.errors.is_empty() {
                        failed_rows.extend(re_validation.errors);
                        continue;
                    }
                } else {
                    continue;
                }
            }
        } else {
            // If validation_result is Err, treat as a failed row
            continue;
        }

        // Update item in database
        queries::update_item(
            conn,
            corrected.id,
            corrected.title.as_deref(),
            corrected.price,
            corrected.quantity,
            corrected.category.as_deref(),
            corrected.condition.as_deref(),
            corrected.brand.as_deref(),
            corrected.upc.as_deref(),
        )?;
    }

    // Save failed rows
    if !failed_rows.is_empty() {
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let failed_dir = home_dir()
            .expect("Failed to get home directory")
            .join(".inventory/failed");
        fs::create_dir_all(&failed_dir)?;
        let failed_path = failed_dir.join(format!("failed_update_{}.json", timestamp));
        let failed_file = File::create(&failed_path)?;
        serde_json::to_writer_pretty(failed_file, &ValidationResult { errors: failed_rows })?;
        println!("Failed rows saved to {}", failed_path.display());
    }

    println!("CSV update completed. {} rows processed.", row_num - 1);
    Ok(())
}

fn update_from_retry(file: String, conn: &Connection) -> anyhow::Result<()> {
    let failed_file = File::open(&file)?;
    let errors: ValidationResult = serde_json::from_reader(failed_file)?;
    let mut failed_rows = Vec::new();

    for error in errors.errors {
        let row_num = error.row.unwrap_or(0);
        let mut row_data = UpdateRow {
            id: error.value.clone().unwrap_or("0".to_string()).parse().unwrap_or(0),
            title: None,
            price: None,
            quantity: None,
            condition: None,
            category: None,
            brand: None,
            upc: None,
        };

        // Prompt for correction
        let noninteractive = std::env::var("INVENTORY_NONINTERACTIVE").is_ok();
        let is_interactive = io::stdin().is_terminal() && io::stdout().is_terminal() && !noninteractive;
        
        if is_interactive {
            print!("Correcting {} for row {}: {:?}. Enter new value or press Enter to skip: ",
                   error.field, row_num, error.value);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                failed_rows.push(error);
                continue;
            }

            match error.field.as_str() {
                "id" => {
                    if let Ok(id) = input.parse::<i64>() {
                        row_data.id = id;
                    } else {
                        failed_rows.push(ValidationError {
                            field: "id".to_string(),
                            message: "Invalid ID format".to_string(),
                            row: Some(row_num),
                            value: Some(input.to_string()),
                        });
                        continue;
                    }
                }
                "title" => row_data.title = Some(input.to_string()),
                "price" => {
                    if let Ok(price) = input.parse::<f64>() {
                        row_data.price = Some(price);
                    } else {
                        failed_rows.push(ValidationError {
                            field: "price".to_string(),
                            message: "Invalid price format".to_string(),
                            row: Some(row_num),
                            value: Some(input.to_string()),
                        });
                        continue;
                    }
                }
                "quantity" => {
                    if let Ok(quantity) = input.parse::<i32>() {
                        row_data.quantity = Some(quantity);
                    } else {
                        failed_rows.push(ValidationError {
                            field: "quantity".to_string(),
                            message: "Invalid quantity format".to_string(),
                            row: Some(row_num),
                            value: Some(input.to_string()),
                        });
                        continue;
                    }
                }
                "condition" => row_data.condition = Some(input.to_string()),
                "category" => row_data.category = Some(input.to_string()),
                "brand" => row_data.brand = Some(input.to_string()),
                "upc" => row_data.upc = Some(input.to_string()),
                _ => continue,
            }
        } else {
            // Non-interactive mode: add error to failed rows
            failed_rows.push(error);
            continue;
        }

        // Check if item exists
        if queries::get_item_by_id(conn, row_data.id)?.is_none() {
            failed_rows.push(ValidationError {
                field: "id".to_string(),
                message: "Item not found".to_string(),
                row: Some(row_num),
                value: Some(row_data.id.to_string()),
            });
            continue;
        }

        // Validate and update
        let validation_result = validate_item_ebay(
            &row_data.title.clone().unwrap_or_default(),
            row_data.price.unwrap_or(f64::MAX),
            row_data.quantity.unwrap_or(i32::MAX),
            &row_data.category.clone().unwrap_or_default(),
            &row_data.condition.clone().unwrap_or_default(),
            row_data.brand.as_deref(),
            row_data.upc.as_deref(),
        );

        if let Ok(validation) = validation_result {
            if !validation.errors.is_empty() {
                failed_rows.extend(validation.errors);
                continue;
            }
        } else {
            continue;
        }

        queries::update_item(
            conn,
            row_data.id,
            row_data.title.as_deref(),
            row_data.price,
            row_data.quantity,
            row_data.category.as_deref(),
            row_data.condition.as_deref(),
            row_data.brand.as_deref(),
            row_data.upc.as_deref(),
        )?;
    }

    // Save failed rows
    if !failed_rows.is_empty() {
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let failed_dir = home_dir()
            .expect("Failed to get home directory")
            .join(".inventory/failed");
        fs::create_dir_all(&failed_dir)?;
        let failed_path = failed_dir.join(format!("failed_update_{}.json", timestamp));
        let failed_file = File::create(&failed_path)?;
        serde_json::to_writer_pretty(failed_file, &ValidationResult { errors: failed_rows })?;
        println!("Failed rows saved to {}", failed_path.display());
    }

    println!("Retry update completed.");
    Ok(())
}

#[derive(Clone)]
struct UpdateRow {
    id: i64,
    title: Option<String>,
    price: Option<f64>,
    quantity: Option<i32>,
    condition: Option<String>,
    category: Option<String>,
    brand: Option<String>,
    upc: Option<String>,
}

impl UpdateRow {
    fn from_record(record: &csv::StringRecord, headers: &csv::StringRecord, row_num: usize) -> anyhow::Result<Self> {
        let mut id = 0;
        let mut title = None;
        let mut price = None;
        let mut quantity = None;
        let mut condition = None;
        let mut category = None;
        let mut brand = None;
        let mut upc = None;

        for (i, field) in record.iter().enumerate() {
            let header = headers.get(i).ok_or_else(|| anyhow::anyhow!("Missing header"))?;
            let trimmed_field = field.trim();
            if trimmed_field.is_empty() {
                continue;
            }
            match header {
                "id" => id = trimmed_field.parse().map_err(|_| anyhow::anyhow!("Invalid ID in row {}", row_num))?,
                "title" => title = Some(trimmed_field.to_string()),
                "price" => price = Some(trimmed_field.parse().map_err(|_| anyhow::anyhow!("Invalid price in row {}", row_num))?),
                "quantity" => quantity = Some(trimmed_field.parse().map_err(|_| anyhow::anyhow!("Invalid quantity in row {}", row_num))?),
                "condition" => condition = Some(trimmed_field.to_string()),
                "category" => category = Some(trimmed_field.to_string()),
                "brand" => brand = Some(trimmed_field.to_string()),
                "upc" => upc = Some(trimmed_field.to_string()),
                _ => {}
            }
        }

        if id == 0 {
            return Err(anyhow::anyhow!("Missing ID in row {}", row_num));
        }

        Ok(UpdateRow {
            id,
            title,
            price,
            quantity,
            condition,
            category,
            brand,
            upc,
        })
    }

    fn is_field_provided(&self, field: &str) -> bool {
        match field {
            "id" => true,
            "title" => self.title.is_some(),
            "price" => self.price.is_some(),
            "quantity" => self.quantity.is_some(),
            "condition" => self.condition.is_some(),
            "category" => self.category.is_some(),
            "brand" => self.brand.is_some(),
            "upc" => self.upc.is_some(),
            _ => false,
        }
    }
} 