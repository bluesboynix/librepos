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

        // Report models
    let report_top_items: Rc<VecModel<ReportItem>> = Rc::new(VecModel::default());
    let report_payments: Rc<VecModel<ReportPayment>> = Rc::new(VecModel::default());
    let report_days: Rc<VecModel<ReportDay>> = Rc::new(VecModel::default());
    let report_bills: Rc<VecModel<ReportBill>> = Rc::new(VecModel::default());

    window.set_report_bills(report_bills.clone().into());
    window.set_report_top_items(report_top_items.clone().into());
    window.set_report_payment_breakdown(report_payments.clone().into());
    window.set_report_daily_sales(report_days.clone().into());

    setup::reports::setup_reports_callbacks(
        &window,
        conn.clone(),
        report_top_items.clone(),
        report_payments.clone(),
        report_days.clone(),
        report_bills.clone(),
    );

    // Initialize with today's date and load once
    let today: String = conn
        .query_row(
            "SELECT date('now', 'localtime')",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    window.set_report_from_date(today.clone().into());
    window.set_report_to_date(today.clone().into());

    // Trigger an initial refresh by firing the callback manually
    window.invoke_load_reports(today.clone().into(), today.into());

    
    window.on_quit(|| {
        std::process::exit(0);
    });

    window.run()?;
    Ok(())
}
