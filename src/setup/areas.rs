use crate::{MainWindow, TableArea, TableCardModel};
use crate::db;
use slint::{Model, VecModel};
use std::rc::Rc;

pub fn rebuild_areas(
    conn: &rusqlite::Connection,
    areas: &Rc<VecModel<TableArea>>,
) {
    let db_areas = match db::areas::get_all_areas(conn) {
        Ok(a) => a,
        Err(_) => return,
    };
    let totals = db::orders::get_open_order_totals(conn).unwrap_or_default();

    let new_areas: Vec<TableArea> = db_areas
        .into_iter()
        .map(|area| {
            let cards_vec: Vec<TableCardModel> = (1..=area.table_count)
                .map(|n| {
                    let label = format!("{} {}", area.name, n);
                    let total = totals.get(&label).copied().unwrap_or(0.0);
                    TableCardModel {
                        label: label.into(),
                        occupied: total > 0.0,
                        total: total as f32,
                    }
                })
                .collect();
            let cards_model = Rc::new(VecModel::from(cards_vec));
            TableArea {
                id: area.id as i32,
                name: area.name.into(),
                tables: area.table_count as i32,
                cards: cards_model.into(),
            }
        })
        .collect();

    areas.set_vec(new_areas);
}

fn build_cards_for_area(
    conn: &rusqlite::Connection,
    area_name: &str,
    table_count: i64,
) -> Vec<TableCardModel> {
    let totals = db::orders::get_open_order_totals(conn).unwrap_or_default();
    (1..=table_count)
        .map(|n| {
            let label = format!("{} {}", area_name, n);
            let total = totals.get(&label).copied().unwrap_or(0.0);
            TableCardModel {
                label: label.into(),
                occupied: total > 0.0,
                total: total as f32,
            }
        })
        .collect()
}

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
            let empty_cards = Rc::new(VecModel::<TableCardModel>::default());
            model.push(TableArea {
                id: new_id as i32,
                name: "New Area".into(),
                tables: 0,
                cards: empty_cards.into(),
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

    // Update area name (also rebuild cards to reflect new name)
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
                let cards_vec = build_cards_for_area(
                    &conn_update_name,
                    &name,
                    area.tables as i64,
                );
                let cards_model = Rc::new(VecModel::from(cards_vec));
                let mut updated = area.clone();
                updated.name = name.into();
                updated.cards = cards_model.into();
                model.set_row_data(index as usize, updated);
            }
        }
    });

    // Update table count (rebuild cards)
    let conn_set_tables = conn.clone();
    let weak_areas_set = Rc::downgrade(&areas);
    window.on_set_tables(move |area_index, new_count| {
        if let Some(model) = weak_areas_set.upgrade() {
            if area_index >= 0 && (area_index as usize) < model.row_count() {
                let area = model.row_data(area_index as usize).unwrap();
                let new_count = if new_count < 0 { 0 } else { new_count };
                let _ = db::areas::update_area(
                    &conn_set_tables,
                    area.id as i64,
                    &area.name,
                    new_count as i64,
                );
                let cards_vec = build_cards_for_area(
                    &conn_set_tables,
                    &area.name,
                    new_count as i64,
                );
                let cards_model = Rc::new(VecModel::from(cards_vec));
                let mut updated = area.clone();
                updated.tables = new_count;
                updated.cards = cards_model.into();
                model.set_row_data(area_index as usize, updated);
            }
        }
    });
}
