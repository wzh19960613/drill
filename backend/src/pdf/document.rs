use std::collections::{HashMap, HashSet};

use super::jsslots;
use super::latex;
use super::markup::{escape_markup, paragraph, Chunk, Paragraph};
use super::payload::{answer_letters, display_options, ExportPayload, LineStyle, Paper, PrintItem};
use crate::fsutil::log_warn;

/// Page size in mm (A4 / B5, same as the print page).
pub fn paper_size(paper: &Paper) -> (f64, f64) {
    match paper {
        Paper::A4 => (210.0, 297.0),
        Paper::B5 => (176.0, 250.0),
    }
}

pub type ImageSlots = HashMap<String, String>;

pub fn collect_images(payload: &ExportPayload) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for item in &payload.items {
        for src in item
            .question
            .stem
            .iter()
            .chain(item.question.solution.iter())
            .chain(item.question.notes.iter())
        {
            if let Paragraph::Image { name } = paragraph(src) {
                if seen.insert(name.clone()) {
                    names.push(name);
                }
            }
        }
    }
    names
}

pub fn build_document(
    payload: &ExportPayload,
    images: &ImageSlots,
    pages: &[Vec<usize>],
    heights: &[f64],
) -> String {
    let total = pages.len();
    let (w, h) = paper_size(&payload.paper);
    let mut doc = String::with_capacity(64 * 1024);
    doc.push_str(preamble());
    // identical text environment to the measuring document
    doc.push_str(&super::layout::text_env(payload));
    doc.push_str(&qblock_override(payload));
    if let Some(style) = &payload.custom_style {
        if !style.trim().is_empty() {
            doc.push('\n');
            doc.push_str(style);
            doc.push('\n');
        }
    }
    let capacity = page_capacity(payload);

    for (i, group) in pages.iter().enumerate() {
        let page_no = i + 1;
        let texts = jsslots::page_texts(payload, page_no, total);
        let bind_left = binding_left(payload, page_no);
        let cap = capacity;
        let n = group.len().max(1) as f64;
        let needs: Vec<f64> = group
            .iter()
            .map(|&index| {
                heights
                    .get(index)
                    .map(|mh| mh + super::layout::MEASURE_PAD)
                    .unwrap_or(20.0)
            })
            .collect();
        let used: f64 = needs.iter().sum::<f64>() + super::layout::GAP * n;
        let bump = ((cap - used) / n).max(0.0);
        let mut open = format!("\n{}", page_def(payload, &texts, bind_left, w, h, i == 0));
        let mut close = String::new();
        if i > 0 {
            open = open.trim_end().to_string();
            open.push('[');
            close = "]
"
            .to_string();
        }
        doc.push_str(&open);
        for (&index, &need) in group.iter().zip(needs.iter()) {
            doc.push_str(&question_block(
                &payload.items[index],
                payload,
                images,
                need + bump,
            ));
        }
        doc.push_str(&close);
    }
    doc
}

