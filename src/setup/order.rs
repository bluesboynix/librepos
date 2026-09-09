use crate::{MainWindow, OrderLine};
use crate::utils::update_bill_totals;
use slint::{ComponentHandle, Model, VecModel};
use std::rc::Rc;

pub fn setup_order_callbacks(
    window: &MainWindow,
    order_lines: Rc<VecModel<OrderLine>>,
) {
    // Add order line
    let weak_order_lines_add = Rc::downgrade(&order_lines);
    let weak_window_order_add = window.as_weak();
    window.on_add_order_line(move |name, price| {
        if let Some(model) = weak_order_lines_add.upgrade() {
            let mut found = false;
            for i in 0..model.row_count() {
                let mut line = model.row_data(i).unwrap();
                if line.name == name {
                    line.quantity += 1;
                    model.set_row_data(i, line);
                    found = true;
                    break;
                }
            }
            if !found {
                model.push(OrderLine {
                    name,
                    price,
                    quantity: 1,
                });
            }
            if let Some(window) = weak_window_order_add.upgrade() {
                update_bill_totals(&window);
            }
        }
    });

    // Update order line quantity
    let weak_order_lines_update = Rc::downgrade(&order_lines);
    let weak_window_order_update = window.as_weak();
    window.on_update_order_line_quantity(move |index, qty| {
        if let Some(model) = weak_order_lines_update.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                if qty > 0 {
                    let mut line = model.row_data(index as usize).unwrap();
                    line.quantity = qty;
                    model.set_row_data(index as usize, line);
                } else {
                    model.remove(index as usize);
                }
                if let Some(window) = weak_window_order_update.upgrade() {
                    update_bill_totals(&window);
                }
            }
        }
    });

    // Remove order line
    let weak_order_lines_remove = Rc::downgrade(&order_lines);
    let weak_window_order_remove = window.as_weak();
    window.on_remove_order_line(move |index| {
        if let Some(model) = weak_order_lines_remove.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                model.remove(index as usize);
                if let Some(window) = weak_window_order_remove.upgrade() {
                    update_bill_totals(&window);
                }
            }
        }
    });

    // Recalculate bill from charge inputs
    let weak_window_recalc = window.as_weak();
    window.on_recalculate_bill(move || {
        if let Some(window) = weak_window_recalc.upgrade() {
            update_bill_totals(&window);
        }
    });
}
