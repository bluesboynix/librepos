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
    if let Ok(Some(v)) = db::settings::get_setting(conn, "store.name") {
        window.set_store_name_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "store.address") {
        window.set_store_address_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "store.phone") {
        window.set_store_phone_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "store.footer") {
        window.set_store_footer_text(v.into());
    }
    if let Ok(Some(v)) = db::settings::get_setting(conn, "store.fssai") {
        window.set_store_fssai_text(v.into());
    }
}

pub fn setup_settings_persistence(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
) {
    // GST
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

    // Currency
    let weak_window_cur = window.as_weak();
    let conn_cur = conn.clone();
    window.on_currency_changed(move || {
        if let Some(window) = weak_window_cur.upgrade() {
            let currency = window.get_currency().to_string();
            let _ = db::settings::set_setting(&conn_cur, "currency", &currency);
        }
    });

    // Store info
    let weak_window_store = window.as_weak();
    let conn_store = conn.clone();
    window.on_store_info_changed(move || {
        if let Some(window) = weak_window_store.upgrade() {
            let name = window.get_store_name_text().to_string();
            let address = window.get_store_address_text().to_string();
            let phone = window.get_store_phone_text().to_string();
            let fssai = window.get_store_fssai_text().to_string();
            let footer = window.get_store_footer_text().to_string();
            
            let _ = db::settings::set_setting(&conn_store, "store.name", &name);
            let _ = db::settings::set_setting(&conn_store, "store.address", &address);
            let _ = db::settings::set_setting(&conn_store, "store.phone", &phone);
            let _ = db::settings::set_setting(&conn_store, "store.fssai", &fssai);
            let _ = db::settings::set_setting(&conn_store, "store.footer", &footer);
        }
    });
}
