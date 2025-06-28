// Delete item command implementation
// TODO: Implement delete-item subcommand 

use anyhow::{Result, Context};
use rusqlite::Connection;
use crate::db::queries::count_items;

/// Check if an item exists in the database
fn item_exists(conn: &Connection, id: i32) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM items WHERE item_id = ?")?;
    let count: i64 = stmt.query_row([id], |row| row.get(0))?;
    Ok(count > 0)
}

/// Get item details for confirmation
fn get_item_details(conn: &Connection, id: i32) -> Result<Option<(String, f64, i32, String)>> {
    let mut stmt = conn.prepare(
        "SELECT title, price, quantity, category FROM items WHERE item_id = ?"
    )?;
    let result = stmt.query_row([id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, String>(3)?,
        ))
    });
    match result {
        Ok(details) => Ok(Some(details)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete an item from the database
fn delete_item(conn: &Connection, id: i32) -> Result<usize> {
    conn.execute("DELETE FROM items WHERE item_id = ?", [id])
        .with_context(|| format!("Failed to delete item with ID: {}", id))
}

pub fn handle_delete_item(id: i32, conn: &Connection) -> Result<()> {
    // Check if the item exists
    if !item_exists(conn, id)? {
        anyhow::bail!("Item with ID {} does not exist", id);
    }

    // Get item details for confirmation
    if let Some((title, price, quantity, category)) = get_item_details(conn, id)? {
        println!("Item to delete:");
        println!("  ID: {}", id);
        println!("  Title: {}", title);
        println!("  Price: ${:.2}", price);
        println!("  Quantity: {}", quantity);
        println!("  Category: {}", category);
        
        // Simple confirmation prompt
        print!("Are you sure you want to delete this item? (y/N): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input != "y" && input != "yes" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Delete the item
    let rows_affected = delete_item(conn, id)?;
    
    if rows_affected == 1 {
        println!("Successfully deleted item with ID: {}", id);
    } else {
        anyhow::bail!("Failed to delete item with ID: {} (no rows affected)", id);
    }
    
    Ok(())
} 