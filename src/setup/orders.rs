use crate::{MainWindow, MenuItem, OrderLine, TableArea};
use crate::db;
use crate::setup::areas::rebuild_areas;
use slint::{ComponentHandle, Model, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Persist the current bill state (items + totals + payments) to a given order.
fn save_bill_to_order(
    conn: &rusqlite::Connection,
    order_id: i64,
    order_model: &Rc<VecModel<OrderLine>>,
    menu_items: &Rc<VecModel<MenuItem>>,
    window: &MainWindow,
) {
    snapshot_items(conn, order_id, order_model, menu_items);

    let subtotal = window.get_bill_subtotal() as f64;
    let cgst = window.get_cgst_text().parse::<f64>().unwrap_or(0.0);
    let sgst = window.get_sgst_text().parse::<f64>().unwrap_or(0.0);
    let gst = window.get_bill_gst() as f64;
    let packaging = window
        .get_packaging_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let delivery = window
        .get_delivery_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let discount = window
        .get_discount_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let total = window.get_bill_total() as f64;
    let customer_name = window.get_customer_name().to_string();
    let customer_phone = window.get_customer_phone().to_string();
    let customer_address = window.get_customer_address().to_string();

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

/// Build snapshot lines for all order items.
fn snapshot_items(
    conn: &rusqlite::Connection,
    order_id: i64,
    order_model: &Rc<VecModel<OrderLine>>,
    menu_items: &Rc<VecModel<MenuItem>>,
) {
    let _ = db::orders::clear_order_items(conn, order_id);

    for i in 0..order_model.row_count() {
        let line = order_model.row_data(i).unwrap();

        // Find the original menu item to snapshot its metadata
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

        let (menu_item_id, name, code, category, cost_price) =
            match meta {
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
        );
    }
}

/// Save payments based on the selected payment mode.
fn save_payments(
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
            let cash = window
                .get_split_cash_text()
                .parse::<f64>()
                .unwrap_or(0.0);
            let upi = window
                .get_split_upi_text()
                .parse::<f64>()
                .unwrap_or(0.0);
            let card = window
                .get_split_card_text()
                .parse::<f64>()
                .unwrap_or(0.0);
            if cash > 0.0 {
                let _ = db::payments::add_payment(
                    conn, order_id, "cash", cash, None,
                );
            }
            if upi > 0.0 {
                let _ = db::payments::add_payment(
                    conn, order_id, "upi", upi, None,
                );
            }
            if card > 0.0 {
                let _ = db::payments::add_payment(
                    conn, order_id, "card", card, None,
                );
            }
        }
        5 => {
            let _ = db::payments::add_payment(conn, order_id, "due", 0.0, None);
        }
        _ => {}
    }
}

fn payment_status_for_mode(mode: i32) -> &'static str {
    match mode {
        1 | 2 | 3 | 4 => "paid",
        5 => "unpaid",
        _ => "unpaid",
    }
}

