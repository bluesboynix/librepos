use rusqlite::{Connection, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Convert month number to a padded folder name like "09-september".
fn month_name(mm: &str) -> &'static str {
    match mm {
        "01" => "01-january",
        "02" => "02-february",
        "03" => "03-march",
        "04" => "04-april",
        "05" => "05-may",
        "06" => "06-june",
        "07" => "07-july",
        "08" => "08-august",
        "09" => "09-september",
        "10" => "10-october",
        "11" => "11-november",
        "12" => "12-december",
        _ => "00-unknown",
    }
}

/// Return `exports/YYYY/MM-month/DD` for today's date (local time).
pub fn today_export_folder(conn: &Connection) -> Result<PathBuf> {
    let y: String = conn.query_row(
        "SELECT strftime('%Y', 'now', 'localtime')",
        [],
        |r| r.get(0),
    )?;
    let m: String = conn.query_row(
        "SELECT strftime('%m', 'now', 'localtime')",
        [],
        |r| r.get(0),
    )?;
    let d: String = conn.query_row(
        "SELECT strftime('%d', 'now', 'localtime')",
        [],
        |r| r.get(0),
    )?;
    Ok(PathBuf::from("exports")
        .join(y)
        .join(month_name(&m))
        .join(d))
}

/// Return `exports/YYYY/MM-month/DD` for a given "YYYY-MM-DD HH:MM:SS"
/// (or "YYYY-MM-DD") string. Falls back to `exports/unknown` on error.
pub fn folder_for_date(date_str: &str) -> PathBuf {
    let date_part = date_str.split_whitespace().next().unwrap_or("");
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() < 3 {
        return PathBuf::from("exports").join("unknown");
    }
    PathBuf::from("exports")
        .join(parts[0])
        .join(month_name(parts[1]))
        .join(parts[2])
}

/// Escape a field for CSV.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv_row(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| csv_field(f))
        .collect::<Vec<_>>()
        .join(",")
        + "\n"
}

pub fn export_reports(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let folder = today_export_folder(conn)?;
    fs::create_dir_all(&folder)?;

    let suffix = format!("{}_{}", from, to);

    // ===== Bills =====
    {
        let path = folder.join(format!("bills_{}.csv", suffix));
        let mut file = fs::File::create(&path)?;
        file.write_all(
            csv_row(&[
                "Bill No".into(),
                "Table".into(),
                "Area".into(),
                "Closed At".into(),
                "Status".into(),
                "Payment Status".into(),
                "Subtotal".into(),
                "CGST".into(),
                "SGST".into(),
                "Packaging".into(),
                "Delivery".into(),
                "Discount".into(),
                "Total".into(),
                "Customer Name".into(),
                "Customer Phone".into(),
                "Customer Address".into(),
            ])
            .as_bytes(),
        )?;

        let mut stmt = conn.prepare(
            "SELECT COALESCE(bill_no, ''), table_name, COALESCE(area_name, ''),
                    COALESCE(closed_at, ''), status, COALESCE(payment_status, ''),
                    subtotal, cgst, sgst, packaging, delivery, discount, total,
                    customer_name, customer_phone, customer_address
             FROM orders
             WHERE status = 'closed'
               AND date(closed_at) BETWEEN ?1 AND ?2
             ORDER BY closed_at ASC",
        )?;
        let rows = stmt.query_map([from, to], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                format!("{:.2}", row.get::<_, f64>(6)?),
                format!("{:.2}", row.get::<_, f64>(7)?),
                format!("{:.2}", row.get::<_, f64>(8)?),
                format!("{:.2}", row.get::<_, f64>(9)?),
                format!("{:.2}", row.get::<_, f64>(10)?),
                format!("{:.2}", row.get::<_, f64>(11)?),
                format!("{:.2}", row.get::<_, f64>(12)?),
                row.get::<_, String>(13)?,
                row.get::<_, String>(14)?,
                row.get::<_, String>(15)?,
            ])
        })?;
        for r in rows {
            file.write_all(csv_row(&r?).as_bytes())?;
        }
    }

    // ===== Items =====
    {
        let path = folder.join(format!("items_{}.csv", suffix));
        let mut file = fs::File::create(&path)?;
        file.write_all(
            csv_row(&[
                "Bill No".into(),
                "Closed At".into(),
                "Item".into(),
                "Category".into(),
                "Qty".into(),
                "Unit Price".into(),
                "Line Total".into(),
                "Note".into(),
            ])
            .as_bytes(),
        )?;

        let mut stmt = conn.prepare(
            "SELECT COALESCE(o.bill_no, ''), COALESCE(o.closed_at, ''),
                    oi.name, COALESCE(oi.category, ''), oi.quantity,
                    oi.unit_price, oi.line_total, COALESCE(oi.note, '')
             FROM order_items oi
             JOIN orders o ON o.id = oi.order_id
             WHERE o.status = 'closed'
               AND date(o.closed_at) BETWEEN ?1 AND ?2
             ORDER BY o.closed_at ASC, oi.id ASC",
        )?;
        let rows = stmt.query_map([from, to], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?.to_string(),
                format!("{:.2}", row.get::<_, f64>(5)?),
                format!("{:.2}", row.get::<_, f64>(6)?),
                row.get::<_, String>(7)?,
            ])
        })?;
        for r in rows {
            file.write_all(csv_row(&r?).as_bytes())?;
        }
    }

    // ===== Payments =====
    {
        let path = folder.join(format!("payments_{}.csv", suffix));
        let mut file = fs::File::create(&path)?;
        file.write_all(
            csv_row(&[
                "Bill No".into(),
                "Closed At".into(),
                "Method".into(),
                "Amount".into(),
            ])
            .as_bytes(),
        )?;

        let mut stmt = conn.prepare(
            "SELECT COALESCE(o.bill_no, ''), COALESCE(o.closed_at, ''),
                    p.method, p.amount
             FROM payments p
             JOIN orders o ON o.id = p.order_id
             WHERE o.status = 'closed'
               AND date(o.closed_at) BETWEEN ?1 AND ?2
             ORDER BY o.closed_at ASC, p.id ASC",
        )?;
        let rows = stmt.query_map([from, to], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                format!("{:.2}", row.get::<_, f64>(3)?),
            ])
        })?;
        for r in rows {
            file.write_all(csv_row(&r?).as_bytes())?;
        }
    }

    Ok(folder.to_string_lossy().to_string())
}
