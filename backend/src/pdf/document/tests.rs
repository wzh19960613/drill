use super::blocks::{block_markup, chunks_markup, display_length, stem};
use super::page::{binding_left, h_margins};
use super::super::markup::{self, Paragraph};
use super::{build_document, collect_images, ExportPayload, ImageSlots};

fn item_with_stem(paragraphs: &[&str]) -> ExportPayload {
    let stem: Vec<String> = paragraphs.iter().map(|s| s.to_string()).collect();
    serde_json::from_value(serde_json::json!({
        "doc": "workbook", "paper": "A4", "perPage": 2, "items": [{
            "id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
            "locate": "P1", "chapter": "c", "qtype": "选择题",
            "stem": stem, "options": [], "correct_id": null,
            "correct_ids": [], "answer": [], "solution": [], "notes": []
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
fn stem_keeps_consecutive_headings() {
    let out = stem(
        &[
            "**（1）**".to_string(),
            "**（2）**".to_string(),
            "正文随后。".to_string(),
            "### 小节一".to_string(),
            "### 小节二".to_string(),
        ],
        &ImageSlots::default(),
    );
    assert!(out.contains("（1）"), "first bold lead kept");
    assert!(out.contains("（2）"), "second bold lead kept");
    assert!(out.contains("正文随后"));
    assert!(out.contains("小节一"), "first heading kept");
    assert!(out.contains("小节二"), "second heading kept");
}

#[test]
fn code_blocks_stay_verbatim_and_survive_inner_fences() {
    let out = block_markup(&Paragraph::Code {
        text: "a[0] = x_1  #include <stdio.h>\n```nested\nx\n```".to_string(),
    });
    assert!(out.contains("a[0]"), "verbatim, no escape backslashes: {out}");
    assert!(out.contains("#include <stdio.h>"), "hash not escaped: {out}");
    assert!(out.starts_with("````"), "fence longer than the inner ``` run");
    assert!(out.ends_with("````\n\n"), "closing fence matches");
}

#[test]
fn consecutive_display_math_merges_into_one_equation() {
    let chunks = markup::split_math("$$a=1$$\n$$b=2$$");
    let out = chunks_markup(&chunks);
    assert!(
        out.contains("#block(width: 100%, above: 0.4em, below: 0.4em)[#mm("),
        "one tightened group expected: {out}"
    );
}

#[test]
fn blank_line_separated_display_math_stays_two_blocks() {
    let srcs = ["$$a=1$$".to_string(), "$$b=2$$".to_string()];
    let out = stem(&srcs, &ImageSlots::default());
    assert_eq!(
        out.matches("#mm(").count(),
        2,
        "two paragraphs stay separate: {out}"
    );
}

#[test]
fn continuous_flow_blocked_by_page_or_total_dependent_footers() {
    let mut p = item_with_stem(&["题干______。"]);
    p.per_page = 0;
    let pages = vec![vec![0usize], vec![0usize]];
    let heights = vec![60.0_f64];

    let doc = build_document(&p, &ImageSlots::default(), &pages, &heights);
    assert!(
        !doc.contains("#page("),
        "default footer keeps the continuous flow"
    );

    p.footer.flip = "(c) => c.page + '/' + c.totalPages".to_string();
    let doc = build_document(&p, &ImageSlots::default(), &pages, &heights);
    assert!(
        doc.contains("#page("),
        "page-varying footer blocks the flow"
    );

    p.footer.flip = "(c) => '共 ' + c.totalPages + ' 页'".to_string();
    let doc = build_document(&p, &ImageSlots::default(), &pages, &heights);
    assert!(
        doc.contains("#page("),
        "total-tracking footer blocks the flow"
    );
}

#[test]
fn display_length_measures_visible_answer() {
    assert_eq!(display_length("42。"), 3);
    assert!(display_length(r"$\dfrac{1}{2}$") <= 3);
    assert!(display_length(r"$x^2+y^2$") <= 6);
    assert_eq!(display_length("**(A) 甲**"), 5);
    assert_eq!(display_length("![[图.svg]]"), 0);
    assert!(display_length("一二三四五六七八九十一二三四五六七八九十一") >= 20);
}
