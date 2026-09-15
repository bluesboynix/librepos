use crate::{MainWindow, MenuItem, OrderLine};
use crate::db;
use slint::{Model, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Read the current bill totals and customer fields from the window.
#[allow(clippy::type_complexity)]
pub fn read_bill_state(
    window: &MainWindow,
) -> (f64, f64, f64, f64, f64, f64, String, String, String) {
    let subtotal = window.get_bill_subtotal() as f64;
    let gst = window.get_bill_gst() as f64;
    let packaging = window.get_packaging_text().parse::<f64>().unwrap_or(0.0);
    let delivery = window.get_delivery_text().parse::<f64>().unwrap_or(0.0);
    let discount = window.get_discount_text().parse::<f64>().unwrap_or(0.0);
    let total = window.get_bill_total() as f64;
    let customer_name = window.get_customer_name().to_string();
    let customer_phone = window.get_customer_phone().to_string();
    let customer_address = window.get_customer_address().to_string();
    (
        subtotal, gst, packaging, delivery, discount, total,
        customer_name, customer_phone, customer_address,
    )
}

/// Snapshot all order items (name, category, prices, note).
pub fn snapshot_items(
    conn: &rusqlite::Connection,
    order_id: i64,
    order_model: &Rc<VecModel<OrderLine>>,
    menu_items: &Rc<VecModel<MenuItem>>,
) {
    let _ = db::orders::clear_order_items(conn, order_id);

    for i in 0..order_model.row_count() {
        let line = order_model.row_data(i).unwrap();

        let mut meta: Option<(i64, String, String, String, f64)> = None;
        for j in 0..menu_items.row_count() {
            let mi = menu_items.row_data(j).unwrap();
            if mi.name == line.name {
                let cost = db::orders::get_menu_item_cost(conn, mi.id as i64)
                    .unwrap_or(0.0);
                meta = Some((
                    mi.id as i64,
                    mi.name.to_string(),
                    mi.code.to_string(),
                    mi.category.to_string(),
                    cost,
                ));
                break;
            }
        }

        let (menu_item_id, name, code, category, cost_price) = match meta {
            Some((id, n, c, cat, cp)) => (Some(id), n, c, cat, cp),
            None => (
                None,
                line.name.to_string(),
                String::new(),
                String::new(),
                0.0,
            ),
        };

        let unit_price = line.price as f64;
        let line_total = unit_price * line.quantity as f64;

        let _ = db::orders::add_order_item_snapshot(
            conn,
            order_id,
            menu_item_id,
            &name,
            &code,
            &category,
            unit_price,
            cost_price,
            line.quantity,
            line_total,
            &line.note,
        );
    }
}

/// Persist the current bill state (items + totals + payments) to a given order.
pub fn save_bill_to_order(
    conn: &rusqlite::Connection,
    order_id: i64,
    order_model: &Rc<VecModel<OrderLine>>,
    menu_items: &Rc<VecModel<MenuItem>>,
    window: &MainWindow,
) {
    snapshot_items(conn, order_id, order_model, menu_items);

    let (
        subtotal, gst, packaging, delivery, discount, total,
        customer_name, customer_phone, customer_address,
    ) = read_bill_state(window);

    let cgst = window.get_cgst_text().parse::<f64>().unwrap_or(0.0);
    let sgst = window.get_sgst_text().parse::<f64>().unwrap_or(0.0);

    let _ = db::orders::update_order_totals(
        conn,
        order_id,
        subtotal,
        cgst,
        sgst,
        gst,
        packaging,
        delivery,
        discount,
        total,
        &customer_name,
        &customer_phone,
        &customer_address,
    );

    save_payments(conn, order_id, window);
}

/// Save payments based on the selected payment mode.
pub fn save_payments(
    conn: &rusqlite::Connection,
    order_id: i64,
    window: &MainWindow,
) {
    let _ = db::payments::clear_payments(conn, order_id);
    let mode = window.get_payment_mode();
    let total = window.get_bill_total() as f64;

    match mode {
        1 => {
            let _ = db::payments::add_payment(conn, order_id, "cash", total, None);
        }
        2 => {
            let _ = db::payments::add_payment(conn, order_id, "upi", total, None);
        }
        3 => {
            let _ = db::payments::add_payment(conn, order_id, "card", total, None);
        }
        4 => {
            let cash = window.get_split_cash_text().parse::<f64>().unwrap_or(0.0);
            let upi = window.get_split_upi_text().parse::<f64>().unwrap_or(0.0);
            let card = window.get_split_card_text().parse::<f64>().unwrap_or(0.0);
            if cash > 0.0 {
                let _ = db::payments::add_payment(conn, order_id, "cash", cash, None);
            }
            if upi > 0.0 {
                let _ = db::payments::add_payment(conn, order_id, "upi", upi, None);
            }
            if card > 0.0 {
                let _ = db::payments::add_payment(conn, order_id, "card", card, None);
            }
        }
        5 => {
            let _ = db::payments::add_payment(conn, order_id, "due", 0.0, None);
        }
        _ => {}
    }
}

/// Map a payment mode to a status string.
pub fn payment_status_for_mode(mode: i32) -> &'static str {
    match mode {
        1 | 2 | 3 | 4 => "paid",
        5 => "unpaid",
        _ => "unpaid",
    }
}

/// Get the cached open order id for a table, or create a new one.
pub fn get_or_create_order_id(
    conn: &rusqlite::Connection,
    open_orders: &Rc<RefCell<HashMap<String, i64>>>,
    table: &str,
) -> i64 {
    if let Some(id) = open_orders.borrow().get(table) {
        *id
    } else {
        let new_id = db::orders::create_order(conn, table).unwrap();
        open_orders.borrow_mut().insert(table.to_string(), new_id);
        new_id
    }
}
