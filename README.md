# LibrePOS

A lightweight, offline-first Point of Sale system for small restaurants and cafés.

Built with **Rust** and **Slint** (a native GUI toolkit). All data is stored locally in SQLite — no internet, no cloud, no subscription.

**Current release: `v0.4.0-alpha`**

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [First-Time Setup](#first-time-setup)
- [Daily Workflow](#daily-workflow)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Data & Backup](#data--backup)
- [Reports & Exports](#reports--exports)
- [Settings Reference](#settings-reference)
- [Known Limitations (Alpha)](#known-limitations-alpha)
- [Roadmap](#roadmap)
- [Reporting Issues](#reporting-issues)
- [License](#license)

---

## Features

### Core POS

- **Menu management** with categories, codes, prices
- **Ingredient tracking** per menu item (name, quantity, unit, cost)
- **Automatic cost & profit** calculation for every menu item
- **Per-table orders** — each table keeps its own open bill
- **Item-level notes** (e.g., "no onion", "extra spicy")
- **Quantity adjust / remove** on each line
- **Move table** — transfer an open bill to a different table
- **Customer details** per order (name, phone, address)

### Payments

- Five payment modes: **Cash**, **UPI**, **Card**, **Split**, **Due (unpaid)**
- Split payment across Cash / UPI / Card with individual amounts
- Payment mode confirmation dialog when closing without a selection
- Payment methods shown in the Bills list and bill detail

### Bill Lifecycle

- **Save & Close** — finalize the bill, generate a bill number
- **Save as open** (Print Bill) — persist the order without closing
- **Dashboard** — save the open order and return to the table grid
- **Void** — cancel a closed bill with a mandatory reason
- **Revive** — restore a voided bill back to closed
- **Edit** — reopen a closed bill for modification
  - Preserves the same bill number
  - Tracks every edit in an edit history

### Reports

- **Summary** — total sales, order count, average order, daily bar chart
- **Top Items** — best-selling items with category filter
- **Payments** — breakdown by payment method
- **Daily Sales** — day-by-day list for the selected range
- **Bills** — searchable list of all bills (by bill no, table, area, customer, phone)
  - Show/hide voided bills
  - Drill into any bill for a full detail view
- Date range: **Today**, **This Week**, **This Month**, or a custom range

### Exports

- **CSV export** — 3 files per range (`bills_*.csv`, `items_*.csv`, `payments_*.csv`)
- **PDF bill export** — thermal-receipt-style PDF (80mm width) for any closed bill
  - Includes store header, FSSAI number, items, totals, payments, footer
  - Ready to send via email or WhatsApp

### Settings

- **Store Information** — name, address, phone, FSSAI licence, footer message
- **Table Area Configuration** — add, rename, reorder, delete areas; set table count
- **GST Configuration** — CGST + SGST percentages (default 0%)
- **Currency Symbol** — configurable (default ₹)
- **Keyboard Shortcuts** — view and change all shortcuts
- **All settings persist** in the database

### Interface

- Native, responsive desktop UI
- Dark theme
- Sidebar navigation
- Status bar with version indicator
- Home shortcut — click "LibrePOS" in the title bar
- Full keyboard shortcut support

---

## Requirements

- **Rust** 1.75 or later ([install via rustup](https://rustup.rs))
- **Linux** (tested), **macOS**, or **Windows**
- **SQLite** — bundled automatically, no separate installation needed

---

## Installation

### From source

```bash
# Clone the repository
git clone <your-repo-url>
cd LibrePOS

# Check out the alpha release
git checkout v0.4.0-alpha

# Build and run in development mode
cargo run

# Or build an optimized release binary
cargo build --release
./target/release/librepos
```

The first build will download dependencies and may take a few minutes.

## Pre-built binary
If a release tarball is provided:
```bash
tar xzf librepos-v0.4.0-alpha-linux.tar.gz
cd librepos-v0.4.0-alpha
./librepos
```

## First-Time Setup
On first launch, the app starts empty. Before taking orders:

### 1. Store Information
Sidebar → Settings → Store Information

Fill in:

| Field	| Purpose |
| ----- | ------- |
| Name	|Appears at top of printed bills |
| Address |	Appears under the store name |
| Phone	| Contact number on the bill |
| FSSAI	| Food licence number (optional) |
| Footer	| Closing message (e.g. "Thank you!") |

### 2. Table Areas
* Sidebar → Settings → Table Area Configuration
* Click + Add new Area to create a section (e.g. "Indoor", "Outdoor", "Delivery")
* Set the number of tables in each area
* Use ↑ / ↓ to reorder (this order is used on the Dashboard)
* Click x to delete an area

### 3. GST (optional)
Sidebar → Settings → GST Configuration
* Set CGST and SGST percentages (e.g. 2.5 + 2.5 for 5% total)
* Leave at 0 if you don't want tax applied

### 4. Currency (optional)
Sidebar → Settings → Currency
* Change the symbol (default ₹)

### 5. Menu
Sidebar → Menu → + Add Item

For each item:

Name, Price, Category are required
* Code is optional (e.g. "B01")
* Add ingredients to track cost and calculate profit
* Click Save

Categories are derived automatically — type any new category name on an item and it appears as a tab in the Menu view.

## Daily Workflow

### Taking an order

1. Dashboard — click a table card

2. The Bill view opens with three panels:

    * Left: Menu (with category tabs and search)
      
    * Center: Current order lines
      
    * Right: Bill summary, payment, customer, actions

3. Click a menu item (or its + button) to add it to the order

4. Adjust quantity with −/+, or remove with X

5. Add item notes with the N button

6. Fill in customer details if needed

7. Select a payment mode

8. Click Save & Close

### If you forget the payment mode
A dialog appears asking if the bill should be marked unpaid. Choose:

  * Yes → closes as unpaid (find it later in Reports → Bills and edit to settle)

  * No → returns to the bill for you to pick a mode

### Viewing and managing past bills
1. Sidebar → Reports → Bills tab

2. Search by bill number, table, area, customer name, or phone

3. Click a row to open the bill detail

4. From there you can:

    * Preview the thermal receipt
    * Save as PDF
    * Edit (reopens the order)
    * Void with a reason
    * Revive a voided bill

### End of day
1. Reports → Summary → set range to Today

2. Check total sales, order count, payment breakdown

3. Reports → Bills → Export CSV for accounting

4. Back up librepos.db (see Data & Backup)

## Keyboard Shortcuts
All shortcuts can be changed in Settings → Keyboard Shortcuts.

| Shortcut | Action |
| -------- | ------ |
| Alt+0	| Go to Dashboard |
| Alt+1	| Go to Reports |
| Alt+2	| Go to Menu |
| Alt+3	| Go to Settings |
| Alt+F	| Focus the search box (current view) |
| Esc	| Close dialog / return to Dashboard |

### To change a shortcut:

  1. Settings → Keyboard Shortcuts
  2. Click Change next to an action
  3. Press the new key combination
  4. Click Save

Use Reset inside the change dialog to restore the default.

## Data & Backup
### Where is my data?
Everything is stored in a single SQLite file:

```text
librepos.db
```

This sits in the current working directory (usually the project root when you run cargo run, or the folder where you launch the binary).

## Backing up
Simply copy librepos.db to a safe location:

```bash
cp librepos.db "backup-$(date +%Y%m%d).db"
```

Do this regularly — daily is recommended. The file is fully portable; copy it to another machine and the entire state (menu, orders, reports) comes along.

## Exports
CSV and PDF exports are written to an exports/ folder next to the database.

```text
exports/
├── bills_2026-09-01_2026-09-17.csv
├── items_2026-09-01_2026-09-17.csv
├── payments_2026-09-01_2026-09-17.csv
└── bill_20260917-0001_1758000000.pdf
```

## Reports & Exports
### CSV export
Reports → Export CSV button (top right).
Writes 3 files for the currently selected date range:

| File | Content |
| ---- | ------- |
| bills_*.csv | One row per bill — totals, tax, customer, timestamps |
| items_*.csv | One row per item ordered — item, qty, price, note |
| payments_*.csv | One row per payment method on each bill |

Open in LibreOffice Calc, Excel, or Google Sheets for analysis.

## PDF bill export
Reports → Bills → click a bill → Preview → Save as PDF.

* The PDF is a thermal-receipt style layout:
* 80 mm wide (standard thermal printer width)
* Courier font (monospace, clean)
* Store header (name, address, phone, FSSAI)
* Bill metadata (bill no, table, area, closed time)
* Customer details (if present)
* Items with quantities and totals
* Notes shown as * note under each item
* Totals with conditional CGST/SGST/Packaging/Delivery/Discount
* Payment breakdown
* Footer message

Note: the in-app preview may clip very long item lists. The PDF export always contains every item.

## Settings Reference
### Store Information
Persisted as store.* keys in the settings table.

### Table Area Configuration
Stored in the areas table with a sort_order column that determines the display order on the Dashboard.

* Add area → new row at the bottom
* Reorder → swaps sort_order values
* Delete → removes the area and (via cascade) any tables that reference it

### GST Configuration
Stored as cgst and sgst in the settings table. Total GST = CGST + SGST.

The rate is applied to every new bill view. Existing closed bills keep their snapshot values.

### Currency
Stored as currency. Used for display everywhere in the app.

### Keyboard Shortcuts
Stored as shortcut.<action> keys in the settings table.

## Known Limitations (Alpha)
This is an alpha release intended for testing in real restaurant environments. Please be aware of the following:

* Bill preview scroll — long item lists may be clipped in the on-screen preview. Use the PDF export for a complete receipt.
* No printer integration — receipts are exported as PDF files. You'll need to print them via your operating system's print dialog, or send them to customers via WhatsApp / email.
* Single user only — no login, no roles, no access restrictions.
* No cloud sync — each machine has its own database. To share data across machines, copy librepos.db manually.
* No refund workflow — voided bills are excluded from reports, but there's no separate "refund" concept. Use Void for cancellations.
* No stock tracking — ingredients are used for cost calculation only. Inventory levels are not decremented on sales. Track stock externally.
* No table reservations or merging — tables can be moved but not merged.
* Reports are read-only — no way to edit past bills except through the Edit action on individual bills.
* No automatic backups — back up librepos.db manually.
* Alpha UI — some spacing and layout details may still change.
* Please report anything unexpected — that's exactly what an alpha is for.

Please report anything unexpected — that's exactly what an alpha is for.

## Roadmap
* Post-alpha features under consideration (in no particular order):
* Printer integration — direct thermal printer support via ESC/POS
* First-run setup wizard — guided initial configuration
* Automated backups — periodic snapshots of the database
* WhatsApp share — format a bill as text for easy sharing
* Multi-user support — PIN-based roles (cashier, manager, admin)
* Table merge — combine two open orders
* Bulk menu import — import items from a CSV
* Light theme — selectable theme in Settings
* 58mm receipt option — for smaller thermal printers
* Stock tracking — optional ingredient inventory

## Reporting Issues
Please open an issue on the project repository with:

1. Steps to reproduce — what you did, what you expected, what happened

2. Version — the small label at the top right of the app (e.g. v0.4.0-alpha)

3. Environment — OS, screen resolution, whether it happens every time

4. Logs / screenshots — terminal output, screenshots of the UI

5. Sample data — if the issue involves specific menu items or bills, describe them

For crashes, run the app from a terminal to capture output:

```bash
cargo run 2> librepos-error.log
```
## License
GPLv3

## Acknowledgements
* Slint — https://slint.dev
* Rusqlite — https://github.com/rusqlite/rusqlite
* printpdf — https://github.com/fschutt/printpdf

## Author
Bluesboynix

---
