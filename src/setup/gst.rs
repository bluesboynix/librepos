use crate::MainWindow;
use crate::utils::update_bill_totals;
use slint::ComponentHandle;

pub fn setup_gst_callback(window: &MainWindow) {
    let weak_window_gst = window.as_weak();
    window.on_recalculate_gst(move || {
        if let Some(window) = weak_window_gst.upgrade() {
            let cgst = window
                .get_cgst_text()
                .parse::<f64>()
                .unwrap_or(0.0);
            let sgst = window
                .get_sgst_text()
                .parse::<f64>()
                .unwrap_or(0.0);
            let gst_rate = cgst + sgst;
            window.set_bill_gst_rate(gst_rate as f32);
            // Recalculate if bill view open
            if window.get_current_view() == 4 {
                update_bill_totals(&window);
            }
        }
    });
}
