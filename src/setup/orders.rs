use crate::{MainWindow, OrderLine, TableArea};
use crate::db;
use crate::setup::areas::rebuild_areas;
use slint::{ComponentHandle, Model, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Get the current bill totals and customer fields from the window.
fn read_bill_state(window: &MainWindow) -> (f64, f64, f64, f64, f64, f64, String, String, String) {
    let subtotal = window.get_bill_subtotal() as f64;
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
    (
        subtotal,
        gst,
        packaging,
        delivery,
        discount,
        total,
        customer_name,
        customer_phone,
        customer_address,
    )
}

/// Persist the current order items and totals to the database.
fn persist_order(
    conn: &rusqlite::Connection,
    order_id: i64,
    order_model: &Rc<VecModel<OrderLine>>,
    window: &MainWindow,
) {
    let (
        subtotal,
        gst,
        packaging,
        delivery,
        discount,
        total,
        customer_name,
        customer_phone,
        customer_address,
    ) = read_bill_state(window);

    db::orders::clear_order_items(conn, order_id).unwrap();
    for i in 0..order_model.row_count() {
        let line = order_model.row_data(i).unwrap();
        db::orders::add_order_item(
            conn,
            order_id,
            &line.name,
            line.price as f64,
            line.quantity,
        )
        .unwrap();
    }

    db::orders::update_order_totals(
        conn,
        order_id,
        subtotal,
        gst,
        packaging,
        delivery,
        discount,
        total,
        &customer_name,
        &customer_phone,
        &customer_address,
    )
    .unwrap();
}

/// Get or create the order id for a given table.
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
        persist_order(&conn_save, order_id, &order_model, &window);
        rebuild_areas(&conn_save, &areas_save);
    });

    // ==================== SAVE & CLOSE ORDER ====================
    let weak_window_save_close = window.as_weak();
    let weak_order_lines_save_close = Rc::downgrade(&order_lines);
    let conn_save_close = conn.clone();
    let open_orders_save_close = open_orders.clone();
    let areas_save_close = areas.clone();
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

        persist_order(&conn_save_close, order_id, &order_model, &window);
        db::orders::close_order(&conn_save_close, order_id).unwrap();
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
    window.on_close_bill(move || {
        let (Some(window), Some(order_model)) = (
            weak_window_back.upgrade(),
            weak_order_lines_back.upgrade(),
        ) else {
            return;
        };

        // Skip saving if the order is empty
        let subtotal = window.get_bill_subtotal() as f64;
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

        if order_model.row_count() == 0
            && subtotal == 0.0
            && gst == 0.0
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

        persist_order(&conn_back, order_id, &order_model, &window);
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
