use crate::{MainWindow, OrderLine};
use crate::utils::update_bill_totals;
use slint::{ComponentHandle, Model, VecModel};
use std::rc::Rc;

pub fn setup_order_callbacks(
    window: &MainWindow,
    order_lines: Rc<VecModel<OrderLine>>,
) {
    // ==================== ADD ORDER LINE ====================
    let weak_order_lines_add = Rc::downgrade(&order_lines);
    let weak_window_order_add = window.as_weak();
    window.on_add_order_line(move |name, price| {
        if let Some(model) = weak_order_lines_add.upgrade() {
            let mut found = false;
            for i in 0..model.row_count() {
                let mut line = model.row_data(i).unwrap();
                if line.name == name && line.note.is_empty() {
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
                    note: "".into(),
                });
            }
            if let Some(window) = weak_window_order_add.upgrade() {
                update_bill_totals(&window);
                window.invoke_autosave_order();
            }
        }
    });

    // ==================== UPDATE QUANTITY ====================
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
                    window.invoke_autosave_order();
                }
            }
        }
    });

    // ==================== REMOVE ORDER LINE ====================
    let weak_order_lines_remove = Rc::downgrade(&order_lines);
    let weak_window_order_remove = window.as_weak();
    window.on_remove_order_line(move |index| {
        if let Some(model) = weak_order_lines_remove.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                model.remove(index as usize);
                if let Some(window) = weak_window_order_remove.upgrade() {
                    update_bill_totals(&window);
                    window.invoke_autosave_order();
                }
            }
        }
    });

    // ==================== UPDATE NOTE ====================
    let weak_order_lines_note = Rc::downgrade(&order_lines);
    let weak_window_order_note = window.as_weak();
    window.on_update_order_line_note(move |index, note| {
        if let Some(model) = weak_order_lines_note.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                let mut line = model.row_data(index as usize).unwrap();
                line.note = note;
                model.set_row_data(index as usize, line);
                if let Some(window) = weak_window_order_note.upgrade() {
                    update_bill_totals(&window);
                    window.invoke_autosave_order();
                }
            }
        }
    });

    // ==================== RECALCULATE (charges) ====================
    let weak_window_recalc = window.as_weak();
    window.on_recalculate_bill(move || {
        if let Some(window) = weak_window_recalc.upgrade() {
            update_bill_totals(&window);
            window.invoke_autosave_order();
        }
    });
}
