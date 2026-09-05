slint::include_modules!();

use slint::{Model, VecModel};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;

    window.window().set_maximized(true);

    // Create initial areas using the generated TableArea type
    let areas: Rc<VecModel<TableArea>> = Rc::new(VecModel::from(vec![
        TableArea { name: "Indoor".into(), tables: 4 },
        TableArea { name: "Outdoor".into(), tables: 4 },
        TableArea { name: "Delivery".into(), tables: 3 },
        TableArea { name: "Take Away".into(), tables: 3 },
    ]));

    // Pass the model to the UI
    window.set_areas(areas.clone().into());

    // Create two weak handles for the closures
    let weak_areas_add = Rc::downgrade(&areas);
    let weak_areas_remove = Rc::downgrade(&areas);

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

    // Quit callback
    window.on_quit(|| {
        std::process::exit(0);
    });

    window.run()?;

    Ok(())
}