fn page_def(
    payload: &ExportPayload,
    texts: &jsslots::PageTexts,
    bind_left: bool,
    w: f64,
    h: f64,
    first: bool,
) -> String {
    let (ml, mr) = h_margins(payload, bind_left);
    let mt = payload.margin_t.max(5.0);
    let mb = payload.margin_b.max(5.0);
    let hf = |slots: &(String, String, String), spec: &super::payload::LineSpec, below: bool| {
        let (b, c, f) = (
            escape_markup(&slots.0),
            escape_markup(&slots.1),
            escape_markup(&slots.2),
        );
        let row = format!("hf-row(([{b}], [{c}], [{f}]), bindLeft: {bind_left})");
        if spec.is_none() {
            let inset = if below { "bottom: 1.5mm" } else { "top: 1.5mm" };
            return format!("block(width: 100%, inset: ({inset}), {row})");
        }
        if spec.is_double() {
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
            return format!("block(width: 100%, inset: (y: 1.5mm), {inner})");
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
    };
    format!(
        "\n{func}(width: {w}mm, height: {h}mm,\n  \
         margin: (top: {mt}mm, bottom: {mb}mm, left: {ml}mm, right: {mr}mm),\n  \
         header: {header}, footer: {footer})\n",
        func = if first { "#set page" } else { "#page" },
        header = hf(&texts.header, &payload.sep_head, true),
        footer = hf(&texts.footer, &payload.sep_foot, false),
    )
}

fn dash_of(spec: &super::payload::LineSpec) -> String {
    match spec.style {
        LineStyle::Dotted => format!("({g:.2}pt, {g:.2}pt)", g = spec.gap.max(0.5)),
        LineStyle::Dashed => format!("(2pt, {g:.2}pt)", g = spec.gap.max(0.5)),
        _ => "\"solid\"".to_string(),
    }
}

fn double_lines(spec: &super::payload::LineSpec, dy1: f64, dy2: f64) -> String {
    let (w1, w2) = spec.double_widths();
    format!(
        "place(bottom + left, dy: {dy1}mm, line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w1:.2}pt))) \
         + place(bottom + left, dy: {dy2}mm, line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w2:.2}pt)))"
    )
}

fn qblock_override(payload: &ExportPayload) -> String {
    let spec = &payload.sep_q;
    let open = "#let qblock(h, body) = block(\n  width: 100%, height: h, breakable: false,\n  above: 2mm, below: 0mm,\n  inset: (top: 1mm, bottom: 2mm),\n";
    if spec.is_none() {
        return format!("{open}  body,\n)\n");
    }
    if spec.is_double() {
        return format!(
            "{open}  body + ({rules}),\n)\n",
            rules = double_lines(spec, 1.6, 0.0)
        );
    }
    format!(
        "{open}  stroke: (bottom: (paint: rgb(\"#b9bcc4\"), thickness: {w:.2}pt, dash: {dash})),\n  body,\n)\n",
        w = spec.width,
        dash = dash_of(spec),
    )
}

fn binding_left(payload: &ExportPayload, page_no: usize) -> bool {
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

fn h_margins(payload: &ExportPayload, bind_left: bool) -> (f64, f64) {
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

/// Header/footer rows live inside the page margins, so unlike the
/// browser-print page no extra footer strip is reserved (a small rounding
/// safety remains).
pub(crate) fn page_capacity(payload: &ExportPayload) -> f64 {
    let (_, paper_h) = paper_size(&payload.paper);
    let content_h = paper_h - payload.margin_t.max(5.0) - payload.margin_b.max(5.0);
    content_h - (content_h * 0.01).max(2.0)
}

pub(crate) fn preamble() -> &'static str {
    r##"
#import "/mitex/prelude.typ": *
#import "/mitex/standard.typ": package
#let mitex-scope = package.scope
// box(): inline math must never break across lines
#let mi(res) = box(eval("$" + res + "$", scope: mitex-scope))
#let mm(res) = eval(mode: "markup", "$ " + res + " $", scope: mitex-scope)

#let qblock(h, body) = block(
  width: 100%, height: h, breakable: false,
  above: 2mm, below: 0mm,
  inset: (top: 1mm, bottom: 2mm),
  stroke: (bottom: (paint: rgb("#b9bcc4"), thickness: 0.6pt, dash: (2pt, 2pt))),
  body,
)

// Shrink option columns until no single option would overflow its column
// (print page `fitOptions`).
#let fitopts(target, avail, opts) = context {
  let ladder = if target == 4 { (4, 2, 1) } else if target == 2 { (2, 1) } else { (1,) }
  let widths = opts.map(o => measure(box(o), width: 100000pt).width)
  let cols = ladder.find(c => {
    let colw = (avail - (c - 1) * 2mm) / c
    widths.all(w => w <= colw)
  })
  let cols = if cols == none { 1 } else { cols }
  grid(columns: (1fr,) * cols, column-gutter: 2mm, row-gutter: 2mm, ..opts)
}

#let hf-row(slots, .., bindLeft: false) = {
  let s = if bindLeft { slots } else { (slots.at(2), slots.at(1), slots.at(0)) }
  grid(
    columns: (1fr, auto, 1fr),
    align: (left, center, right),
    text(size: 0.71em, fill: rgb("#999999"), s.at(0)),
    text(size: 0.71em, fill: rgb("#999999"), s.at(1)),
    text(size: 0.71em, fill: rgb("#999999"), s.at(2)),
  )
}