/// Get the cached open order id for a table, or create a new one.
fn get_or_create_order_id(
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

pub fn setup_order_persistence(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    order_lines: Rc<VecModel<OrderLine>>,
    menu_items: Rc<VecModel<MenuItem>>,
    areas: Rc<VecModel<TableArea>>,
) {
    let open_orders: Rc<RefCell<HashMap<String, i64>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // ==================== OPEN TABLE ====================
    let weak_window_open = window.as_weak();
    let weak_order_lines_open = Rc::downgrade(&order_lines);
    let conn_open = conn.clone();
    let open_orders_open = open_orders.clone();
    window.on_open_table(move |table_name: slint::SharedString| {
        let table = table_name.to_string();
        let (Some(window), Some(order_model)) = (
            weak_window_open.upgrade(),
            weak_order_lines_open.upgrade(),
        ) else {
            return;
        };

        order_model.set_vec(Vec::<OrderLine>::new());

        let order_id = if let Some(id) = open_orders_open.borrow().get(&table) {
            Some(*id)
        } else {
            db::orders::get_open_order_by_table(&conn_open, &table)
                .ok()
                .flatten()
                .map(|o| o.id)
        };

        match order_id {
            Some(id) => {
                if let Ok(items) = db::orders::get_order_items(&conn_open, id) {
                    let lines: Vec<OrderLine> = items
                        .into_iter()
                        .map(|item| OrderLine {
                            name: item.name.into(),
                            price: item.price as f32,
                            quantity: item.quantity,
                        })
                        .collect();
                    order_model.set_vec(lines);
                }
                if let Ok(Some(order)) =
                    db::orders::get_open_order_by_table(&conn_open, &table)
                {
                    window.set_bill_subtotal(order.subtotal as f32);
                    window.set_bill_gst(order.gst as f32);
                    window.set_bill_total(order.total as f32);
                    window.set_packaging_text(
                        format!("{:.2}", order.packaging).into(),
                    );
                    window.set_delivery_text(
                        format!("{:.2}", order.delivery).into(),
                    );
                    window.set_discount_text(
                        format!("{:.2}", order.discount).into(),
                    );
                    window.set_customer_name(order.customer_name.into());
                    window.set_customer_phone(order.customer_phone.into());
                    window.set_customer_address(order.customer_address.into());
                }
                open_orders_open.borrow_mut().insert(table.clone(), id);
            }
            None => {
                window.set_bill_subtotal(0.0);
                window.set_bill_gst(0.0);
                window.set_bill_total(0.0);
                window.set_packaging_text("0.00".into());
                window.set_delivery_text("0.00".into());
                window.set_discount_text("0.00".into());
                window.set_customer_name("".into());
                window.set_customer_phone("".into());
                window.set_customer_address("".into());
            }
        }

        window.set_current_bill_table(table_name.clone());
        window.set_current_view(4);
    });

    // ==================== SAVE ORDER (PRINT) ====================
    let weak_window_save = window.as_weak();
    let weak_order_lines_save = Rc::downgrade(&order_lines);
    let conn_save = conn.clone();
    let open_orders_save = open_orders.clone();
    let areas_save = areas.clone();
    let menu_items_save = menu_items.clone();
    window.on_save_order(move || {
        let (Some(window), Some(order_model)) = (
            weak_window_save.upgrade(),
            weak_order_lines_save.upgrade(),
        ) else {
            return;
        };

        let table = window.get_current_bill_table().to_string();
        let order_id =
            get_or_create_order_id(&conn_save, &open_orders_save, &table);

        save_bill_to_order(
            &conn_save,
            order_id,
            &order_model,
            &menu_items_save,
            &window,
        );
        rebuild_areas(&conn_save, &areas_save);
    });

    // ==================== SAVE & CLOSE ORDER ====================
    let weak_window_save_close = window.as_weak();
    let weak_order_lines_save_close = Rc::downgrade(&order_lines);
    let conn_save_close = conn.clone();
    let open_orders_save_close = open_orders.clone();
    let areas_save_close = areas.clone();
    let menu_items_close = menu_items.clone();
    window.on_save_and_close_order(move || {
        let (Some(window), Some(order_model)) = (
            weak_window_save_close.upgrade(),
            weak_order_lines_save_close.upgrade(),
        ) else {
            return;
        };

        let table = window.get_current_bill_table().to_string();
        let order_id = get_or_create_order_id(
            &conn_save_close,
            &open_orders_save_close,
            &table,
        );

        save_bill_to_order(
            &conn_save_close,
            order_id,
            &order_model,
            &menu_items_close,
            &window,
        );

        // Generate bill number and close the order
        let bill_no = db::sequences::next_bill_number(&conn_save_close)
            .unwrap_or_default();
        let area_name = table.split(' ').next().unwrap_or("").to_string();
        let payment_status =
            payment_status_for_mode(window.get_payment_mode());

        let _ = db::orders::close_order(
            &conn_save_close,
            order_id,
            &bill_no,
            &area_name,
            payment_status,
        );

        open_orders_save_close.borrow_mut().remove(&table);
        rebuild_areas(&conn_save_close, &areas_save_close);

        order_model.set_vec(Vec::<OrderLine>::new());
        window.set_current_view(0);
    });

    // ==================== DASHBOARD (SAVE OPEN & GO BACK) ====================
    let weak_window_back = window.as_weak();
    let weak_order_lines_back = Rc::downgrade(&order_lines);
    let conn_back = conn.clone();
    let open_orders_back = open_orders.clone();
    let areas_back = areas.clone();
    let menu_items_back = menu_items.clone();
    window.on_close_bill(move || {
        let (Some(window), Some(order_model)) = (
            weak_window_back.upgrade(),
            weak_order_lines_back.upgrade(),
        ) else {
            return;
        };

        // Skip saving if the order is empty
        let subtotal = window.get_bill_subtotal() as f64;
        let packaging = window
            .get_packaging_text()
            .parse::<f64>()
            .unwrap_or(0.0);
        let delivery = window
            .get_delivery_text()
            .parse::<f64>()
            .unwrap_or(0.0);
        let discount = window
            .get_discount_text()
            .parse::<f64>()
            .unwrap_or(0.0);

        if order_model.row_count() == 0
            && subtotal == 0.0
            && packaging == 0.0
            && delivery == 0.0
            && discount == 0.0
        {
            order_model.set_vec(Vec::<OrderLine>::new());
            window.set_current_view(0);
            return;
        }

        let table = window.get_current_bill_table().to_string();
        let order_id =
            get_or_create_order_id(&conn_back, &open_orders_back, &table);

        save_bill_to_order(
            &conn_back,
            order_id,
            &order_model,
            &menu_items_back,
            &window,
        );

        rebuild_areas(&conn_back, &areas_back);
        order_model.set_vec(Vec::<OrderLine>::new());
        window.set_current_view(0);
    });

    // ==================== MOVE TABLE ====================
    let weak_window_move = window.as_weak();
    let conn_move = conn.clone();
    let open_orders_move = open_orders.clone();
    let areas_move = areas.clone();
    window.on_move_table(move |new_table_name: slint::SharedString| {
        let Some(window) = weak_window_move.upgrade() else {
            return;
        };

        let new_table = new_table_name.to_string();
        let old_table = window.get_current_bill_table().to_string();
        if old_table == new_table || new_table.is_empty() {
            return;
        }

        let old_order_id = open_orders_move
            .borrow()
            .get(&old_table)
            .copied()
            .or_else(|| {
                db::orders::get_open_order_by_table(&conn_move, &old_table)
                    .ok()
                    .flatten()
                    .map(|o| o.id)
            });

        let new_order_id = open_orders_move
            .borrow()
            .get(&new_table)
            .copied()
            .or_else(|| {
                db::orders::get_open_order_by_table(&conn_move, &new_table)
                    .ok()
                    .flatten()
                    .map(|o| o.id)
            });

        // Don't overwrite an existing open order on the target table
        if new_order_id.is_some() {
            return;
        }

        if let Some(order_id) = old_order_id {
            let _ = db::orders::update_order_table(
                &conn_move,
                order_id,
                &new_table,
            );
            open_orders_move.borrow_mut().remove(&old_table);
            open_orders_move
                .borrow_mut()
                .insert(new_table.clone(), order_id);
        }

        window.set_current_bill_table(new_table_name.clone());
        rebuild_areas(&conn_move, &areas_move);
    });
}
