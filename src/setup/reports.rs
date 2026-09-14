use crate::{
    MainWindow, ReportBill, ReportDay, ReportItem, ReportPayment,
    ReportSummary,
};
use crate::db;
use slint::{ComponentHandle, VecModel};
use std::rc::Rc;

fn nice_label(v: f64) -> String {
    if v <= 0.0 {
        return "0".to_string();
    }
    if v < 1000.0 {
        return format!("{:.0}", v);
    }
    if v < 1_000_000.0 {
        return format!("{:.1}K", v / 1000.0);
    }
    format!("{:.1}M", v / 1_000_000.0)
}

fn run_report(
    conn: &rusqlite::Connection,
    window: &MainWindow,
    from: &str,
    to: &str,
    top_items: &Rc<VecModel<ReportItem>>,
    payments: &Rc<VecModel<ReportPayment>>,
    days: &Rc<VecModel<ReportDay>>,
    bills: &Rc<VecModel<ReportBill>>,
) {
    if let Ok(s) = db::reports::get_summary(conn, from, to) {
        window.set_report_summary(ReportSummary {
            total_sales: format!("{:.2}", s.total_sales).into(),
            order_count: s.order_count as i32,
            avg_order: format!("{:.2}", s.avg_order).into(),
        });
    }

    if let Ok(items) = db::reports::get_top_items(conn, from, to, 20) {
        let rows: Vec<ReportItem> = items
            .into_iter()
            .map(|i| ReportItem {
                name: i.name.into(),
                quantity: i.quantity as i32,
                revenue: format!("{:.2}", i.revenue).into(),
            })
            .collect();
        top_items.set_vec(rows);
    }

    if let Ok(p) = db::reports::get_payment_breakdown(conn, from, to) {
        let rows: Vec<ReportPayment> = p
            .into_iter()
            .map(|x| ReportPayment {
                method: x.method.into(),
                amount: format!("{:.2}", x.amount).into(),
                count: x.count as i32,
            })
            .collect();
        payments.set_vec(rows);
    }

        if let Ok(d) = db::reports::get_daily_sales(conn, from, to) {
        let rows: Vec<ReportDay> = d
            .into_iter()
            .map(|x| ReportDay {
                date: x.date.into(),
                sales: x.sales as f32,
                sales_display: format!("{:.2}", x.sales).into(),
                orders: x.orders as i32,
            })
            .collect();

        // Compute max for chart scaling
        let max_sale = rows
            .iter()
            .map(|d| d.sales)
            .fold(0.0f32, f32::max);
        window.set_report_max_daily_sale(max_sale);
        window.set_report_max_daily_sale_text(
            format!("{:.2}", max_sale).into(),
        );

        // Build y-axis labels
        let max_f = max_sale as f64;
        let ticks: Vec<slint::SharedString> = vec![
            nice_label(max_f).into(),
            nice_label(max_f * 0.75).into(),
            nice_label(max_f * 0.5).into(),
            nice_label(max_f * 0.25).into(),
            nice_label(0.0).into(),
        ];
        window.set_report_y_labels(
            Rc::new(VecModel::from(ticks)).into(),
        );

        days.set_vec(rows);
    }
    
    if let Ok(b) = db::reports::get_bills(conn, from, to, 100) {
        let rows: Vec<ReportBill> = b
            .into_iter()
            .map(|x| ReportBill {
                bill_no: x.bill_no.into(),
                table_name: x.table_name.into(),
                area_name: x.area_name.into(),
                total: format!("{:.2}", x.total).into(),
                closed_at: x.closed_at.into(),
                payment_status: x.payment_status.into(),
            })
            .collect();
        bills.set_vec(rows);
    }
}

fn compute_range(conn: &rusqlite::Connection, kind: i32) -> (String, String) {
    match kind {
        0 => {
            // Today
            let today: String = conn
                .query_row(
                    "SELECT date('now', 'localtime')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            (today.clone(), today)
        }
        1 => {
            // Last 7 days
            let from: String = conn
                .query_row(
                    "SELECT date('now', 'localtime', '-6 days')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            let to: String = conn
                .query_row(
                    "SELECT date('now', 'localtime')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            (from, to)
        }
        2 => {
            // This month
            let from: String = conn
                .query_row(
                    "SELECT date('now', 'localtime', 'start of month')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            let to: String = conn
                .query_row(
                    "SELECT date('now', 'localtime')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            (from, to)
        }
        _ => {
            let today: String = conn
                .query_row(
                    "SELECT date('now', 'localtime')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            (today.clone(), today)
        }
    }
}

pub fn setup_reports_callbacks(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    top_items: Rc<VecModel<ReportItem>>,
    payments: Rc<VecModel<ReportPayment>>,
    days: Rc<VecModel<ReportDay>>,
    bills: Rc<VecModel<ReportBill>>,
) {
    // Refresh with explicit from/to
    let weak_window_refresh = window.as_weak();
    let conn_refresh = conn.clone();
    let top_items_r = top_items.clone();
    let payments_r = payments.clone();
    let days_r = days.clone();
    let bills_r = bills.clone();
    window.on_load_reports(move |from, to| {
        if let Some(window) = weak_window_refresh.upgrade() {
            run_report(
                &conn_refresh,
                &window,
                &from.to_string(),
                &to.to_string(),
                &top_items_r,
                &payments_r,
                &days_r,
                &bills_r,
            );
        }
    });

    // Quick range selector
    let weak_window_qr = window.as_weak();
    let conn_qr = conn.clone();
    let top_items_qr = top_items.clone();
    let payments_qr = payments.clone();
    let days_qr = days.clone();
    let bills_qr = bills.clone();
    window.on_reports_quick_range(move |kind| {
        let Some(window) = weak_window_qr.upgrade() else {
            return;
        };

        let (from, to) = compute_range(&conn_qr, kind);

        window.set_report_from_date(from.clone().into());
        window.set_report_to_date(to.clone().into());

        run_report(
            &conn_qr,
            &window,
            &from,
            &to,
            &top_items_qr,
            &payments_qr,
            &days_qr,
            &bills_qr,
        );
    });
}
