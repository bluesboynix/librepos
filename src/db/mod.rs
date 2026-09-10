pub mod areas;
pub mod menu;
pub mod orders;
pub mod payments;
pub mod sequences;
pub mod settings;
pub mod reports;

use rusqlite::Connection;

pub fn init_db(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;

    // 1. Create tables (no indexes here)
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS menu_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            code TEXT UNIQUE,
            price REAL NOT NULL,
            category TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS ingredients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            unit TEXT,
            unit_price REAL,
            stock REAL DEFAULT 0,
            low_stock_threshold REAL DEFAULT 0
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
            bill_no TEXT,
            table_name TEXT NOT NULL,
            area_name TEXT DEFAULT '',
            status TEXT DEFAULT 'open',
            payment_status TEXT DEFAULT 'unpaid',
            subtotal REAL DEFAULT 0,
            cgst REAL DEFAULT 0,
            sgst REAL DEFAULT 0,
            gst REAL DEFAULT 0,
            packaging REAL DEFAULT 0,
            delivery REAL DEFAULT 0,
            discount REAL DEFAULT 0,
            total REAL DEFAULT 0,
            customer_name TEXT DEFAULT '',
            customer_phone TEXT DEFAULT '',
            customer_address TEXT DEFAULT '',
            opened_at TEXT DEFAULT (datetime('now','localtime')),
            closed_at TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            menu_item_id INTEGER,
            name TEXT NOT NULL,
            code TEXT,
            category TEXT DEFAULT '',
            unit_price REAL DEFAULT 0,
            cost_price REAL DEFAULT 0,
            quantity INTEGER NOT NULL,
            line_total REAL DEFAULT 0,
            price REAL DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            method TEXT NOT NULL,
            amount REAL NOT NULL,
            reference TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS bill_sequences (
            day TEXT PRIMARY KEY,
            last_seq INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS stock_movements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ingredient_id INTEGER NOT NULL,
            change REAL NOT NULL,
            reason TEXT,
            order_id INTEGER,
            unit_cost REAL,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        ",
    )?;

    // 2. Safe migrations for existing DBs (ignore errors if column exists)
    let migrations = [
        "ALTER TABLE orders ADD COLUMN bill_no TEXT",
        "ALTER TABLE orders ADD COLUMN area_name TEXT DEFAULT ''",
        "ALTER TABLE orders ADD COLUMN payment_status TEXT DEFAULT 'unpaid'",
        "ALTER TABLE orders ADD COLUMN cgst REAL DEFAULT 0",
        "ALTER TABLE orders ADD COLUMN sgst REAL DEFAULT 0",
        "ALTER TABLE orders ADD COLUMN opened_at TEXT",
        "ALTER TABLE orders ADD COLUMN closed_at TEXT",
        "ALTER TABLE order_items ADD COLUMN menu_item_id INTEGER",
        "ALTER TABLE order_items ADD COLUMN code TEXT",
        "ALTER TABLE order_items ADD COLUMN category TEXT DEFAULT ''",
        "ALTER TABLE order_items ADD COLUMN unit_price REAL DEFAULT 0",
        "ALTER TABLE order_items ADD COLUMN cost_price REAL DEFAULT 0",
        "ALTER TABLE order_items ADD COLUMN line_total REAL DEFAULT 0",
        "ALTER TABLE ingredients ADD COLUMN stock REAL DEFAULT 0",
        "ALTER TABLE ingredients ADD COLUMN low_stock_threshold REAL DEFAULT 0",
    ];
    for sql in migrations.iter() {
        let _ = conn.execute(sql, []);
    }

    // 3. Create indexes (after migrations so columns exist)
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_orders_status        ON orders(status);
        CREATE INDEX IF NOT EXISTS idx_orders_closed_at     ON orders(closed_at);
        CREATE INDEX IF NOT EXISTS idx_orders_bill_no       ON orders(bill_no);
        CREATE INDEX IF NOT EXISTS idx_order_items_order    ON order_items(order_id);
        CREATE INDEX IF NOT EXISTS idx_order_items_item     ON order_items(menu_item_id);
        CREATE INDEX IF NOT EXISTS idx_order_items_category ON order_items(category);
        CREATE INDEX IF NOT EXISTS idx_payments_order       ON payments(order_id);
        CREATE INDEX IF NOT EXISTS idx_payments_method      ON payments(method);
        CREATE INDEX IF NOT EXISTS idx_payments_created     ON payments(created_at);
        CREATE INDEX IF NOT EXISTS idx_stock_mov_ing        ON stock_movements(ingredient_id);
        CREATE INDEX IF NOT EXISTS idx_stock_mov_created    ON stock_movements(created_at);
        ",
    )?;

    Ok(conn)
}
