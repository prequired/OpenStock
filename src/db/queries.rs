// Database queries implementation
// TODO: Implement CRUD operations 

use rusqlite::{Connection, params, Result};
use chrono::Utc;
use crate::output::format::InventoryItem;

/// Minimal struct for testing insertions
pub struct NewItem<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub price: f64,
    pub quantity: i32,
    pub photos: Option<&'a str>,
    pub category: &'a str,
    pub condition: &'a str,
    pub brand: Option<&'a str>,
    pub upc: Option<&'a str>,
    pub item_specifics: Option<&'a str>,
    pub shipping_details: Option<&'a str>,
    pub size: Option<&'a str>,
    pub original_price: Option<f64>,
    pub hashtags: Option<&'a str>,
    pub colorway: Option<&'a str>,
    pub release_date: Option<&'a str>,
    pub platform_status: Option<&'a str>,
    pub internal_notes: Option<&'a str>,
    pub status: &'a str,
}

/// Insert a new item into the items table
pub fn insert_item(conn: &Connection, item: &NewItem) -> Result<usize> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"INSERT INTO items (
            title, description, price, quantity, photos, category, condition, brand, upc,
            item_specifics, shipping_details, size, original_price, hashtags, colorway, release_date,
            platform_status, internal_notes, last_updated, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)"#,
        params![
            item.title,
            item.description,
            item.price,
            item.quantity,
            item.photos,
            item.category,
            item.condition,
            item.brand,
            item.upc,
            item.item_specifics,
            item.shipping_details,
            item.size,
            item.original_price,
            item.hashtags,
            item.colorway,
            item.release_date,
            item.platform_status,
            item.internal_notes,
            now,
            item.status,
        ],
    )
}

/// Retrieve all items from the database
pub fn get_all_items(conn: &Connection) -> Result<Vec<InventoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT item_id, title, price, quantity, condition, category, brand FROM items ORDER BY item_id"
    )?;
    
    let items = stmt.query_map([], |row| {
        Ok(InventoryItem::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    })?
    .collect::<Result<Vec<_>>>()?;
    
    Ok(items)
}

/// Count items in the table (for test validation)
pub fn count_items(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM items")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

/// Get an item by ID
pub fn get_item_by_id(conn: &Connection, id: i64) -> Result<Option<std::collections::HashMap<String, String>>> {
    let fields = ["item_id", "title", "price", "quantity", "category", "condition", "brand", "upc"];
    let field_list = fields.join(", ");
    let mut stmt = conn.prepare(&format!("SELECT {} FROM items WHERE item_id = ?", field_list))?;
    let mut rows = stmt.query_map([id], |row| {
        let mut item = std::collections::HashMap::new();
        for (i, field) in fields.iter().enumerate() {
            let value: String = match field {
                &"price" => format!("{:.2}", row.get::<_, f64>(i)?),
                &"quantity" => row.get::<_, i32>(i)?.to_string(),
                &"item_id" => row.get::<_, i64>(i)?.to_string(),
                _ => row.get::<_, Option<String>>(i)?.unwrap_or_default(),
            };
            item.insert(field.to_string(), value);
        }
        Ok(item)
    })?;

    Ok(rows.next().transpose()?)
}

/// Update an item with partial updates (only update provided fields)
pub fn update_item(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    price: Option<f64>,
    quantity: Option<i32>,
    category: Option<&str>,
    condition: Option<&str>,
    brand: Option<&str>,
    upc: Option<&str>,
) -> Result<()> {
    let mut updates = Vec::new();
    let mut params = vec![];
    let last_updated = Utc::now().to_rfc3339();

    if let Some(title) = title {
        updates.push("title = ?".to_string());
        params.push(title.to_string());
    }
    if let Some(price) = price {
        updates.push("price = ?".to_string());
        params.push(price.to_string());
    }
    if let Some(quantity) = quantity {
        updates.push("quantity = ?".to_string());
        params.push(quantity.to_string());
    }
    if let Some(category) = category {
        updates.push("category = ?".to_string());
        params.push(category.to_string());
    }
    if let Some(condition) = condition {
        updates.push("condition = ?".to_string());
        params.push(condition.to_string());
    }
    if let Some(brand) = brand {
        updates.push("brand = ?".to_string());
        params.push(brand.to_string());
    }
    if let Some(upc) = upc {
        updates.push("upc = ?".to_string());
        params.push(upc.to_string());
    }
    updates.push("last_updated = ?".to_string());
    params.push(last_updated);

    if updates.is_empty() {
        return Ok(());
    }

    let query = format!("UPDATE items SET {} WHERE item_id = ?", updates.join(", "));
    params.push(id.to_string());
    let affected = conn.execute(&query, rusqlite::params_from_iter(params))?;
    if affected == 0 {
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }
    Ok(())
} 