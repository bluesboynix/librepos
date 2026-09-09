use crate::{IngredientRow, MainWindow, MenuItem, OrderLine};
use crate::db;
use slint::{Model, VecModel};
use std::rc::Rc;

pub fn reload_menu_items(
    conn: &rusqlite::Connection,
    model: &Rc<VecModel<MenuItem>>,
) {
    if let Ok(db_items) = db::menu::get_all_menu_items(conn) {
        model.set_vec(
            db_items
                .into_iter()
                .map(|item| MenuItem {
                    id: item.id as i32,
                    name: item.name.into(),
                    code: item.code.unwrap_or_default().into(),
                    price: item.price as f32,
                    category: item.category.into(),
                })
                .collect::<Vec<_>>(),
        );
    }
}

pub fn update_totals(window: &MainWindow) {
    let ingredients = window.get_current_ingredients();
    let total_cost: f64 = (0..ingredients.row_count())
        .map(|i| ingredients.row_data(i).unwrap().estimated_cost as f64)
        .sum();
    let price = window
        .get_current_price_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let profit = price - total_cost;
    window.set_total_cost_text(format!("{:.2}", total_cost).into());
    window.set_profit_text(format!("{:.2}", profit).into());
}

pub fn update_bill_totals(window: &MainWindow) {
    let order_lines = window.get_order_lines();
    let subtotal: f64 = (0..order_lines.row_count())
        .map(|i| {
            let line = order_lines.row_data(i).unwrap();
            line.price as f64 * line.quantity as f64
        })
        .sum();

    let cgst = window
        .get_cgst_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let sgst = window
        .get_sgst_text()
        .parse::<f64>()
        .unwrap_or(0.0);
    let gst_rate = cgst + sgst;

    let gst = subtotal * gst_rate / 100.0;
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
    let total = subtotal + gst + packaging + delivery - discount;

    window.set_bill_subtotal(subtotal as f32);
    window.set_bill_gst(gst as f32);
    window.set_bill_total(total as f32);
    window.set_bill_gst_rate(gst_rate as f32);
}
