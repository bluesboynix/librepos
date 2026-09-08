pub mod menu;

use rusqlite::Connection;

pub fn init_db(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS menu_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            code TEXT UNIQUE,
            price REAL NOT NULL,
            category TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS ingredients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            unit TEXT,
            unit_price REAL
        );
        CREATE TABLE IF NOT EXISTS menu_item_ingredients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            menu_item_id INTEGER NOT NULL,
            ingredient_id INTEGER NOT NULL,
            quantity REAL,
            unit TEXT,
            estimated_cost REAL,
            FOREIGN KEY (menu_item_id) REFERENCES menu_items(id) ON DELETE CASCADE,
            FOREIGN KEY (ingredient_id) REFERENCES ingredients(id)
        );
        "
    )?;
    Ok(conn)
}
