pub mod menu;
pub mod areas;
pub mod orders;
pub mod settings;

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
        CREATE TABLE IF NOT EXISTS areas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            table_count INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_name TEXT NOT NULL,
            status TEXT DEFAULT 'open',
            subtotal REAL DEFAULT 0,
            gst REAL DEFAULT 0,
            packaging REAL DEFAULT 0,
            delivery REAL DEFAULT 0,
            discount REAL DEFAULT 0,
            total REAL DEFAULT 0,
            customer_name TEXT DEFAULT '',
            customer_phone TEXT DEFAULT '',
            customer_address TEXT DEFAULT '',
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            price REAL NOT NULL,
            quantity INTEGER NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        "
    )?;

    // Safe migrations for existing DBs
    let _ = conn.execute("ALTER TABLE orders ADD COLUMN customer_name TEXT DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE orders ADD COLUMN customer_phone TEXT DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE orders ADD COLUMN customer_address TEXT DEFAULT ''", []);
    
    Ok(conn)
}
