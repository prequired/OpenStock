// List inventory command implementation
// TODO: Implement list-inventory subcommand 

use anyhow::Result;
use rusqlite::Connection;
use crate::db::queries::get_all_items;
use crate::output::format::{format_items, InventoryItem};

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Csv,
    Table,
}

pub fn handle_list_inventory(conn: &Connection, format: Option<OutputFormat>) -> Result<()> {
    let format = format.unwrap_or(OutputFormat::Json);
    
    // Retrieve all items from the database
    let items = get_all_items(conn)?;
    
    // Format output according to specified format
    let format_str = match format {
        OutputFormat::Json => "json",
        OutputFormat::Csv => "csv",
        OutputFormat::Table => "table",
    };
    
    let output = format_items(&items, format_str)?;
    println!("{}", output);
    
    Ok(())
} 