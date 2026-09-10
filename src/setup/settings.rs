use crate::MainWindow;
use crate::db;
use crate::utils::update_bill_totals;
use slint::ComponentHandle;
use std::rc::Rc;

pub fn load_settings(window: &MainWindow, conn: &rusqlite::Connection) {
    if let Ok(Some(v)) = db::settings::get_setting(conn, "cgst") {
        window.set_cgst_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "sgst") {
        window.set_sgst_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "currency") {
        window.set_currency(v.into());
    }
}

pub fn setup_settings_persistence(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
) {
    // GST changes: save to DB and refresh bill
    let weak_window_gst = window.as_weak();
    let conn_gst = conn.clone();
    window.on_recalculate_gst(move || {
        if let Some(window) = weak_window_gst.upgrade() {
            let cgst = window.get_cgst_text().to_string();
            let sgst = window.get_sgst_text().to_string();
            let _ = db::settings::set_setting(&conn_gst, "cgst", &cgst);
            let _ = db::settings::set_setting(&conn_gst, "sgst", &sgst);

            let cgst_f = cgst.parse::<f64>().unwrap_or(0.0);
            let sgst_f = sgst.parse::<f64>().unwrap_or(0.0);
            let gst_rate = cgst_f + sgst_f;
            window.set_bill_gst_rate(gst_rate as f32);
            if window.get_current_view() == 4 {
                update_bill_totals(&window);
            }
        }
    });

    // Currency change
    let weak_window_cur = window.as_weak();
    let conn_cur = conn.clone();
    window.on_currency_changed(move || {
        if let Some(window) = weak_window_cur.upgrade() {
            let currency = window.get_currency().to_string();
            let _ = db::settings::set_setting(&conn_cur, "currency", &currency);
        }
    });
}