#let anslabel(label) = {
  v(1.4mm)
  block(below: 1.8mm, text(size: 0.857em, fill: rgb("#999999"), weight: 600, tracking: 1.5pt, label))
}

#let ansbox(body) = block(
  width: 100%, radius: 4.5pt, above: 4mm, below: 4mm,
  stroke: 1pt + rgb("#b9bcc4"),
  outset: (y: 1.5mm), inset: (x: 2mm, y: 2mm),
  align(center, text(size: 1.2em, weight: 700, body)),
)

#let quoteblock(body) = block(
  width: 100%, above: 2.5mm, below: 2mm,
  inset: (left: 2mm, y: 1mm),
  stroke: (left: 1pt + rgb("#cccccc")),
  text(fill: rgb("#666666"), body),
)

#let fig(key) = image(sys.inputs.at(key), format: "svg", width: 44mm)
"##
}

fn question_block(
    item: &PrintItem,
    payload: &ExportPayload,
    images: &ImageSlots,
    h: f64,
) -> String {
    format!(
        "#qblock({h:.2}mm)[\n{}]\n",
        question_content(item, payload, images)
    )
}

/// Render one question's body, shared by the final document and the
/// measuring document.
pub(crate) fn question_content(
    item: &PrintItem,
    payload: &ExportPayload,
    images: &ImageSlots,
) -> String {
    let mut body = String::new();
    body.push_str(&meta_line(item, payload));
    body.push_str(&question_lead(item, images));
    if payload.is_workbook() {
        body.push_str(&options(item, payload));
    } else {
        body.push_str(&options(item, payload));
        body.push_str(&answers(item, images));
    }
    body
}

fn question_lead(item: &PrintItem, images: &ImageSlots) -> String {
    let mut out = format!("#strong[{}.]", item.seq);
    let mut started = false;
    for src in &item.question.stem {
        match paragraph(src) {
            Paragraph::Image { name } => {
                started |= emit_lead_block(
                    &mut out,
                    images.get(&name).map(|k| format!("#fig(\"{k}\")\n\n")),
                );
            }
            Paragraph::Quote(chunks) => {
                let inner = chunks_markup(&chunks);
                started |= emit_lead_block(&mut out, Some(format!("#quoteblock[{inner}]\n\n")));
            }
            Paragraph::Body(chunks) => {
                let inner = chunks_markup(&chunks);
                if started {
                    out.push_str(&inner);
                    out.push_str("\n\n");
                } else {
                    out.push(' ');
                    out.push_str(&inner);
                    out.push_str("\n\n");
                    started = true;
                }
            }
        }
    }
    if !started {
        out.push_str("\n\n");
    }
    out
}

fn emit_lead_block(out: &mut String, markup: Option<String>) -> bool {
    match markup {
        Some(markup) => {
            out.push_str("\n\n");
            out.push_str(&markup);
            true
        }
        None => false,
    }
}

fn meta_line(item: &PrintItem, payload: &ExportPayload) -> String {
    if !payload.show_meta {
        return String::new();
    }
    let q = &item.question;
    let subject = if q.subject.is_empty() {
        "未知"
    } else {
        q.subject.as_str()
    };
    let meta =
        super::jsslots::question_meta(payload, item.seq as usize, &q.chapter, &q.locate, subject);
    let text = escape_markup(&format!("{subject} · {meta}"));
    format!("#align(right, text(size: 0.71em, fill: rgb(\"#999999\"))[{text}])\n#v(0.5mm)\n")
}

