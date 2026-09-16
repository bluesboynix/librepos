use crate::{
    BillDetail, BillLine, BillPayment, MainWindow, ReportBill, ReportDay, ReportItem, ReportPayment, ReportSummary};
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
    
        
    if let Ok(b) = db::reports::get_bills(
        conn,
        from,
        to,
        100,
        window.get_bills_show_voided(),
        "",
    ) {
        let rows: Vec<ReportBill> = b
            .into_iter()
            .map(|x| ReportBill {
                id: x.id as i32,
                bill_no: x.bill_no.into(),
                table_name: x.table_name.into(),
                area_name: x.area_name.into(),
                total: format!("{:.2}", x.total).into(),
                closed_at: x.closed_at.into(),
                payment_status: x.payment_status.into(),
                status: x.status.into(),
                void_reason: x.void_reason.into(),
                edit_count: x.edit_count,
                payment_methods: x.payment_methods.into(),
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

    // ==================== LOAD BILL DETAIL ====================
    let weak_window_bill = window.as_weak();
    let conn_bill = conn.clone();
    window.on_load_bill_detail(move |order_id| {
        let Some(window) = weak_window_bill.upgrade() else {
            return;
        };
        let Some(detail) =
            db::reports::get_bill_detail(&conn_bill, order_id as i64).ok().flatten()
            else {
                return;
            };

        let items: Vec<BillLine> = detail
            .items
            .into_iter()
            .map(|i| BillLine {
                name: i.name.into(),
                category: i.category.into(),
                quantity: i.quantity,
                unit_price: format!("{:.2}", i.unit_price).into(),
                line_total: format!("{:.2}", i.line_total).into(),
                note: i.note.into(),
            })
            .collect();

        let payments: Vec<BillPayment> = detail
            .payments
            .into_iter()
            .map(|p| BillPayment {
                method: p.method.into(),
                amount: format!("{:.2}", p.amount).into(),
            })
            .collect();

        window.set_current_bill_detail(BillDetail {
            id: detail.id as i32,
            bill_no: detail.bill_no.into(),
            table_name: detail.table_name.into(),
            area_name: detail.area_name.into(),
            closed_at: detail.closed_at.into(),
            payment_status: detail.payment_status.into(),
            status: detail.status.into(),
            void_reason: detail.void_reason.into(),
            voided_at: detail.voided_at.into(),
            subtotal: format!("{:.2}", detail.subtotal).into(),
            cgst: format!("{:.2}", detail.cgst).into(),
            sgst: format!("{:.2}", detail.sgst).into(),
            gst: format!("{:.2}", detail.gst).into(),
            packaging: format!("{:.2}", detail.packaging).into(),
            delivery: format!("{:.2}", detail.delivery).into(),
            discount: format!("{:.2}", detail.discount).into(),
            total: format!("{:.2}", detail.total).into(),
            customer_name: detail.customer_name.into(),
            customer_phone: detail.customer_phone.into(),
            customer_address: detail.customer_address.into(),
            items: Rc::new(VecModel::from(items)).into(),
            payments: Rc::new(VecModel::from(payments)).into(),
            edit_history: Rc::new(VecModel::from(
                detail.edit_history
                    .into_iter()
                    .map(slint::SharedString::from)
                    .collect::<Vec<_>>(),
            )).into(),
        });
        window.set_bills_detail_open(true);
    });

    let weak_window_clear = window.as_weak();
    window.on_clear_bill_detail(move || {
        if let Some(window) = weak_window_clear.upgrade() {
            window.set_bills_detail_open(false);
        }
    });

    // ==================== VOID BILL ====================
    // Perform void (after dialog confirmed)
    let weak_window_v = window.as_weak();
    let conn_v = conn.clone();
    window.on_perform_void(move || {
        let Some(window) = weak_window_v.upgrade() else { return; };
        let detail = window.get_current_bill_detail();
        let reason = window.get_void_reason().to_string();
        let _ = db::orders::void_order(&conn_v, detail.id as i64, &reason);
        window.set_void_reason("".into());
        window.set_bills_detail_open(false);
        // Reload bills list
        let from = window.get_report_from_date().to_string();
        let to = window.get_report_to_date().to_string();
        window.invoke_load_reports(from.into(), to.into());
    });

    // Revive
    let weak_window_rv = window.as_weak();
    let conn_rv = conn.clone();
    window.on_revive(move || {
        let Some(window) = weak_window_rv.upgrade() else { return; };
        let detail = window.get_current_bill_detail();
        let _ = db::orders::revive_order(&conn_rv, detail.id as i64);
        window.set_bills_detail_open(false);
        let from = window.get_report_from_date().to_string();
        let to = window.get_report_to_date().to_string();
        window.invoke_load_reports(from.into(), to.into());
    });

    // Toggle voided view
    let weak_window_t = window.as_weak();
    let conn_t = conn.clone();
    let bills_t = bills.clone();
    window.on_toggle_voided(move || {
        if let Some(window) = weak_window_t.upgrade() {
            let current = window.get_bills_show_voided();
            window.set_bills_show_voided(!current);
            let from = window.get_report_from_date().to_string();
            let to = window.get_report_to_date().to_string();
            let query = window.get_bills_search_text().to_string();
            let include_voided = !current;

            if let Ok(b) = db::reports::get_bills(
                &conn_t, &from, &to, 100, include_voided, &query,
            ) {
                let rows: Vec<ReportBill> = b
                    .into_iter()
                    .map(|x| ReportBill {
                        id: x.id as i32,
                        bill_no: x.bill_no.into(),
                        table_name: x.table_name.into(),
                        area_name: x.area_name.into(),
                        total: format!("{:.2}", x.total).into(),
                        closed_at: x.closed_at.into(),
                        payment_status: x.payment_status.into(),
                        status: x.status.into(),
                        void_reason: x.void_reason.into(),
                        edit_count: x.edit_count,
                        payment_methods: x.payment_methods.into(),
                    })
                    .collect();
                bills_t.set_vec(rows);
            }
        }
    });

    // ==================== FILTER BILLS ====================
    let weak_window_filt = window.as_weak();
    let conn_filt = conn.clone();
    let bills_filt = bills.clone();
    window.on_filter_bills(move |query| {
        let Some(window) = weak_window_filt.upgrade() else {
            return;
        };
        let from = window.get_report_from_date().to_string();
        let to = window.get_report_to_date().to_string();
        let include_voided = window.get_bills_show_voided();
        let q = query.to_string();

        if let Ok(b) =
            db::reports::get_bills(&conn_filt, &from, &to, 100, include_voided, &q)
        {
            let rows: Vec<ReportBill> = b
                .into_iter()
                .map(|x| ReportBill {
                    id: x.id as i32,
                    bill_no: x.bill_no.into(),
                    table_name: x.table_name.into(),
                    area_name: x.area_name.into(),
                    total: format!("{:.2}", x.total).into(),
                    closed_at: x.closed_at.into(),
                    payment_status: x.payment_status.into(),
                    status: x.status.into(),
                    void_reason: x.void_reason.into(),
                    edit_count: x.edit_count,
                    payment_methods: x.payment_methods.into(),
                })
                .collect();
            bills_filt.set_vec(rows);
        }
    });

    
    // ==================== EDIT CLOSED BILL ====================
    let weak_window_edit = window.as_weak();
    let conn_edit = conn.clone();
    window.on_edit_bill(move || {
        let Some(window) = weak_window_edit.upgrade() else {
            return;
        };
        let detail = window.get_current_bill_detail();
        let order_id = detail.id as i64;
        let table_name = detail.table_name.to_string();

        // Safety: don't reopen if the table already has a different open order
        if let Ok(Some(existing)) =
            db::orders::get_open_order_by_table(&conn_edit, &table_name)
        {
            if existing.id != order_id {
                eprintln!(
                    "[edit] table {} already has open order id={}",
                    table_name, existing.id
                );
                return;
            }
        }

        let _ = db::orders::reopen_order(&conn_edit, order_id);

        // Close detail view and hand off to BillView
        window.set_bills_detail_open(false);
        window.invoke_open_table(table_name.into());
    });
}
