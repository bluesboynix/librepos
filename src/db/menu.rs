use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub price: f64,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub unit: Option<String>,
    pub unit_price: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct MenuItemIngredient {
    pub id: i64,
    pub menu_item_id: i64,
    pub ingredient_id: i64,
    pub quantity: f64,
    pub unit: Option<String>,
    pub estimated_cost: f64,
}

// ---------- Menu Items ----------
pub fn add_menu_item(conn: &Connection, name: &str, code: Option<&str>, price: f64, category: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO menu_items (name, code, price, category) VALUES (?1, ?2, ?3, ?4)",
        params![name, code, price, category],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_menu_item(conn: &Connection, id: i64) -> Result<MenuItem> {
    conn.query_row(
        "SELECT id, name, code, price, category FROM menu_items WHERE id = ?1",
        [id],
        |row| {
            Ok(MenuItem {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                price: row.get(3)?,
                category: row.get(4)?,
            })
        },
    )
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

pub fn update_menu_item(conn: &Connection, id: i64, name: &str, code: Option<&str>, price: f64, category: &str) -> Result<()> {
    conn.execute(
        "UPDATE menu_items SET name = ?1, code = ?2, price = ?3, category = ?4, updated_at = datetime('now') WHERE id = ?5",
        params![name, code, price, category, id],
    )?;
    Ok(())
}

pub fn delete_menu_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM menu_items WHERE id = ?1", [id])?;
    Ok(())
}

// ---------- Ingredients ----------
pub fn add_ingredient(conn: &Connection, name: &str, unit: Option<&str>, unit_price: Option<f64>) -> Result<i64> {
    conn.execute(
        "INSERT INTO ingredients (name, unit, unit_price) VALUES (?1, ?2, ?3)",
        params![name, unit, unit_price],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_all_ingredients(conn: &Connection) -> Result<Vec<Ingredient>> {
    let mut stmt = conn.prepare("SELECT id, name, unit, unit_price FROM ingredients ORDER BY name")?;
    let ingredients = stmt
        .query_map([], |row| {
            Ok(Ingredient {
                id: row.get(0)?,
                name: row.get(1)?,
                unit: row.get(2)?,
                unit_price: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(ingredients)
}

// ---------- Menu Item Ingredients ----------
pub fn add_ingredient_to_item(conn: &Connection, menu_item_id: i64, ingredient_id: i64, quantity: f64, unit: Option<&str>, estimated_cost: f64) -> Result<()> {
    conn.execute(
        "INSERT INTO menu_item_ingredients (menu_item_id, ingredient_id, quantity, unit, estimated_cost) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![menu_item_id, ingredient_id, quantity, unit, estimated_cost],
    )?;
    Ok(())
}

pub fn get_ingredients_for_item(conn: &Connection, menu_item_id: i64) -> Result<Vec<(i64, i64, String, f64, Option<String>, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT mii.id, i.id, i.name, mii.quantity, mii.unit, mii.estimated_cost
         FROM menu_item_ingredients mii
         JOIN ingredients i ON i.id = mii.ingredient_id
         WHERE mii.menu_item_id = ?1"
    )?;
    let rows = stmt
        .query_map([menu_item_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,   // link id
                row.get::<_, i64>(1)?,   // ingredient id
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, f64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn delete_ingredient_link(conn: &Connection, link_id: i64) -> Result<()> {
    conn.execute("DELETE FROM menu_item_ingredients WHERE id = ?1", [link_id])?;
    Ok(())
}

pub fn delete_all_ingredients_for_item(conn: &Connection, menu_item_id: i64) -> Result<()> {
    conn.execute("DELETE FROM menu_item_ingredients WHERE menu_item_id = ?1", [menu_item_id])?;
    Ok(())
}
