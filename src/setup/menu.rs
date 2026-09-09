use crate::{IngredientRow, MainWindow, MenuItem};
use crate::db;
use crate::utils::{reload_menu_items, update_totals};
use slint::{ComponentHandle, Model, VecModel};   // add ComponentHandle
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup_menu_callbacks(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    menu_items: Rc<VecModel<MenuItem>>,
    current_ingredients: Rc<RefCell<Option<Rc<VecModel<IngredientRow>>>>>,
) {
    let conn = conn.clone();

    // Add menu item
    let conn_add = conn.clone();
    let weak_menu_items_add = Rc::downgrade(&menu_items);
    window.on_add_menu_item(move || {
        if let Some(model) = weak_menu_items_add.upgrade() {
            let _ = db::menu::add_menu_item(
                &conn_add,
                "New Item",
                None,
                0.0,
                "Test",
            );
            reload_menu_items(&conn_add, &model);
        }
    });

    // Remove menu item
    let conn_remove = conn.clone();
    let weak_menu_items_remove = Rc::downgrade(&menu_items);
    window.on_remove_menu_item(move |index| {
        if let Some(model) = weak_menu_items_remove.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                let id = model.row_data(index as usize).unwrap().id as i64;
                let _ = db::menu::delete_menu_item(&conn_remove, id);
                reload_menu_items(&conn_remove, &model);
            }
        }
    });

    // Open menu item details
    let conn_open = conn.clone();
    let weak_menu_items_open = Rc::downgrade(&menu_items);
    let weak_window_open = window.as_weak();
    let current_ingredients_open = current_ingredients.clone();
    window.on_open_menu_item(move |index| {
        if let (Some(window), Some(model)) = (
            weak_window_open.upgrade(),
            weak_menu_items_open.upgrade(),
        ) {
            if index >= 0 && (index as usize) < model.row_count() {
                let item = model.row_data(index as usize).unwrap();
                let ingredients = db::menu::get_ingredients_for_item(
                    &conn_open,
                    item.id as i64,
                )
                .unwrap_or_default();
                let ing_rows: Vec<IngredientRow> = ingredients
                    .into_iter()
                    .map(
                        |(link_id, ing_id, name, qty, unit, cost)| {
                            IngredientRow {
                                link_id: link_id as i32,
                                ingredient_id: ing_id as i32,
                                name: name.into(),
                                quantity: qty as f32,
                                unit: unit.unwrap_or_default().into(),
                                estimated_cost: cost as f32,
                            }
                        },
                    )
                    .collect();
                let ing_model = Rc::new(VecModel::from(ing_rows));
                window.set_current_menu_item(item.clone());
                window.set_name_text(item.name.clone());
                window.set_code_text(item.code.clone());
                window.set_current_price_text(
                    item.price.to_string().into(),
                );
                window.set_category_text(item.category.clone());
                window.set_current_ingredients(ing_model.clone().into());
                *current_ingredients_open.borrow_mut() = Some(ing_model);
                update_totals(&window);
                window.set_current_view(5);
            }
        }
    });

    // Save menu item
    let conn_save = conn.clone();
    let weak_menu_items_save = Rc::downgrade(&menu_items);
    let weak_window_save = window.as_weak();
    window.on_save_menu_item(move || {
        if let (Some(window), Some(model)) = (
            weak_window_save.upgrade(),
            weak_menu_items_save.upgrade(),
        ) {
            let item = window.get_current_menu_item();
            let id = item.id as i64;
            let name = window.get_name_text();
            let code = window.get_code_text();
            let price_text = window.get_current_price_text();
            let price = price_text.parse::<f64>().unwrap_or(0.0);
            let category = window.get_category_text();
            if let Err(e) = db::menu::update_menu_item(
                &conn_save,
                id,
                &name,
                Some(&code),
                price,
                &category,
            ) {
                eprintln!("Update failed: {}", e);
            }
            reload_menu_items(&conn_save, &model);
            window.set_current_view(2);
        }
    });

    // Cancel menu item
    let weak_window_cancel = window.as_weak();
    window.on_cancel_menu_item(move || {
        if let Some(window) = weak_window_cancel.upgrade() {
            window.set_current_view(2);
        }
    });

    // Add ingredient
    let conn_add_ing = conn.clone();
    let weak_window_add_ing = window.as_weak();
    let current_ingredients_add = current_ingredients.clone();
    window.on_add_ingredient(move || {
        if let Some(window) = weak_window_add_ing.upgrade() {
            let item = window.get_current_menu_item();
            let menu_item_id = item.id as i64;
            let new_ing_id = db::menu::add_ingredient(
                &conn_add_ing,
                "New Ingredient",
                Some("g"),
                Some(0.0),
            )
            .unwrap();
            let new_link_id = db::menu::add_ingredient_to_item(
                &conn_add_ing,
                menu_item_id,
                new_ing_id,
                0.0,
                Some("g"),
                0.0,
            )
            .unwrap();
            if let Some(model) = current_ingredients_add.borrow().as_ref() {
                model.push(IngredientRow {
                    link_id: new_link_id as i32,
                    ingredient_id: new_ing_id as i32,
                    name: "New Ingredient".into(),
                    quantity: 0.0,
                    unit: "g".into(),
                    estimated_cost: 0.0,
                });
                update_totals(&window);
            }
        }
    });

    // Remove ingredient
    let conn_remove_ing = conn.clone();
    let weak_window_remove_ing = window.as_weak();
    let current_ingredients_remove = current_ingredients.clone();
    window.on_remove_ingredient(move |index| {
        if let Some(window) = weak_window_remove_ing.upgrade() {
            if let Some(model) = current_ingredients_remove.borrow().as_ref() {
                if index >= 0 && (index as usize) < model.row_count() {
                    let ing = model.row_data(index as usize).unwrap();
                    let _ = db::menu::delete_ingredient_link(
                        &conn_remove_ing,
                        ing.link_id as i64,
                    );
                    model.remove(index as usize);
                    update_totals(&window);
                }
            }
        }
    });

    // Update ingredient quantity
    let conn_update_qty = conn.clone();
    let current_ingredients_qty = current_ingredients.clone();
    let weak_window_qty = window.as_weak();
    window.on_update_ingredient_quantity(move |index, text| {
        if let Some(model) = current_ingredients_qty.borrow().as_ref() {
            if index >= 0 && (index as usize) < model.row_count() {
                let ing = model.row_data(index as usize).unwrap();
                let new_qty = text.parse::<f64>().unwrap_or(0.0);
                let _ = db::menu::update_ingredient_link(
                    &conn_update_qty,
                    ing.link_id as i64,
                    new_qty,
                    Some(&ing.unit),
                    ing.estimated_cost as f64,
                );
                let mut updated = ing.clone();
                updated.quantity = new_qty as f32;
                model.set_row_data(index as usize, updated);
                if let Some(window) = weak_window_qty.upgrade() {
                    update_totals(&window);
                }
            }
        }
    });

    // Update ingredient unit
    let conn_update_unit = conn.clone();
    let current_ingredients_unit = current_ingredients.clone();
    let weak_window_unit = window.as_weak();
    window.on_update_ingredient_unit(move |index, text| {
        if let Some(model) = current_ingredients_unit.borrow().as_ref() {
            if index >= 0 && (index as usize) < model.row_count() {
                let ing = model.row_data(index as usize).unwrap();
                let new_unit = text;
                let _ = db::menu::update_ingredient_link(
                    &conn_update_unit,
                    ing.link_id as i64,
                    ing.quantity as f64,
                    Some(&new_unit),
                    ing.estimated_cost as f64,
                );
                let mut updated = ing.clone();
                updated.unit = new_unit.into();
                model.set_row_data(index as usize, updated);
                if let Some(window) = weak_window_unit.upgrade() {
                    update_totals(&window);
                }
            }
        }
    });

    // Update ingredient cost
    let conn_update_cost = conn.clone();
    let current_ingredients_cost = current_ingredients.clone();
    let weak_window_cost = window.as_weak();
    window.on_update_ingredient_cost(move |index, text| {
        if let Some(model) = current_ingredients_cost.borrow().as_ref() {
            if index >= 0 && (index as usize) < model.row_count() {
                let ing = model.row_data(index as usize).unwrap();
                let new_cost = text.parse::<f64>().unwrap_or(0.0);
                let _ = db::menu::update_ingredient_link(
                    &conn_update_cost,
                    ing.link_id as i64,
                    ing.quantity as f64,
                    Some(&ing.unit),
                    new_cost,
                );
                let mut updated = ing.clone();
                updated.estimated_cost = new_cost as f32;
                model.set_row_data(index as usize, updated);
                if let Some(window) = weak_window_cost.upgrade() {
                    update_totals(&window);
                }
            }
        }
    });

    // Update ingredient name
    let conn_update_name = conn.clone();
    let current_ingredients_name = current_ingredients.clone();
    let weak_window_name = window.as_weak();
    window.on_update_ingredient_name(move |index, text| {
        if let Some(model) = current_ingredients_name.borrow().as_ref() {
            if index >= 0 && (index as usize) < model.row_count() {
                let ing = model.row_data(index as usize).unwrap();
                let _ = db::menu::update_ingredient_name(
                    &conn_update_name,
                    ing.ingredient_id as i64,
                    &text,
                );
                let mut updated = ing.clone();
                updated.name = text.into();
                model.set_row_data(index as usize, updated);
                if let Some(window) = weak_window_name.upgrade() {
                    update_totals(&window);
                }
            }
        }
    });
}
