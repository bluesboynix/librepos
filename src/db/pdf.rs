use crate::db::reports::BillDetail;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const PAGE_WIDTH_MM: f32 = 80.0;
const MARGIN_MM: f32 = 5.0;
const LINE_HEIGHT_MM: f32 = 4.5;

fn courier_width_mm(text: &str, size_pt: f32) -> f32 {
    let n = text.chars().count() as f32;
    n * size_pt * 0.6 * 0.3528
}

fn sanitize(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii() {
            out.push(c);
        } else if c == '₹' {
            out.push_str("Rs.");
        } else if c == '—' || c == '–' {
            out.push('-');
        } else if c == '“' || c == '”' {
            out.push('"');
        } else if c == '‘' || c == '’' {
            out.push('\'');
        } else {
            out.push('?');
        }
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn fmt2(v: f64) -> String {
    format!("{:.2}", v)
}

struct Line {
    text: String,
    size: f32,
    bold: bool,
    align: u8,
}

pub fn generate_bill_pdf(
    detail: &BillDetail,
    store_name: &str,
    store_address: &str,
    store_phone: &str,
    store_fssai: &str,
    store_footer: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut lines: Vec<Line> = Vec::new();

    if !store_name.is_empty() {
        lines.push(Line { text: sanitize(store_name), size: 12.0, bold: true, align: 1 });
    }
    if !store_address.is_empty() {
        for part in store_address.lines() {
            lines.push(Line { text: sanitize(part), size: 9.0, bold: false, align: 1 });
        }
    }
    if !store_phone.is_empty() {
        lines.push(Line { text: format!("Ph: {}", sanitize(store_phone)), size: 9.0, bold: false, align: 1 });
    }
    if !store_fssai.is_empty() {
        lines.push(Line { text: format!("FSSAI: {}", sanitize(store_fssai)), size: 9.0, bold: false, align: 1 });
    }

    lines.push(Line { text: "-".repeat(36), size: 9.0, bold: false, align: 1 });
    lines.push(Line { text: format!("Bill : {}", sanitize(&detail.bill_no)), size: 10.0, bold: false, align: 0 });
    lines.push(Line { text: format!("Table: {}", sanitize(&detail.table_name)), size: 10.0, bold: false, align: 0 });
    lines.push(Line { text: format!("Area : {}", sanitize(&detail.area_name)), size: 10.0, bold: false, align: 0 });
    lines.push(Line { text: format!("Closed: {}", sanitize(&detail.closed_at)), size: 10.0, bold: false, align: 0 });
    if !detail.customer_name.is_empty() {
        lines.push(Line { text: format!("Cust : {}", sanitize(&detail.customer_name)), size: 10.0, bold: false, align: 0 });
    }
    if !detail.customer_phone.is_empty() {
        lines.push(Line { text: format!("Phone: {}", sanitize(&detail.customer_phone)), size: 10.0, bold: false, align: 0 });
    }

    lines.push(Line { text: "-".repeat(33), size: 9.0, bold: false, align: 0 });

    lines.push(Line {
        text: format!("{:<20}{:>3}{:>10}", "Item", "Qty", "Amount"),
        size: 10.0, bold: true, align: 0,
    });
    for item in &detail.items {
        let name = truncate(&sanitize(&item.name), 20);
        let qty = item.quantity.to_string();
        let amount = fmt2(item.line_total);
        lines.push(Line {
            text: format!("{:<20}{:>3}{:>10}", name, qty, amount),
            size: 10.0, bold: false, align: 0,
        });
        if !item.note.is_empty() {
            let note = sanitize(&item.note);
            lines.push(Line {
                text: format!("  * {}", truncate(&note, 28)),
                size: 9.0, bold: false, align: 0,
            });
        }
    }

    lines.push(Line { text: "-".repeat(33), size: 9.0, bold: false, align: 0 });

    let push_total = |label: &str, value: String, bold: bool, size: f32, lines: &mut Vec<Line>| {
        lines.push(Line {
            text: format!("{:<20}{:>13}", label, value),
            size, bold, align: 0,
        });
    };

    push_total("Subtotal", fmt2(detail.subtotal), false, 10.0, &mut lines);
    if detail.cgst != 0.0 {
        push_total("CGST", fmt2(detail.cgst), false, 10.0, &mut lines);
    }
    if detail.sgst != 0.0 {
        push_total("SGST", fmt2(detail.sgst), false, 10.0, &mut lines);
    }
    if detail.packaging != 0.0 {
        push_total("Packaging", fmt2(detail.packaging), false, 10.0, &mut lines);
    }
    if detail.delivery != 0.0 {
        push_total("Delivery", fmt2(detail.delivery), false, 10.0, &mut lines);
    }
    if detail.discount != 0.0 {
        push_total("Discount", format!("-{}", fmt2(detail.discount)), false, 10.0, &mut lines);
    }

    lines.push(Line { text: "-".repeat(33), size: 9.0, bold: false, align: 0 });

    lines.push(Line {
        text: format!("{:<18}{:>12}", "TOTAL", fmt2(detail.total)),
        size: 11.0, bold: true, align: 0,
    });

    if !detail.payments.is_empty() {
        lines.push(Line { text: "-".repeat(33), size: 9.0, bold: false, align: 0 });
        for p in &detail.payments {
            push_total(&p.method, fmt2(p.amount), false, 10.0, &mut lines);
        }
    }

    if !store_footer.is_empty() {
        lines.push(Line { text: "-".repeat(36), size: 9.0, bold: false, align: 1 });
        for part in store_footer.lines() {
            lines.push(Line { text: sanitize(part), size: 10.0, bold: false, align: 1 });
        }
    }

    let content_h = lines.len() as f32 * LINE_HEIGHT_MM;
    let page_h = 2.0 * MARGIN_MM + content_h;

    let (doc, page1, layer1) = PdfDocument::new(
        "Bill",
        Mm(PAGE_WIDTH_MM),
        Mm(page_h),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Courier)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::CourierBold)?;

    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = page_h - MARGIN_MM;
    for line in &lines {
        if !line.text.is_empty() {
            let text_w = courier_width_mm(&line.text, line.size);
            let x = match line.align {
                1 => (PAGE_WIDTH_MM - text_w) / 2.0,
                2 => PAGE_WIDTH_MM - MARGIN_MM - text_w,
                _ => MARGIN_MM,
            };
            let f = if line.bold { &font_bold } else { &font };
            layer.use_text(line.text.clone(), line.size, Mm(x), Mm(y), f);
        }
        y -= LINE_HEIGHT_MM;
    }

    let export_dir = "exports";
    std::fs::create_dir_all(export_dir)?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let safe_bill = detail.bill_no.replace(
        |c: char| !c.is_alphanumeric() && c != '-',
        "_",
    );
    let filename = format!("bill_{}_{}.pdf", safe_bill, ts);
    let path = Path::new(export_dir).join(&filename);

    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

    Ok(path.to_string_lossy().to_string())
}
