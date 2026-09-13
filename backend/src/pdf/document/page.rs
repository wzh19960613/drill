use super::super::jsslots;
use super::super::markup::escape_markup;
use super::super::payload::{ExportPayload, LineSpec};
use super::preamble::dash_of;

pub(crate) fn binding_left(payload: &ExportPayload, page_no: usize) -> bool {
    if !payload.binding_long {
        return false;
    }
    let odd = page_no % 2 == 1;
    if payload.first_page_left {
        !odd
    } else {
        odd
    }
}

pub(crate) fn h_margins(payload: &ExportPayload, bind_left: bool) -> (f64, f64) {
    let l = payload.margin_l.max(5.0);
    let r = payload.margin_r.max(5.0);
    if !payload.binding_long {
        return (l, r);
    }
    let inner = l.max(r) + payload.binding_extra.max(0.0);
    let outer = l.min(r);
    if bind_left {
        (inner, outer)
    } else {
        (outer, inner)
    }
}

pub(crate) fn measure_width(payload: &ExportPayload) -> f64 {
    let base = payload.margin_l.max(5.0) + payload.margin_r.max(5.0);
    if payload.binding_long {
        base + payload.binding_extra.max(0.0)
    } else {
        base
    }
}

pub(crate) fn page_capacity(payload: &ExportPayload) -> f64 {
    let (_, paper_h) = super::paper_size(&payload.paper);
    let content_h = paper_h - payload.margin_t.max(5.0) - payload.margin_b.max(5.0);
    content_h - (content_h * 0.01).max(2.0)
}

pub(super) fn hf_constant(payload: &ExportPayload, total: usize) -> bool {
    let probe = |page: usize, t: usize| jsslots::page_texts(payload, page, t);
    let t1 = probe(1, total);
    let t2 = probe(2, total);
    let alt = probe(1, total + 1);
    let band = |a: &(String, String, String),
                b: &(String, String, String),
                alt: &(String, String, String),
                footer: bool| {
        (a == b && a == alt)
            || (footer && a.0 == alt.0 && a.1 == alt.1 && a.2 == "1" && b.2 == "2")
    };
    total < 2
        || (band(&t1.header, &t2.header, &alt.header, false)
            && band(&t1.footer, &t2.footer, &alt.footer, true))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn page_def(
    payload: &ExportPayload,
    texts: &jsslots::PageTexts,
    bind_left: bool,
    w: f64,
    h: f64,
    first: bool,
    page_no: usize,
) -> String {
    let (ml, mr) = h_margins(payload, bind_left);
    let mt = payload.margin_t.max(5.0);
    let mb = payload.margin_b.max(5.0);
    format!(
        "\n{func}(width: {w}mm, height: {h}mm,\n  \
         margin: (top: {mt}mm, bottom: {mb}mm, left: {ml}mm, right: {mr}mm),\n  \
         header: {header}, footer: {footer})\n",
        func = if first { "#set page" } else { "#page" },
        header = band_block(&texts.header, &payload.sep_head, true, bind_left, page_no),
        footer = band_block(&texts.footer, &payload.sep_foot, false, bind_left, page_no),
    )
}

pub(super) fn band_block(
    slots: &(String, String, String),
    spec: &LineSpec,
    below: bool,
    bind_left: bool,
    page_no: usize,
) -> String {
    let (b, c, f) = (
        escape_markup(&slots.0),
        escape_markup(&slots.1),
        escape_markup(&slots.2),
    );
    let f = if !below && f == page_no.to_string() {
        "#context str(counter(page).get().first())".to_string()
    } else {
        f
    };
    let row = format!("hf-row(([{b}], [{c}], [{f}]), bindLeft: {bind_left})");
    if spec.is_none() {
        let inset = if below { "bottom: 1.5mm" } else { "top: 1.5mm" };
        return format!("block(width: 100%, inset: ({inset}), {row})");
    }
    if spec.is_double() {
        return lined_band(spec, &row, below);
    }
    let (side_inset, side_stroke) = if below {
        ("bottom: 1.5mm", "bottom")
    } else {
        ("top: 1.5mm", "top")
    };
    format!(
        "block(width: 100%, inset: ({side_inset}), stroke: ({side_stroke}: (paint: rgb(\"#b9bcc4\"), thickness: {w:.2}pt, dash: {dash})), {row})",
        w = spec.width,
        dash = dash_of(spec),
    )
}

fn lined_band(spec: &LineSpec, row: &str, below: bool) -> String {
    let (w1, w2) = spec.double_widths();
    let line1 = format!(
        "line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w1:.2}pt))"
    );
    let line2 = format!(
        "line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w2:.2}pt))"
    );
    let inner = if below {
        format!("[#v(0.2mm) #{row} #v(0.8mm) #{line1} #v(0.5mm) #{line2}]")
    } else {
        format!("[#{line1} #v(0.5mm) #{line2} #v(0.8mm) #{row}]")
    };
    format!("block(width: 100%, inset: (y: 1.5mm), {inner})")
}
