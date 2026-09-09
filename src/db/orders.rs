use rusqlite::{Connection, Result, params, OptionalExtension};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Order {
    pub id: i64,
    pub table_name: String,
    pub subtotal: f64,
    pub gst: f64,
    pub packaging: f64,
    pub delivery: f64,
    pub discount: f64,
    pub total: f64,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub name: String,
    pub price: f64,
    pub quantity: i32,
}

pub fn create_order(conn: &Connection, table_name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO orders (table_name) VALUES (?1)",
        [table_name],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_open_order_by_table(conn: &Connection, table_name: &str) -> Result<Option<Order>> {
    let mut stmt = conn.prepare(
        "SELECT id, table_name, subtotal, gst, packaging, delivery, discount, total
         FROM orders WHERE table_name = ?1 AND status = 'open'
         ORDER BY id DESC LIMIT 1",
    )?;
    let order = stmt.query_row([table_name], |row| {
        Ok(Order {
            id: row.get(0)?,
            table_name: row.get(1)?,
            subtotal: row.get(2)?,
            gst: row.get(3)?,
            packaging: row.get(4)?,
            delivery: row.get(5)?,
            discount: row.get(6)?,
            total: row.get(7)?,
        })
    }).optional()?;
    Ok(order)
}

pub fn get_order_items(conn: &Connection, order_id: i64) -> Result<Vec<OrderItem>> {
    let mut stmt = conn.prepare(
        "SELECT name, price, quantity FROM order_items WHERE order_id = ?1",
    )?;
    let items = stmt
        .query_map([order_id], |row| {
            Ok(OrderItem {
                name: row.get(0)?,
                price: row.get(1)?,
                quantity: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(items)
}

pub fn clear_order_items(conn: &Connection, order_id: i64) -> Result<()> {
    conn.execute("DELETE FROM order_items WHERE order_id = ?1", [order_id])?;
    Ok(())
}

pub fn add_order_item(
    conn: &Connection,
    order_id: i64,
    name: &str,
    price: f64,
    quantity: i32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO order_items (order_id, name, price, quantity) VALUES (?1, ?2, ?3, ?4)",
        params![order_id, name, price, quantity],
    )?;
    Ok(())
}

pub fn update_order_totals(
    conn: &Connection,
    order_id: i64,
    subtotal: f64,
    gst: f64,
    packaging: f64,
    delivery: f64,
    discount: f64,
    total: f64,
) -> Result<()> {
    conn.execute(
        "UPDATE orders SET subtotal = ?1, gst = ?2, packaging = ?3, delivery = ?4, discount = ?5, total = ?6, updated_at = datetime('now') WHERE id = ?7",
        params![subtotal, gst, packaging, delivery, discount, total, order_id],
    )?;
    Ok(())
}

pub fn close_order(conn: &Connection, order_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE orders SET status = 'closed', updated_at = datetime('now') WHERE id = ?1",
        [order_id],
    )?;
    Ok(())
}

pub fn get_open_order_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT table_name FROM orders WHERE status='open'")?;
    let tables = stmt.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>>>()?;
    Ok(tables)
}
