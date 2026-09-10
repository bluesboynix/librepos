use rusqlite::{Connection, Result};

pub struct Summary {
    pub total_sales: f64,
    pub order_count: i64,
    pub avg_order: f64,
}

pub struct ItemSale {
    pub name: String,
    pub quantity: i64,
    pub revenue: f64,
}

pub struct PaymentRow {
    pub method: String,
    pub amount: f64,
    pub count: i64,
}

pub struct DailySale {
    pub date: String,
    pub sales: f64,
    pub orders: i64,
}

pub fn get_summary(conn: &Connection, from: &str, to: &str) -> Result<Summary> {
    let (total, count): (f64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(total), 0), COUNT(*)
         FROM orders
         WHERE status='closed'
           AND date(closed_at) BETWEEN ?1 AND ?2",
        [from, to],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let avg = if count > 0 {
        total / count as f64
    } else {
        0.0
    };
    Ok(Summary {
        total_sales: total,
        order_count: count,
        avg_order: avg,
    })
}

pub fn get_top_items(
    conn: &Connection,
    from: &str,
    to: &str,
    limit: i64,
) -> Result<Vec<ItemSale>> {
    let mut stmt = conn.prepare(
        "SELECT oi.name, SUM(oi.quantity), SUM(oi.line_total)
         FROM order_items oi
         JOIN orders o ON o.id = oi.order_id
         WHERE o.status='closed'
           AND date(o.closed_at) BETWEEN ?1 AND ?2
         GROUP BY oi.name
         ORDER BY SUM(oi.line_total) DESC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![from, to, limit], |row| {
            Ok(ItemSale {
                name: row.get(0)?,
                quantity: row.get(1)?,
                revenue: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_payment_breakdown(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<Vec<PaymentRow>> {
    let mut stmt = conn.prepare(
        "SELECT p.method, SUM(p.amount), COUNT(DISTINCT p.order_id)
         FROM payments p
         JOIN orders o ON o.id = p.order_id
         WHERE o.status='closed'
           AND date(o.closed_at) BETWEEN ?1 AND ?2
         GROUP BY p.method
         ORDER BY SUM(p.amount) DESC",
    )?;
    let rows = stmt
        .query_map([from, to], |row| {
            Ok(PaymentRow {
                method: row.get(0)?,
                amount: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_daily_sales(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<Vec<DailySale>> {
    let mut stmt = conn.prepare(
        "SELECT date(closed_at), SUM(total), COUNT(*)
         FROM orders
         WHERE status='closed'
           AND date(closed_at) BETWEEN ?1 AND ?2
         GROUP BY date(closed_at)
         ORDER BY date(closed_at) DESC",
    )?;
    let rows = stmt
        .query_map([from, to], |row| {
            Ok(DailySale {
                date: row.get(0)?,
                sales: row.get(1)?,
                orders: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub struct BillRow {
    pub bill_no: String,
    pub table_name: String,
    pub area_name: String,
    pub total: f64,
    pub closed_at: String,
    pub payment_status: String,
}

pub fn get_bills(
    conn: &Connection,
    from: &str,
    to: &str,
    limit: i64,
) -> Result<Vec<BillRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(bill_no, ''),
                table_name,
                COALESCE(area_name, ''),
                total,
                COALESCE(closed_at, ''),
                COALESCE(payment_status, '')
         FROM orders
         WHERE status='closed'
           AND date(closed_at) BETWEEN ?1 AND ?2
         ORDER BY closed_at DESC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![from, to, limit], |row| {
            Ok(BillRow {
                bill_no: row.get(0)?,
                table_name: row.get(1)?,
                area_name: row.get(2)?,
                total: row.get(3)?,
                closed_at: row.get(4)?,
                payment_status: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}