fn stem(paragraphs: &[String], images: &ImageSlots) -> String {
    let mut out = String::new();
    for src in paragraphs {
        match paragraph(src) {
            Paragraph::Image { name } => {
                if let Some(key) = images.get(&name) {
                    out.push_str(&format!("#fig(\"{key}\")\n\n"));
                } else {
                    log_warn(&format!("[warn] export svg not found in sources: {name}"));
                }
            }
            Paragraph::Quote(chunks) => {
                let inner = chunks_markup(&chunks);
                out.push_str(&format!("#quoteblock[{inner}]\n\n"));
            }
            Paragraph::Body(chunks) => {
                let inner = chunks_markup(&chunks);
                out.push_str(&inner);
                out.push_str("\n\n");
            }
        }
    }
    out
}

fn chunks_markup(chunks: &[Chunk]) -> String {
    let mut out = String::new();
    for chunk in chunks {
        match chunk {
            Chunk::Text(text) => out.push_str(text),
            Chunk::MathInline { latex } => out.push_str(&latex::inline(latex.as_str())),
            Chunk::MathBlock { latex } => {
                out.push('\n');
                out.push_str(&latex::block(latex.as_str()));
                out.push('\n');
            }
        }
    }
    out
}

fn options(item: &PrintItem, payload: &ExportPayload) -> String {
    let opts = display_options(item);
    if opts.is_empty() {
        return String::new();
    }
    let (w, _) = paper_size(&payload.paper);
    let avail = w - measure_width(payload);
    let cells: Vec<String> = opts
        .iter()
        .map(|(letter, text)| {
            let body = one_paragraph(text);
            format!("[#strong[（{letter}）]{body}]")
        })
        .collect();
    format!(
        "#v(2.5mm)\n#fitopts({target}, {avail:.1}mm, ({}))\n\n",
        cells.join(", "),
        target = payload.opts_per_row
    )
}

fn answers(item: &PrintItem, images: &ImageSlots) -> String {
    let q = &item.question;
    let mut out = String::new();
    if !q.answer_line.is_empty() {
        out.push_str(&format!("#ansbox[{}]\n", answer_content(item)));
    }
    if !q.solution.is_empty() {
        out.push_str(&stem(&q.solution, images));
    }
    if !q.notes.is_empty() {
        out.push_str("#anslabel[备注]\n");
        for src in &q.notes {
            let gray = format!("#text(fill: rgb(\"#666666\"))[{}]", one_paragraph(src));
            out.push_str(&gray);
            out.push_str("\n\n");
        }
    }
    out
}

/// Render the answer box: the mapped option text when the shuffle order is
/// known, otherwise the raw answer line (frontend `answerHtml`).
fn answer_content(item: &PrintItem) -> String {
    match answer_letters(item) {
        Some(letters) if letters.len() == 1 => {
            let letter = letters[0];
            let text = display_options(item)
                .into_iter()
                .find(|(l, _)| *l == letter)
                .map(|(_, text)| text)
                .unwrap_or_default();
            format!(
                "（{letter}）{}",
                one_paragraph_inner(&strip_trailing_period(&text), true)
            )
        }
        Some(letters) => letters.iter().collect::<String>(),
        None => one_paragraph_inner(&strip_trailing_period(&item.question.answer_line), true),
    }
}

fn one_paragraph_inner(src: &str, display_fractions: bool) -> String {
    let mut out = String::new();
    for chunk in super::markup::split_math(src) {
        match chunk {
            Chunk::Text(text) => out.push_str(&text),
            Chunk::MathInline { latex } => out.push_str(&if display_fractions {
                latex::inline_display(latex.as_str())
            } else {
                latex::inline(latex.as_str())
            }),
            Chunk::MathBlock { latex } => out.push_str(&latex::block(latex.as_str())),
        }
    }
    out
}

fn one_paragraph(src: &str) -> String {
    one_paragraph_inner(src, false)
}

