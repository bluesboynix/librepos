use crate::{MainWindow, TableArea};
use crate::db;
use slint::{Model, VecModel};
use std::rc::Rc;

pub fn setup_area_callbacks(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    areas: Rc<VecModel<TableArea>>,
) {
    // Add area
    let conn_add_area = conn.clone();
    let weak_areas_add = Rc::downgrade(&areas);
    window.on_add_area(move || {
        if let Some(model) = weak_areas_add.upgrade() {
            let new_id = db::areas::add_area(&conn_add_area, "New Area", 0).unwrap();
            model.push(TableArea {
                id: new_id as i32,
                name: "New Area".into(),
                tables: 0,
            });
        }
    });

    // Remove area
    let conn_remove_area = conn.clone();
    let weak_areas_remove = Rc::downgrade(&areas);
    window.on_remove_area(move |index| {
        if let Some(model) = weak_areas_remove.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                let area = model.row_data(index as usize).unwrap();
                let _ = db::areas::delete_area(&conn_remove_area, area.id as i64);
                model.remove(index as usize);
            }
        }
    });

    // Update table count
    let conn_set_tables = conn.clone();
    let weak_areas_set = Rc::downgrade(&areas);
    window.on_set_tables(move |area_index, new_count| {
        if let Some(model) = weak_areas_set.upgrade() {
            if area_index >= 0 && (area_index as usize) < model.row_count() {
                let area = model.row_data(area_index as usize).unwrap();
                let _ = db::areas::update_area(
                    &conn_set_tables,
                    area.id as i64,
                    &area.name,
                    new_count as i64,
                );
                let mut updated = area.clone();
                updated.tables = new_count;
                model.set_row_data(area_index as usize, updated);
            }
        }
    });

    // Update area name
    let conn_update_name = conn.clone();
    let weak_areas_name = Rc::downgrade(&areas);
    window.on_update_area_name(move |index, name| {
        if let Some(model) = weak_areas_name.upgrade() {
            if index >= 0 && (index as usize) < model.row_count() {
                let area = model.row_data(index as usize).unwrap();
                let _ = db::areas::update_area(
                    &conn_update_name,
                    area.id as i64,
                    &name,
                    area.tables as i64,
                );
                let mut updated = area.clone();
                updated.name = name.into();
                model.set_row_data(index as usize, updated);
            }
        }
    });
}
