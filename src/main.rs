slint::include_modules!();

mod db;
mod setup;
mod utils;

use slint::{Model, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;
    window.window().set_maximized(true);

    let conn = Rc::new(db::init_db("librepos.db")?);

    // Load menu items
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

    // Filtered model (initially identical to full list)
    let filtered_menu_items: Rc<VecModel<MenuItem>> = Rc::new(VecModel::from(
        (0..menu_items.row_count())
            .map(|i| menu_items.row_data(i).unwrap())
            .collect::<Vec<_>>(),
    ));
    window.set_filtered_menu_items(filtered_menu_items.clone().into());

    // Load areas
    let areas: Rc<VecModel<TableArea>> = Rc::new(VecModel::default());
    window.set_areas(areas.clone().into());
    setup::areas::rebuild_areas(&conn, &areas);

    // Shared current ingredient model
    let current_ingredients: Rc<
        RefCell<Option<Rc<VecModel<IngredientRow>>>>,
    > = Rc::new(RefCell::new(None));

    // Order lines model
    let order_lines: Rc<VecModel<OrderLine>> =
        Rc::new(VecModel::default());
    window.set_order_lines(order_lines.clone().into());

        // Setup callbacks
    setup::menu::setup_menu_callbacks(
        &window,
        conn.clone(),
        menu_items.clone(),
        filtered_menu_items.clone(),
        current_ingredients,
    );
    setup::menu::setup_menu_search(
        &window,
        menu_items.clone(),
        filtered_menu_items.clone(),
    );
    setup::order::setup_order_callbacks(&window, order_lines.clone());
    setup::areas::setup_area_callbacks(&window, conn.clone(), areas.clone());
    setup::settings::load_settings(&window, &conn);
    setup::settings::setup_settings_persistence(&window, conn.clone());
    setup::orders::setup_order_persistence(
        &window,
        conn.clone(),
        order_lines.clone(),
        menu_items.clone(),
        areas.clone(),
    );

    window.on_quit(|| {
        std::process::exit(0);
    });

    window.run()?;
    Ok(())
}
