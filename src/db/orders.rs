use rusqlite::{Connection, Result, OptionalExtension, params};

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
        "INSERT INTO orders (table_name, opened_at)
         VALUES (?1, datetime('now','localtime'))",
        [table_name],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_open_order_by_table(
    conn: &Connection,
    table_name: &str,
) -> Result<Option<Order>> {
    let mut stmt = conn.prepare(
        "SELECT id, table_name, subtotal, gst, packaging, delivery,
                discount, total, customer_name, customer_phone,
                customer_address
         FROM orders
         WHERE table_name = ?1 AND status = 'open'
         ORDER BY id DESC LIMIT 1",
    )?;
    let order = stmt
        .query_row([table_name], |row| {
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
        })
        .optional()?;
    Ok(order)
}

pub fn get_order_items(conn: &Connection, order_id: i64) -> Result<Vec<OrderItem>> {
    let mut stmt = conn.prepare(
        "SELECT name, unit_price, quantity FROM order_items WHERE order_id = ?1",
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

/// Insert one snapshot line. All prices/names are frozen at this moment.
#[allow(clippy::too_many_arguments)]
pub fn add_order_item_snapshot(
    conn: &Connection,
    order_id: i64,
    menu_item_id: Option<i64>,
    name: &str,
    code: &str,
    category: &str,
    unit_price: f64,
    cost_price: f64,
    quantity: i32,
    line_total: f64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO order_items
         (order_id, menu_item_id, name, code, category,
          unit_price, cost_price, quantity, line_total, price)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            order_id,
            menu_item_id,
            name,
            code,
            category,
            unit_price,
            cost_price,
            quantity,
            line_total,
            unit_price
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_order_totals(
    conn: &Connection,
    order_id: i64,
    subtotal: f64,
    cgst: f64,
    sgst: f64,
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
        "UPDATE orders SET
            subtotal = ?1, cgst = ?2, sgst = ?3, gst = ?4,
            packaging = ?5, delivery = ?6, discount = ?7, total = ?8,
            customer_name = ?9, customer_phone = ?10, customer_address = ?11,
            updated_at = datetime('now','localtime')
         WHERE id = ?12",
        params![
            subtotal, cgst, sgst, gst,
            packaging, delivery, discount, total,
            customer_name, customer_phone, customer_address,
            order_id
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn close_order(
    conn: &Connection,
    order_id: i64,
    bill_no: &str,
    area_name: &str,
    payment_status: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE orders SET
            status = 'closed',
            bill_no = ?1,
            area_name = ?2,
            payment_status = ?3,
            closed_at = datetime('now','localtime'),
            updated_at = datetime('now','localtime')
         WHERE id = ?4",
        params![bill_no, area_name, payment_status, order_id],
    )?;
    Ok(())
}

pub fn update_order_table(
    conn: &Connection,
    order_id: i64,
    new_table_name: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE orders SET table_name = ?1,
            updated_at = datetime('now','localtime')
         WHERE id = ?2",
        params![new_table_name, order_id],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_open_order_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT table_name FROM orders WHERE status='open'")?;
    let tables = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>>>()?;
    Ok(tables)
}

pub fn get_open_order_totals(
    conn: &Connection,
) -> Result<std::collections::HashMap<String, f64>> {
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

/// Get the cost price for a menu item by summing its ingredients.
pub fn get_menu_item_cost(conn: &Connection, menu_item_id: i64) -> Result<f64> {
    let cost: f64 = conn.query_row(
        "SELECT COALESCE(SUM(estimated_cost), 0)
         FROM menu_item_ingredients
         WHERE menu_item_id = ?1",
        [menu_item_id],
        |row| row.get(0),
    )?;
    Ok(cost)
}

/// Look up a menu item's code and category by id.
#[allow(dead_code)]
pub fn get_menu_item_meta(
    conn: &Connection,
    menu_item_id: i64,
) -> Result<(String, String, String)> {
    conn.query_row(
        "SELECT name, COALESCE(code, ''), COALESCE(category, '')
         FROM menu_items WHERE id = ?1",
        [menu_item_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}
