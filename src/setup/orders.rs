use crate::{MainWindow, OrderLine};
use crate::db;
use slint::{ComponentHandle, Model, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn setup_order_persistence(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    order_lines: Rc<VecModel<OrderLine>>,
) {
    let open_orders: Rc<RefCell<HashMap<String, i64>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // OPEN TABLE
    let weak_window_open = window.as_weak();
    let weak_order_lines_open = Rc::downgrade(&order_lines);
    let conn_open = conn.clone();
    let open_orders_open = open_orders.clone();
    window.on_open_table(move |table_name: slint::SharedString| {
        let table = table_name.to_string();
        if let (Some(window), Some(order_model)) = (
            weak_window_open.upgrade(),
            weak_order_lines_open.upgrade(),
        ) {
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
                }
            }

            window.set_current_bill_table(table_name.clone());
            window.set_current_view(4);
        }
    });

    // SAVE ORDER (PRINT)
    let weak_window_save = window.as_weak();
    let weak_order_lines_save = Rc::downgrade(&order_lines);
    let conn_save = conn.clone();
    let open_orders_save = open_orders.clone();
    window.on_save_order(move || {
        if let (Some(window), Some(order_model)) = (
            weak_window_save.upgrade(),
            weak_order_lines_save.upgrade(),
        ) {
            let table = window.get_current_bill_table().to_string();
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

            let order_id = if let Some(id) = open_orders_save.borrow().get(&table) {
                *id
            } else {
                let new_id = db::orders::create_order(&conn_save, &table).unwrap();
                open_orders_save.borrow_mut().insert(table.clone(), new_id);
                new_id
            };

            db::orders::clear_order_items(&conn_save, order_id).unwrap();
            for i in 0..order_model.row_count() {
                let line = order_model.row_data(i).unwrap();
                db::orders::add_order_item(
                    &conn_save,
                    order_id,
                    &line.name,
                    line.price as f64,
                    line.quantity,
                )
                .unwrap();
            }
            db::orders::update_order_totals(
                &conn_save,
                order_id,
                subtotal,
                gst,
                packaging,
                delivery,
                discount,
                total,
            )
            .unwrap();
        }
    });

    // SAVE & CLOSE ORDER
    let weak_window_save_close = window.as_weak();
    let weak_order_lines_save_close = Rc::downgrade(&order_lines);
    let conn_save_close = conn.clone();
    let open_orders_save_close = open_orders.clone();
    window.on_save_and_close_order(move || {
        if let (Some(window), Some(order_model)) = (
            weak_window_save_close.upgrade(),
            weak_order_lines_save_close.upgrade(),
        ) {
            let table = window.get_current_bill_table().to_string();
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

            let order_id = if let Some(id) = open_orders_save_close.borrow().get(&table) {
                *id
            } else {
                let new_id = db::orders::create_order(&conn_save_close, &table).unwrap();
                open_orders_save_close.borrow_mut().insert(table.clone(), new_id);
                new_id
            };

            db::orders::clear_order_items(&conn_save_close, order_id).unwrap();
            for i in 0..order_model.row_count() {
                let line = order_model.row_data(i).unwrap();
                db::orders::add_order_item(
                    &conn_save_close,
                    order_id,
                    &line.name,
                    line.price as f64,
                    line.quantity,
                )
                .unwrap();
            }
            db::orders::update_order_totals(
                &conn_save_close,
                order_id,
                subtotal,
                gst,
                packaging,
                delivery,
                discount,
                total,
            )
            .unwrap();

            db::orders::close_order(&conn_save_close, order_id).unwrap();
            open_orders_save_close.borrow_mut().remove(&table);

            order_model.set_vec(Vec::<OrderLine>::new());
            window.set_current_view(0);
        }
    });

    // DASHBOARD (SAVE OPEN & GO BACK)
    let weak_window_back = window.as_weak();
    let weak_order_lines_back = Rc::downgrade(&order_lines);
    let conn_back = conn.clone();
    let open_orders_back = open_orders.clone();
    window.on_close_bill(move || {
        if let (Some(window), Some(order_model)) = (
            weak_window_back.upgrade(),
            weak_order_lines_back.upgrade(),
        ) {
            let table = window.get_current_bill_table().to_string();
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

            let order_id = if let Some(id) = open_orders_back.borrow().get(&table) {
                *id
            } else {
                let new_id = db::orders::create_order(&conn_back, &table).unwrap();
                open_orders_back.borrow_mut().insert(table.clone(), new_id);
                new_id
            };

            db::orders::clear_order_items(&conn_back, order_id).unwrap();
            for i in 0..order_model.row_count() {
                let line = order_model.row_data(i).unwrap();
                db::orders::add_order_item(
                    &conn_back,
                    order_id,
                    &line.name,
                    line.price as f64,
                    line.quantity,
                )
                .unwrap();
            }
            db::orders::update_order_totals(
                &conn_back,
                order_id,
                subtotal,
                gst,
                packaging,
                delivery,
                discount,
                total,
            )
            .unwrap();

            order_model.set_vec(Vec::<OrderLine>::new());
            window.set_current_view(0);
        }
    });
}
