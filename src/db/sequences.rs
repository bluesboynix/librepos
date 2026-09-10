use rusqlite::{Connection, Result, params};

/// Generate the next bill number for today, formatted as YYYYMMDD-NNNN.
pub fn next_bill_number(conn: &Connection) -> Result<String> {
    let day: String = conn.query_row(
        "SELECT strftime('%Y%m%d', 'now', 'localtime')",
        [],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO bill_sequences (day, last_seq) VALUES (?1, 1)
         ON CONFLICT(day) DO UPDATE SET last_seq = last_seq + 1",
        params![day],
    )?;

    let seq: i64 = conn.query_row(
        "SELECT last_seq FROM bill_sequences WHERE day = ?1",
        [&day],
        |row| row.get(0),
    )?;

    Ok(format!("{}-{:04}", day, seq))
}
