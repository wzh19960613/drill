//! Two-pass pagination, mirroring the browser print page.

use super::document;
use super::payload::ExportPayload;

/// Page-content constants of the print page (mm).
pub(crate) const GAP: f64 = 2.0;
/// Conservative bonus over measured heights to absorb rounding differences.
pub(crate) const MEASURE_PAD: f64 = 1.5;

/// Text environment shared by the measuring and the final document: both
/// MUST render with identical text/paragraph settings, otherwise the
/// measured heights will not match the real layout.
pub(crate) fn text_env(payload: &ExportPayload) -> String {
    let base = 10.5 * payload.font_scale.max(0.5);
    format!(
        "#set text(font: (\"Noto Sans SC\", \"Libertinus Serif\", \"Noto Emoji\"), size: {base:.2}pt, lang: \"zh\")\n\
         #set par(justify: false, leading: 1.2em, spacing: 1.45em)\n"
    )
}

/// Source of the measuring document: one question per tall page, with an
/// `H<height>X` marker rendered next to each block for extraction.
pub fn build_measure_source(payload: &ExportPayload, images: &document::ImageSlots) -> String {
    let (w, _) = document::paper_size(&payload.paper);
    let mx = document::measure_width(payload);
    let tb = payload.margin_t.max(5.0);
    let mut doc = String::with_capacity(16 * 1024);
    doc.push_str(document::preamble());
    doc.push_str(&format!(
        "\n#set page(width: {w}mm, height: 1200mm, margin: (top: {tb}mm, bottom: {tb}mm, x: {mx2:.1}mm))\n{}\n",
        text_env(payload),
        mx2 = mx / 2.0,
        tb = tb
    ));
    for item in &payload.items {
        doc.push_str(&format!(
            "#context [H#str(calc.round(measure(block(\n\
              inset: (top: 1mm, bottom: 2mm),\n\
              width: 100%,\n\
              [\n{}\n]), width: {w}mm - {mx:.1}mm).height / 1pt, digits: 2))X]\n",
            document::question_content(item, payload, images),
            mx = mx,
        ));
        doc.push_str("\n#pagebreak()\n");
    }
    doc
}

/// Pull the `H<value>X` markers out of the measuring document, in page order.
pub fn extract_heights(doc: &typst_layout::PagedDocument) -> Vec<f64> {
    let mut heights = Vec::new();
    for page in doc.pages() {
        let mut text = String::new();
        collect_text(&page.frame, &mut text);
        if let Some(value) = parse_marker(&text) {
            heights.push(value * 25.4 / 72.0);
        }
    }
    heights
}

fn collect_text(frame: &typst::layout::Frame, out: &mut String) {
    for (_, item) in frame.items() {
        match item {
            typst::layout::FrameItem::Text(text) => out.push_str(&text.text),
            typst::layout::FrameItem::Group(group) => collect_text(&group.frame, out),
            _ => {}
        }
    }
}

fn parse_marker(text: &str) -> Option<f64> {
    let start = text.find('H')? + 1;
    let end = text[start..].find('X')? + start;
    text[start..end].parse().ok()
}

/// Greedy packing of questions into pages (frontend `repack`).
/// Each page takes up to `perPage` questions while the measured heights plus
/// the gaps between blocks still fit the page capacity.
pub fn repack(heights: &[f64], payload: &ExportPayload) -> Vec<Vec<usize>> {
    let capacity = document::page_capacity(payload);
    let per_page = if payload.per_page == 0 {
        usize::MAX
    } else {
        payload.per_page.max(1) as usize
    };
    let min_h = block_min_h(capacity, per_page);

    let mut pages: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut used = 0.0;
    for (index, &height) in heights.iter().enumerate() {
        let page_min = min_h;
        let h = page_min.max(height + MEASURE_PAD);
        let full = current.len() >= per_page;
        let exhausted = !current.is_empty() && used + h > capacity + 0.01;
        if full || exhausted {
            pages.push(std::mem::take(&mut current));
            used = 0.0;
        }
        current.push(index);
        used += h + GAP;
    }
    if !current.is_empty() {
        pages.push(current);
    }
    pages
}

fn block_min_h(available: f64, per_page: usize) -> f64 {
    ((available - (per_page as f64 - 1.0) * GAP) / per_page as f64).max(20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_respecting_per_page_and_capacity() {
        let payload: ExportPayload =
            serde_json::from_str(r#"{"doc":"workbook","paper":"A4","perPage":2,"items":[]}"#)
                .unwrap();
        let heights = vec![200.0; 3];
        let pages = repack(&heights, &payload);
        assert_eq!(
            pages.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![1, 1, 1]
        );
        let pages = repack(&[10.0; 5], &payload);
        assert_eq!(
            pages.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![2, 2, 1]
        );
    }

    #[test]
    fn marker_parsing() {
        assert_eq!(parse_marker("前 H123.45X 后"), Some(123.45));
        assert_eq!(parse_marker("none"), None);
        assert_eq!(parse_marker("H 1 X"), None, "space breaks the marker");
        assert_eq!(parse_marker("HX"), None, "empty value does not parse");
    }

    #[test]
    fn repack_edge_cases() {
        let p: ExportPayload =
            serde_json::from_str(r#"{"doc":"workbook","paper":"A4","perPage":2,"items":[]}"#)
                .unwrap();
        assert!(repack(&[], &p).is_empty());
        assert_eq!(repack(&[9999.0], &p), vec![vec![0]]);
        let p0: ExportPayload =
            serde_json::from_str(r#"{"doc":"workbook","paper":"A4","perPage":0,"items":[]}"#)
                .unwrap();
        let pages = repack(&[10.0; 40], &p0);
        assert_eq!(
            pages.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![12, 12, 12, 4]
        );
    }

    #[test]
    fn repack_respects_capacity_boundary() {
        let p: ExportPayload =
            serde_json::from_str(r#"{"doc":"workbook","paper":"A4","perPage":2,"items":[]}"#)
                .unwrap();
        let cap = document::page_capacity(&p);
        let h = (cap - 2.0 * GAP) / 2.0 - MEASURE_PAD;
        assert_eq!(repack(&[h, h], &p), vec![vec![0, 1]]);
        assert_eq!(repack(&[h * 2.0, h], &p), vec![vec![0], vec![1]]);
    }
}
