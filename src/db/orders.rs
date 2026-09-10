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
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_address: String,
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
        "SELECT id, table_name, subtotal, gst, packaging, delivery, discount, total,
                customer_name, customer_phone, customer_address
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
            customer_name: row.get(8)?,
            customer_phone: row.get(9)?,
            customer_address: row.get(10)?,
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

#[allow(dead_code)]
pub fn update_order_totals(
    conn: &Connection,
    order_id: i64,
    subtotal: f64,
    gst: f64,
    packaging: f64,
    delivery: f64,
    discount: f64,
    total: f64,
    customer_name: &str,
    customer_phone: &str,
    customer_address: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE orders SET subtotal = ?1, gst = ?2, packaging = ?3, delivery = ?4,
                          discount = ?5, total = ?6,
                          customer_name = ?7, customer_phone = ?8, customer_address = ?9,
                          updated_at = datetime('now')
         WHERE id = ?10",
        params![subtotal, gst, packaging, delivery, discount, total,
                customer_name, customer_phone, customer_address, order_id],
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

#[allow(dead_code)]
pub fn get_open_order_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT table_name FROM orders WHERE status='open'")?;
    let tables = stmt.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>>>()?;
    Ok(tables)
}

#[allow(dead_code)]
pub fn update_order_table(
    conn: &Connection,
    order_id: i64,
    new_table_name: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE orders SET table_name = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![new_table_name, order_id],
    )?;
    Ok(())
}

pub fn get_open_order_totals(conn: &Connection) -> Result<std::collections::HashMap<String, f64>> {
    let mut stmt = conn.prepare(
        "SELECT table_name, total FROM orders WHERE status='open'",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (name, total) = row?;
        map.insert(name, total);
    }
    Ok(map)
}
