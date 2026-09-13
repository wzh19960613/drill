use super::*;
use crate::pdf::document;

fn item(order: Option<Vec<usize>>) -> PrintItem {
    let question = serde_json::from_str::<Question>(
        r#"{
            "id": "P1-1", "source": "s1", "subject": "", "origin": "",
            "locate": "P1-1", "chapter": "c", "qtype": "选择题",
            "stem": [], "options": [
                {"id": 1, "text": "a"}, {"id": 2, "text": "b"},
                {"id": 3, "text": "c"}],
            "correct_id": 2, "correct_ids": [2], "answer": [],
            "solution": [], "notes": []
        }"#,
    )
    .unwrap();
    PrintItem {
        question,
        seq: 1,
        option_order: order,
    }
}

#[test]
fn display_options_follow_order() {
    let it = item(Some(vec![2, 0, 1]));
    let opts = display_options(&it);
    assert_eq!(opts[0], ('A', "c".to_string()));
    assert_eq!(opts[1], ('B', "a".to_string()));
    assert_eq!(opts[2], ('C', "b".to_string()));
}

#[test]
fn display_options_fall_back_on_invalid_orders() {
    let plain: Vec<(char, String)> = display_options(&item(None));
    for order in [
        vec![99, 0, 0],
        vec![0, 0, 1],
        vec![0, 1],
        vec![0, 1, 2, 3],
    ] {
        let got = display_options(&item(Some(order.clone())));
        assert_eq!(got, plain, "order {order:?} must fall back to in-order");
    }
}

#[test]
fn more_than_26_options_get_a_placeholder_letter() {
    let mut it = item(None);
    it.question.options = (0..28)
        .map(|i| crate::model::QOption {
            id: (i + 1) as u8,
            text: format!("o{i}"),
        })
        .collect();
    let opts = display_options(&it);
    assert_eq!(opts.len(), 28);
    assert_eq!(opts[25].0, 'Z');
    assert_eq!(opts[26].0, '?', "no overflow past Z");
    assert_eq!(opts[27].0, '?');
}

#[test]
fn answer_letters_are_remapped_and_sorted() {
    let it = item(Some(vec![1, 0, 2]));
    assert_eq!(answer_letters(&it), Some(vec!['A']));
}

#[test]
fn answer_letters_none_without_order() {
    let it = item(None);
    assert_eq!(answer_letters(&it), None);
}

#[test]
fn answer_letters_none_when_order_length_mismatches() {
    let it = item(Some(vec![0]));
    assert_eq!(answer_letters(&it), None);
}

#[test]
fn answer_letters_none_on_out_of_range_order() {
    let it = item(Some(vec![99, 0, 0]));
    assert_eq!(answer_letters(&it), None);
}

#[test]
fn answer_letters_none_without_any_correct_id() {
    let mut it = item(Some(vec![0, 1, 2]));
    it.question.correct_id = None;
    it.question.correct_ids = vec![];
    assert_eq!(answer_letters(&it), None);
}

#[test]
fn answer_letters_multi_choice_sorted() {
    let mut it = item(Some(vec![2, 1, 0]));
    it.question.correct_ids = vec![1, 3];
    it.question.correct_id = Some(1);
    assert_eq!(answer_letters(&it), Some(vec!['A', 'C']));
}

#[test]
fn answer_letters_unknown_id_yields_placeholder() {
    let mut it = item(Some(vec![0, 1, 2]));

    it.question.correct_ids = vec![255];
    it.question.correct_id = Some(255);
    assert_eq!(answer_letters(&it), Some(vec!['?']));
}

#[test]
fn defaults_fill_missing_fields() {
    let p: ExportPayload =
        serde_json::from_str(r#"{"doc":"workbook","paper":"A4","date":"","items":[]}"#)
            .unwrap();
    assert_eq!(p.per_page, 2);
    assert_eq!(
        (p.margin_t, p.margin_b, p.margin_l, p.margin_r),
        (14.0, 14.0, 16.0, 16.0)
    );
    assert_eq!(p.sep_q.style, LineStyle::Dashed);
    assert_eq!(p.sep_foot.style, LineStyle::None);
    assert!(p.show_meta);
    assert_eq!(p.font_scale, 1.0);
    assert_eq!(p.opts_per_row, 4);
    assert!(!p.binding_long);
    assert_eq!(p.title, "");
}

#[test]
fn unknown_paper_falls_back_to_a4() {
    let p: ExportPayload =
        serde_json::from_str(r#"{"doc":"workbook","paper":"A3","date":"","items":[]}"#)
            .unwrap();
    assert_eq!(p.paper, Paper::A4);
    assert_eq!(document::paper_size(&p.paper), (210.0, 297.0));
}

#[test]
fn paper_defaults_and_b5() {
    let p: ExportPayload = serde_json::from_str(r#"{"doc":"workbook","items":[]}"#).unwrap();
    assert_eq!(p.paper, Paper::A4, "absent paper defaults to A4");
    let p: ExportPayload =
        serde_json::from_str(r#"{"doc":"workbook","paper":"B5","items":[]}"#).unwrap();
    assert_eq!(p.paper, Paper::B5);
    assert_eq!(document::paper_size(&p.paper), (176.0, 250.0));
    assert_eq!(p.paper.to_string(), "B5");
}

#[test]
fn line_style_wire_names_roundtrip() {
    for (wire, style) in [
        ("none", LineStyle::None),
        ("solid", LineStyle::Solid),
        ("dashed", LineStyle::Dashed),
        ("dotted", LineStyle::Dotted),
        ("double", LineStyle::Double),
        ("thickThin", LineStyle::ThickThin),
    ] {
        let spec: LineSpec =
            serde_json::from_str(&format!(r#"{{"style":"{wire}","width":1,"gap":2}}"#))
                .unwrap();
        assert_eq!(spec.style, style, "wire {wire}");
        let back = serde_json::to_value(&spec).unwrap();
        assert_eq!(back["style"], json_str(wire));
    }

    let spec: LineSpec = serde_json::from_str(r#"{"style":"fancy"}"#).unwrap();
    assert_eq!(spec.style, LineStyle::Solid);
}

fn json_str(s: &str) -> serde_json::Value {
    serde_json::Value::String(s.to_string())
}
