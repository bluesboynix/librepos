slint::include_modules!();

use slint::{Model, VecModel};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;

    window.window().set_maximized(true);

    let areas: Rc<VecModel<TableArea>> = Rc::new(VecModel::from(vec![
        TableArea { name: "Indoor".into(), tables: 4 },
        TableArea { name: "Outdoor".into(), tables: 4 },
        TableArea { name: "Delivery".into(), tables: 3 },
        TableArea { name: "Take Away".into(), tables: 3 },
    ]));

    window.set_areas(areas.clone().into());

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

    window.on_quit(|| {
        std::process::exit(0);
    });

    window.run()?;

    Ok(())
}
