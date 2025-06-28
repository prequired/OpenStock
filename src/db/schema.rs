// Database schema implementation
// TODO: Implement SQLite schema creation 

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use dirs::home_dir;
use std::fs;
use crate::config::optimization::optimize_database;

pub const DB_FILENAME: &str = "inventory.db";

/// Returns the default path to the inventory database (~/.inventory/inventory.db)
pub fn default_db_path() -> PathBuf {
    let mut path = home_dir().expect("Could not determine home directory");
    path.push(".inventory");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path.push(DB_FILENAME);
    path
}

/// SQL for creating the items table with all constraints
pub const CREATE_ITEMS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS items (
    item_id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) <= 80),
    description TEXT,
    price REAL NOT NULL CHECK(price >= 0),
    quantity INTEGER NOT NULL CHECK(quantity >= 0),
    photos TEXT,
    category TEXT NOT NULL,
    condition TEXT NOT NULL,
    brand TEXT,
    upc TEXT,
    item_specifics TEXT,
    shipping_details TEXT,
    size TEXT,
    original_price REAL,
    hashtags TEXT,
    colorway TEXT,
    release_date TEXT,
    platform_status TEXT,
    internal_notes TEXT,
    last_updated TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('active', 'sold', 'draft'))
);
"#;

/// Initializes the database and creates the items table if it doesn't exist
pub fn initialize_database(db_path: Option<&PathBuf>) -> Result<Connection> {
    let conn = match db_path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).expect("Failed to create database directory");
                }
            }
            Connection::open(path)?
        },
        None => Connection::open_in_memory()?,
    };
    
    // Create the items table
    conn.execute(CREATE_ITEMS_TABLE_SQL, [])?;
    
    // Apply database optimizations and create indexes
    if let Err(e) = optimize_database(&conn) {
        eprintln!("Warning: Failed to optimize database: {}", e);
        // Continue without optimization rather than failing
    }
    
    Ok(conn)
} 