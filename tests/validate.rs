use inventory::commands::validate::handle_validate;
use std::fs::{self, File};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_csv(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", contents).unwrap();
    file
}

#[test]
fn test_validate_valid_file() {
    let csv = write_csv("item_id,title,description,price,quantity,upc,category,condition,brand\n1,Test,Desc,10.0,5,123,shoes,new,Nike\n");
    let result = handle_validate(csv.path().to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_validate_invalid_file() {
    let csv = write_csv("item_id,title,description,price,quantity,upc,category,condition,brand\n1,Test,Desc,-5.0,5,123,shoes,new,Nike\n2,LongTitleThatExceedsEbayEightyCharacterLimitLongTitleLongTitleLongTitleLongTitleLongTitle,Desc,10.0,5,123,shoes,new,Nike\n3,,Desc,10.0,5,123,shoes,new,Nike\n4,Test,Desc,abc,5,123,shoes,new,Nike\n5,Test,Desc,10.0,-1,123,shoes,new,Nike\n6,Test,Desc,10.0,5,123,shoes,,Nike\n");
    let result = handle_validate(csv.path().to_str().unwrap());
    assert!(result.is_err());
    // Check that a failed file was created
    let failed_dir = dirs::home_dir().unwrap().join(".inventory/failed");
    let found = fs::read_dir(&failed_dir).unwrap().any(|e| {
        e.as_ref().unwrap().file_name().to_str().unwrap().starts_with("validate_")
    });
    assert!(found);
}

#[test]
fn test_validate_schema_mismatch() {
    let csv = write_csv("id,title,description,price,quantity,upc,category,condition,brand\n1,Test,Desc,10.0,5,123,shoes,new,Nike\n");
    let result = handle_validate(csv.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_validate_empty_file() {
    let csv = write_csv("");
    let result = handle_validate(csv.path().to_str().unwrap());
    assert!(result.is_err());
} 