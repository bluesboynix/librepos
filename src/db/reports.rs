use rusqlite::{Connection, Result};
use rusqlite::OptionalExtension;

pub struct BillDetail {
    pub id: i64,
    pub bill_no: String,
    pub table_name: String,
    pub area_name: String,
    pub closed_at: String,
    pub payment_status: String,
    pub status: String,
    pub void_reason: String,
    pub voided_at: String,
    pub subtotal: f64,
    pub cgst: f64,
    pub sgst: f64,
    pub gst: f64,
    pub packaging: f64,
    pub delivery: f64,
    pub discount: f64,
    pub total: f64,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_address: String,
    pub items: Vec<BillLineRow>,
    pub payments: Vec<BillPaymentRow>,
}

pub struct BillLineRow {
    pub name: String,
    pub category: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
    pub note: String,
}

pub struct BillPaymentRow {
    pub method: String,
    pub amount: f64,
}

pub fn get_bill_detail(conn: &Connection, order_id: i64) -> Result<Option<BillDetail>> {
    let order = conn
        .query_row(
                "SELECT id, COALESCE(bill_no, ''), table_name,
                    COALESCE(area_name, ''), COALESCE(closed_at, ''),
                    COALESCE(payment_status, ''),
                    COALESCE(status, 'closed'),
                    COALESCE(void_reason, ''),
                    COALESCE(voided_at, ''),
                    subtotal, cgst, sgst, gst, packaging, delivery, discount, total,
                    customer_name, customer_phone, customer_address
             FROM orders
             WHERE id = ?1
             LIMIT 1",
            [order_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,     // id
                    row.get::<_, String>(1)?,  // bill_no
                    row.get::<_, String>(2)?,  // table_name
                    row.get::<_, String>(3)?,  // area_name
                    row.get::<_, String>(4)?,  // closed_at
                    row.get::<_, String>(5)?,  // payment_status
                    row.get::<_, String>(6)?,  // status
                    row.get::<_, String>(7)?,  // void_reason
                    row.get::<_, String>(8)?,  // voided_at
                    row.get::<_, f64>(9)?,     // subtotal
                    row.get::<_, f64>(10)?,    // cgst
                    row.get::<_, f64>(11)?,    // sgst
                    row.get::<_, f64>(12)?,    // gst
                    row.get::<_, f64>(13)?,    // packaging
                    row.get::<_, f64>(14)?,    // delivery
                    row.get::<_, f64>(15)?,    // discount
                    row.get::<_, f64>(16)?,    // total
                    row.get::<_, String>(17)?, // customer_name
                    row.get::<_, String>(18)?, // customer_phone
                    row.get::<_, String>(19)?, // customer_address
                ))
            },
        )
        .optional()?;

        let Some((
        id, bill_no, table_name, area_name, closed_at, payment_status,
        status, void_reason, voided_at,
        subtotal, cgst, sgst, gst, packaging, delivery, discount, total,
        customer_name, customer_phone, customer_address,
    )) = order
    else {
        return Ok(None);
    };

    // Items query uses order_id (the parameter) now
    let mut stmt = conn.prepare(
        "SELECT name, COALESCE(category, ''), quantity, unit_price, line_total,
                COALESCE(note, '')
         FROM order_items WHERE order_id = ?1",
    )?;
    let items = stmt
        .query_map([order_id], |row| {
            Ok(BillLineRow {
                name: row.get(0)?,
                category: row.get(1)?,
                quantity: row.get(2)?,
                unit_price: row.get(3)?,
                line_total: row.get(4)?,
                note: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    let mut stmt = conn.prepare(
        "SELECT method, amount FROM payments WHERE order_id = ?1",
    )?;
    let payments = stmt
        .query_map([order_id], |row| {
            Ok(BillPaymentRow {
                method: row.get(0)?,
                amount: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

        Ok(Some(BillDetail {
        id,
        bill_no,
        table_name,
        area_name,
        closed_at,
        payment_status,
        status,
        void_reason,
        voided_at,
        subtotal,
        cgst,
        sgst,
        gst,
        packaging,
        delivery,
        discount,
        total,
        customer_name,
        customer_phone,
        customer_address,
        items,
        payments,
    }))
}


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
        "SELECT strftime('%m-%d', closed_at), SUM(total), COUNT(*)
         FROM orders
         WHERE status='closed'
           AND date(closed_at) BETWEEN ?1 AND ?2
         GROUP BY date(closed_at)
         ORDER BY date(closed_at) ASC",
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
    pub id: i64,
    pub bill_no: String,
    pub table_name: String,
    pub area_name: String,
    pub total: f64,
    pub closed_at: String,
    pub payment_status: String,
    pub status: String,
    pub void_reason: String,
}

pub fn get_bills(
    conn: &Connection,
    from: &str,
    to: &str,
    limit: i64,
    include_voided: bool,
) -> Result<Vec<BillRow>> {
    let sql = if include_voided {
        "SELECT id, COALESCE(bill_no, ''), table_name,
                COALESCE(area_name, ''), total,
                COALESCE(closed_at, ''), COALESCE(payment_status, ''),
                status, COALESCE(void_reason, '')
         FROM orders
         WHERE status IN ('closed', 'void')
           AND date(closed_at) BETWEEN ?1 AND ?2
         ORDER BY closed_at DESC
         LIMIT ?3"
    } else {
        "SELECT id, COALESCE(bill_no, ''), table_name,
                COALESCE(area_name, ''), total,
                COALESCE(closed_at, ''), COALESCE(payment_status, ''),
                status, COALESCE(void_reason, '')
         FROM orders
         WHERE status = 'closed'
           AND date(closed_at) BETWEEN ?1 AND ?2
         ORDER BY closed_at DESC
         LIMIT ?3"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(rusqlite::params![from, to, limit], |row| {
            Ok(BillRow {
                id: row.get(0)?,
                bill_no: row.get(1)?,
                table_name: row.get(2)?,
                area_name: row.get(3)?,
                total: row.get(4)?,
                closed_at: row.get(5)?,
                payment_status: row.get(6)?,
                status: row.get(7)?,
                void_reason: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}
