use rusqlite::{Connection, Result, params};

pub fn add_payment(
    conn: &Connection,
    order_id: i64,
    method: &str,
    amount: f64,
    reference: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO payments (order_id, method, amount, reference)
         VALUES (?1, ?2, ?3, ?4)",
        params![order_id, method, amount, reference],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_payments_for_order(
    conn: &Connection,
    order_id: i64,
) -> Result<Vec<(String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT method, amount FROM payments WHERE order_id = ?1",
    )?;
    let rows = stmt
        .query_map([order_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn clear_payments(conn: &Connection, order_id: i64) -> Result<()> {
    conn.execute("DELETE FROM payments WHERE order_id = ?1", [order_id])?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_sales_by_method(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<Vec<(String, f64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT method, SUM(amount), COUNT(*)
         FROM payments
         WHERE created_at BETWEEN ?1 AND ?2
         GROUP BY method
         ORDER BY SUM(amount) DESC",
    )?;
    let rows = stmt
        .query_map([from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}
