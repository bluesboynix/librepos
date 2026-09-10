use crate::{
    MainWindow, ReportBill, ReportDay, ReportItem, ReportPayment,
    ReportSummary,
};
use crate::db;
use slint::{ComponentHandle, VecModel};
use std::rc::Rc;

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
            total_sales: s.total_sales as f32,
            order_count: s.order_count as i32,
            avg_order: s.avg_order as f32,
        });
    }

    if let Ok(items) = db::reports::get_top_items(conn, from, to, 20) {
        let rows: Vec<ReportItem> = items
            .into_iter()
            .map(|i| ReportItem {
                name: i.name.into(),
                quantity: i.quantity as i32,
                revenue: i.revenue as f32,
            })
            .collect();
        top_items.set_vec(rows);
    }

    if let Ok(p) = db::reports::get_payment_breakdown(conn, from, to) {
        let rows: Vec<ReportPayment> = p
            .into_iter()
            .map(|x| ReportPayment {
                method: x.method.into(),
                amount: x.amount as f32,
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
                orders: x.orders as i32,
            })
            .collect();
        days.set_vec(rows);
    }

    if let Ok(b) = db::reports::get_bills(conn, from, to, 100) {
        let rows: Vec<ReportBill> = b
            .into_iter()
            .map(|x| ReportBill {
                bill_no: x.bill_no.into(),
                table_name: x.table_name.into(),
                area_name: x.area_name.into(),
                total: x.total as f32,
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
