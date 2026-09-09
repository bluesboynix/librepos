use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub table_count: i64,
}

pub fn get_all_areas(conn: &Connection) -> Result<Vec<Area>> {
    let mut stmt = conn.prepare("SELECT id, name, table_count FROM areas ORDER BY name")?;
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
    conn.execute(
        "INSERT INTO areas (name, table_count) VALUES (?1, ?2)",
        params![name, table_count],
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
