use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub table_count: i64,
}

pub fn get_all_areas(conn: &Connection) -> Result<Vec<Area>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, table_count, COALESCE(sort_order, 0)
         FROM areas
         ORDER BY sort_order ASC, id ASC",
    )?;
    let areas = stmt
        .query_map([], |row| {
            Ok(Area {
                id: row.get(0)?,
                name: row.get(1)?,
                table_count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(areas)
}

pub fn add_area(conn: &Connection, name: &str, table_count: i64) -> Result<i64> {
    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM areas",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    conn.execute(
        "INSERT INTO areas (name, table_count, sort_order)
         VALUES (?1, ?2, ?3)",
        params![name, table_count, next_order],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_area(conn: &Connection, id: i64, name: &str, table_count: i64) -> Result<()> {
    conn.execute(
        "UPDATE areas SET name = ?1, table_count = ?2 WHERE id = ?3",
        params![name, table_count, id],
    )?;
    Ok(())
}

pub fn delete_area(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM areas WHERE id = ?1", [id])?;
    Ok(())
}

/// Swap the sort_order of two areas by id.
pub fn swap_area_order(conn: &Connection, id1: i64, id2: i64) -> Result<()> {
    let s1: i64 = conn.query_row(
        "SELECT COALESCE(sort_order, 0) FROM areas WHERE id = ?1",
        [id1],
        |r| r.get(0),
    )?;
    let s2: i64 = conn.query_row(
        "SELECT COALESCE(sort_order, 0) FROM areas WHERE id = ?1",
        [id2],
        |r| r.get(0),
    )?;
    conn.execute(
        "UPDATE areas SET sort_order = ?1 WHERE id = ?2",
        params![s2, id1],
    )?;
    conn.execute(
        "UPDATE areas SET sort_order = ?1 WHERE id = ?2",
        params![s1, id2],
    )?;
    Ok(())
}
