use rusqlite::{Connection, Result};

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub price: f64,
    pub category: String,
}

pub fn add_menu_item(conn: &Connection, name: &str, code: Option<&str>, price: f64, category: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO menu_items (name, code, price, category) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, code, price, category],
    )?;
    Ok(())
}

pub fn get_all_menu_items(conn: &Connection) -> Result<Vec<MenuItem>> {
    let mut stmt = conn.prepare("SELECT id, name, code, price, category FROM menu_items ORDER BY name")?;
    let items = stmt
        .query_map([], |row| {
            Ok(MenuItem {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                price: row.get(3)?,
                category: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(items)
}

pub fn delete_menu_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM menu_items WHERE id = ?1", [id])?;
    Ok(())
}
