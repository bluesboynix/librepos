slint::include_modules!();

mod db;

use slint::{Model, VecModel};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;
    window.window().set_maximized(true);

    // Initialize database (single file, shared connection)
    let conn = Rc::new(db::init_db("librepos.db")?);

    // Load menu items from DB
    let db_menu_items = db::menu::get_all_menu_items(&conn)?;
    let menu_items: Rc<VecModel<MenuItem>> = Rc::new(VecModel::from(
        db_menu_items
            .into_iter()
            .map(|item| MenuItem {
                id: item.id as i32,
                name: item.name.into(),
                code: item.code.unwrap_or_default().into(),
                price: item.price as f32,
                category: item.category.into(),
            })
            .collect::<Vec<_>>(),
    ));

    window.set_menu_items(menu_items.clone().into());

    // Areas (still in-memory for now, will move to DB later)
    let areas: Rc<VecModel<TableArea>> = Rc::new(VecModel::from(vec![
        TableArea { name: "Indoor".into(), tables: 4 },
        TableArea { name: "Outdoor".into(), tables: 4 },
        TableArea { name: "Delivery".into(), tables: 3 },
        TableArea { name: "Take Away".into(), tables: 3 },
    ]));
    window.set_areas(areas.clone().into());

    // Menu callbacks (separate weak handles)
    let weak_menu_items_add = Rc::downgrade(&menu_items);
    let weak_menu_items_remove = Rc::downgrade(&menu_items);
    let conn_menu = conn.clone();

    window.on_add_menu_item(move || {
        if let Some(model) = weak_menu_items_add.upgrade() {
            let _ = db::menu::add_menu_item(&conn_menu, "New Item", None, 0.0, "Test");
            if let Ok(db_items) = db::menu::get_all_menu_items(&conn_menu) {
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
    });

    window.on_remove_menu_item(move |index| {
        if let Some(model) = weak_menu_items_remove.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                let id = model.row_data(index as usize).unwrap().id as i64;
                let _ = db::menu::delete_menu_item(&conn, id);
                if let Ok(db_items) = db::menu::get_all_menu_items(&conn) {
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
        }
    });

    // Area callbacks (separate weak handles)
    let weak_areas_add = Rc::downgrade(&areas);
    let weak_areas_remove = Rc::downgrade(&areas);
    let weak_areas_set = Rc::downgrade(&areas);

    window.on_add_area(move || {
        if let Some(areas) = weak_areas_add.upgrade() {
            let new_area = TableArea { name: "New Area".into(), tables: 0 };
            areas.push(new_area);
        }
    });

    window.on_remove_area(move |index| {
        if let Some(areas) = weak_areas_remove.upgrade() {
            if index >= 0 && (index as usize) < areas.row_count() {
                areas.remove(index as usize);
            }
        }
    });

    window.on_set_tables(move |area_index, new_count| {
        if let Some(areas) = weak_areas_set.upgrade() {
            if area_index >= 0 && (area_index as usize) < areas.row_count() {
                let mut area = areas.row_data(area_index as usize).unwrap();
                area.tables = new_count;
                areas.set_row_data(area_index as usize, area);
            }
        }
    });

    // Quit
    window.on_quit(|| {
        std::process::exit(0);
    });

    window.run()?;
    Ok(())
}