/// The print page strips one trailing period from the answer line.
fn strip_trailing_period(src: &str) -> String {
    src.trim_end()
        .trim_end_matches(['。', '.'])
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_with_stem(paragraphs: &[&str]) -> ExportPayload {
        let stem: Vec<String> = paragraphs.iter().map(|s| s.to_string()).collect();
        serde_json::from_value(serde_json::json!({
            "doc": "workbook", "paper": "A4", "perPage": 2, "items": [{
                "id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
                "locate": "P1", "chapter": "c", "qtype": "选择题",
                "stem": stem, "options": [], "correct_id": null,
                "correct_ids": [], "answer_line": "", "solution": [], "notes": []
            }]
        }))
        .unwrap()
    }

    #[test]
    fn collect_images_dedups_while_keeping_first_seen_order() {
        let payload = item_with_stem(&["![[b.svg]]", "text", "![[a.svg]]", "![[b.svg]]"]);
        assert_eq!(
            collect_images(&payload),
            vec!["b.svg".to_string(), "a.svg".to_string()]
        );
    }

    #[test]
    fn collect_images_across_solution_and_notes() {
        let mut payload = item_with_stem(&["![[s.svg]]"]);
        payload.items[0].question.solution = vec!["![[n.svg]]".into()];
        payload.items[0].question.notes = vec!["![[s.svg]]".into()];
        assert_eq!(
            collect_images(&payload),
            vec!["s.svg".to_string(), "n.svg".to_string()]
        );
    }

    fn binding_payload(
        binding_long: bool,
        first_page_left: bool,
        l: f64,
        r: f64,
        extra: f64,
    ) -> ExportPayload {
        serde_json::from_value(serde_json::json!({
            "doc": "workbook", "paper": "A4",
            "bindingLong": binding_long, "firstPageLeft": first_page_left,
            "marginL": l, "marginR": r, "bindingExtra": extra,
            "items": []
        }))
        .unwrap()
    }

    #[test]
    fn binding_side_truth_table() {
        let mut p = binding_payload(true, false, 20.0, 10.0, 4.0);
        assert!(binding_left(&p, 1), "odd pages bind left by default");
        assert!(!binding_left(&p, 2));
        assert!(binding_left(&p, 3));
        p.first_page_left = true;
        assert!(!binding_left(&p, 1), "first page left flips the parity");
        assert!(binding_left(&p, 2));
        p.binding_long = false;
        assert!(!binding_left(&p, 1), "no long binding: never shifts");
        assert!(!binding_left(&p, 2));
    }

    #[test]
    fn h_margins_truth_table() {
        let p = binding_payload(false, false, 20.0, 10.0, 4.0);
        assert_eq!(h_margins(&p, true), (20.0, 10.0), "no binding: raw margins");
        assert_eq!(h_margins(&p, false), (20.0, 10.0));

        let p = binding_payload(true, false, 20.0, 10.0, 4.0);
        // inner = max(l, r) + extra, outer = min(l, r)
        assert_eq!(
            h_margins(&p, true),
            (24.0, 10.0),
            "bind left: inner on the left"
        );
        assert_eq!(h_margins(&p, false), (10.0, 24.0));

        let p = binding_payload(true, false, 8.0, 9.0, 0.0);
        assert_eq!(h_margins(&p, false), (8.0, 9.0));
        let p = binding_payload(false, false, 2.0, 3.0, 0.0);
        assert_eq!(
            h_margins(&p, true),
            (5.0, 5.0),
            "margins are floored at 5mm"
        );
    }

    #[test]
    fn strip_trailing_period_edges() {
        assert_eq!(strip_trailing_period("42"), "42");
        assert_eq!(strip_trailing_period("42。。"), "42");
        assert_eq!(strip_trailing_period("42.。"), "42");
        assert_eq!(strip_trailing_period("42 "), "42", "trailing space trimmed");
        assert_eq!(strip_trailing_period("…"), "…", "ellipsis is not a period");
        assert_eq!(strip_trailing_period(""), "");
        assert_eq!(strip_trailing_period("。"), "");
    }
}
