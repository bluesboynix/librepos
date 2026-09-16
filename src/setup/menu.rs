use crate::{IngredientRow, MainWindow, MenuItem};
use crate::db;
use crate::utils::{refresh_menu_categories, reload_menu_items, update_totals};
use slint::{ComponentHandle, Model, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup_menu_callbacks(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    menu_items: Rc<VecModel<MenuItem>>,
    filtered_menu_items: Rc<VecModel<MenuItem>>,
    menu_categories: Rc<VecModel<slint::SharedString>>,
    current_ingredients: Rc<RefCell<Option<Rc<VecModel<IngredientRow>>>>>,
) {
    let conn = conn.clone();

    // ==================== ADD MENU ITEM ====================
    let conn_add = conn.clone();
    let weak_menu_items_add = Rc::downgrade(&menu_items);
    let weak_filtered_add = Rc::downgrade(&filtered_menu_items);
    let weak_window_add = window.as_weak();
    let current_ingredients_add = current_ingredients.clone();
    let cats_add = menu_categories.clone();
    window.on_add_menu_item(move || {
        let (Some(window), Some(model), Some(filtered)) = (
            weak_window_add.upgrade(),
            weak_menu_items_add.upgrade(),
            weak_filtered_add.upgrade(),
        ) else {
            return;
        };

        let new_id = db::menu::add_menu_item(
            &conn_add,
            "New Item",
            None,
            0.0,
            "Test",
        )
        .unwrap();

        reload_menu_items(&conn_add, &model, &filtered);
        refresh_menu_categories(&model, &cats_add);

        // Find the new item and open details
        let mut new_item: Option<MenuItem> = None;
        for i in 0..model.row_count() {
            let item = model.row_data(i).unwrap();
            if item.id == new_id as i32 {
                new_item = Some(item);
                break;
            }
        }

        let Some(item) = new_item else { return; };

        let empty_ing = Rc::new(VecModel::<IngredientRow>::default());
        window.set_current_menu_item(item.clone());
        window.set_name_text(item.name.clone());
        window.set_code_text(item.code.clone());
        window.set_current_price_text(item.price.to_string().into());
        window.set_category_text(item.category.clone());
        window.set_current_ingredients(empty_ing.clone().into());
        *current_ingredients_add.borrow_mut() = Some(empty_ing);
        update_totals(&window);
        window.set_current_view(5);
    });

    // ==================== REMOVE MENU ITEM ====================
    let conn_remove = conn.clone();
    let weak_menu_items_remove = Rc::downgrade(&menu_items);
    let weak_filtered_remove = Rc::downgrade(&filtered_menu_items);
    let cats_remove = menu_categories.clone();
    window.on_remove_menu_item(move |id| {
        if let (Some(model), Some(filtered)) =
            (weak_menu_items_remove.upgrade(), weak_filtered_remove.upgrade())
        {
            if let Err(e) = db::menu::delete_menu_item(&conn_remove, id as i64) {
                eprintln!("Delete failed: {}", e);
            }
            reload_menu_items(&conn_remove, &model, &filtered);
            refresh_menu_categories(&model, &cats_remove);
        }
    });

    // ==================== OPEN MENU ITEM DETAILS ====================
    let conn_open = conn.clone();
    let weak_menu_items_open = Rc::downgrade(&menu_items);
    let weak_window_open = window.as_weak();
    let current_ingredients_open = current_ingredients.clone();
    window.on_open_menu_item(move |item_id| {
        let (Some(window), Some(model)) = (
            weak_window_open.upgrade(),
            weak_menu_items_open.upgrade(),
        ) else {
            return;
        };

        let mut found: Option<MenuItem> = None;
        for i in 0..model.row_count() {
            let item = model.row_data(i).unwrap();
            if item.id == item_id {
                found = Some(item);
                break;
            }
        }

        let Some(item) = found else { return; };

        let ingredients = db::menu::get_ingredients_for_item(
            &conn_open,
            item.id as i64,
        )
        .unwrap_or_default();

        let ing_rows: Vec<IngredientRow> = ingredients
            .into_iter()
            .map(|(link_id, ing_id, name, qty, unit, cost)| IngredientRow {
                link_id: link_id as i32,
                ingredient_id: ing_id as i32,
                name: name.into(),
                quantity: qty as f32,
                unit: unit.unwrap_or_default().into(),
                estimated_cost: cost as f32,
            })
            .collect();

        let ing_model = Rc::new(VecModel::from(ing_rows));
        window.set_current_menu_item(item.clone());
        window.set_name_text(item.name.clone());
        window.set_code_text(item.code.clone());
        window.set_current_price_text(item.price.to_string().into());
        window.set_category_text(item.category.clone());
        window.set_current_ingredients(ing_model.clone().into());
        *current_ingredients_open.borrow_mut() = Some(ing_model);
        update_totals(&window);
        window.set_current_view(5);
    });

    // ==================== SAVE MENU ITEM ====================
    let conn_save = conn.clone();
    let weak_menu_items_save = Rc::downgrade(&menu_items);
    let weak_filtered_save = Rc::downgrade(&filtered_menu_items);
    let weak_window_save = window.as_weak();
    let cats_save = menu_categories.clone();
    window.on_save_menu_item(move || {
        let (Some(window), Some(model), Some(filtered)) = (
            weak_window_save.upgrade(),
            weak_menu_items_save.upgrade(),
            weak_filtered_save.upgrade(),
        ) else {
            return;
        };

        let item = window.get_current_menu_item();
        let id = item.id as i64;
        let name = window.get_name_text();
        let code = window.get_code_text();
        let price_text = window.get_current_price_text();
        let price = price_text.parse::<f64>().unwrap_or(0.0);
        let category = window.get_category_text();

        let code_opt: Option<&str> =
            if code.is_empty() { None } else { Some(&code) };

        let _ = db::menu::update_menu_item(
            &conn_save,
            id,
            &name,
            code_opt,
            price,
            &category,
        );

        reload_menu_items(&conn_save, &model, &filtered);
        refresh_menu_categories(&model, &cats_save);
        window.set_current_view(2);
    });

    // ==================== CANCEL MENU ITEM ====================
    let weak_window_cancel = window.as_weak();
    window.on_cancel_menu_item(move || {
        if let Some(window) = weak_window_cancel.upgrade() {
            window.set_current_view(2);
        }
    });

    // ==================== ADD INGREDIENT ====================
    let conn_add_ing = conn.clone();
    let weak_window_add_ing = window.as_weak();
    let current_ingredients_add_ing = current_ingredients.clone();
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
            if let Some(model) = current_ingredients_add_ing.borrow().as_ref() {
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

    // ==================== REMOVE INGREDIENT ====================
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

    // ==================== UPDATE INGREDIENT QUANTITY ====================
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

    // ==================== UPDATE INGREDIENT UNIT ====================
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

    // ==================== UPDATE INGREDIENT COST ====================
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

    // ==================== UPDATE INGREDIENT NAME ====================
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

pub fn setup_menu_search(
    window: &MainWindow,
    full: Rc<VecModel<MenuItem>>,
    filtered: Rc<VecModel<MenuItem>>,
) {
    let weak_full = Rc::downgrade(&full);
    let weak_filtered = Rc::downgrade(&filtered);
    window.on_filter_menu_items(move |query, category| {
        let (Some(full), Some(filtered)) =
            (weak_full.upgrade(), weak_filtered.upgrade())
        else {
            return;
        };

        let q = query.to_string().to_lowercase();
        let cat = category.to_string();

        let mut list: Vec<MenuItem> = Vec::new();
        for i in 0..full.row_count() {
            let item = full.row_data(i).unwrap();

            // Category filter (empty category = show all)
            if !cat.is_empty() && item.category.as_str() != cat {
                continue;
            }

            // Search filter (empty query = show all in current category)
            if !q.is_empty() {
                let name = item.name.to_lowercase();
                let item_cat = item.category.to_lowercase();
                if !name.contains(&q) && !item_cat.contains(&q) {
                    continue;
                }
            }

            list.push(item);
        }

        filtered.set_vec(list);
    });
}
