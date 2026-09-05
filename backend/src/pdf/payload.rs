//! Field names match the frontend `PrintPayload` (camelCase extras; the
//! question model itself is shared with the REST API layer).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::model::Question;

pub const DOC_WORKBOOK: &str = "workbook";
pub const DOC_ANSWERS: &str = "answers";

/// Paper size; unknown values fall back to A4 (serde `other`), matching the
/// previous silent default.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Paper {
    B5,
    #[default]
    #[serde(other)]
    A4,
}

impl Paper {
    pub fn as_str(self) -> &'static str {
        match self {
            Paper::A4 => "A4",
            Paper::B5 => "B5",
        }
    }
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Separator line style; unknown values fall back to solid, which is what
/// the old string matching already did for them.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LineStyle {
    None,
    Dashed,
    Dotted,
    Double,
    ThickThin,
    #[serde(other)]
    Solid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPayload {
    pub doc: String,
    #[serde(default)]
    pub paper: Paper,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub book_id: Option<String>,
    #[serde(default = "default_sep_q")]
    pub sep_q: LineSpec,
    #[serde(default = "default_sep_head")]
    pub sep_head: LineSpec,
    #[serde(default = "default_sep_foot")]
    pub sep_foot: LineSpec,
    #[serde(default = "default_true")]
    pub show_meta: bool,
    #[serde(default)]
    pub binding_extra: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_t: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_b: f64,
    #[serde(default = "default_margin_lr")]
    pub margin_l: f64,
    #[serde(default = "default_margin_lr")]
    pub margin_r: f64,
    #[serde(default)]
    pub meta: MetaSlots,

    #[serde(default = "default_font_scale")]
    pub font_scale: f64,
    #[serde(default = "default_opts_per_row")]
    pub opts_per_row: u8,
    #[serde(default, rename = "bindingLong")]
    pub binding_long: bool,
    #[serde(default, rename = "firstPageLeft")]
    pub first_page_left: bool,
    #[serde(default)]
    pub header: SlotSet,
    #[serde(default)]
    pub footer: SlotSet,
    #[serde(default, rename = "customStyle")]
    pub custom_style: Option<String>,
    #[serde(default)]
    pub items: Vec<PrintItem>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SlotSet {
    #[serde(default)]
    pub binding: String,
    #[serde(default)]
    pub center: String,
    #[serde(default)]
    pub flip: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LineSpec {
    #[serde(default = "default_line_style")]
    pub style: LineStyle,
    #[serde(default = "default_line_width")]
    pub width: f64,
    #[serde(default = "default_line_gap")]
    pub gap: f64,
}

impl LineSpec {
    pub fn is_none(&self) -> bool {
        matches!(self.style, LineStyle::None)
    }

    /// Double rules draw two strokes (a wide one and a narrow one).
    pub fn is_double(&self) -> bool {
        matches!(self.style, LineStyle::Double | LineStyle::ThickThin)
    }

    pub fn double_widths(&self) -> (f64, f64) {
        match self.style {
            LineStyle::Double => (self.width, self.width),
            // thickThin: heavy stroke outside, hairline inside
            _ => (self.width * 1.8, self.width * 0.6),
        }
    }
}

fn default_line_style() -> LineStyle {
    LineStyle::Dashed
}

fn default_line_width() -> f64 {
    0.6
}

fn default_line_gap() -> f64 {
    2.0
}

fn default_sep_q() -> LineSpec {
    LineSpec {
        style: LineStyle::Dashed,
        width: 0.6,
        gap: 2.0,
    }
}

fn default_sep_head() -> LineSpec {
    LineSpec {
        style: LineStyle::Dashed,
        width: 0.6,
        gap: 2.0,
    }
}

fn default_sep_foot() -> LineSpec {
    LineSpec {
        style: LineStyle::None,
        width: 0.6,
        gap: 2.0,
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetaSlots {
    #[serde(default)]
    pub workbook: String,
    #[serde(default)]
    pub answers: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrintItem {
    #[serde(flatten)]
    pub question: Question,
    pub seq: u32,
    pub option_order: Option<Vec<usize>>,
}

fn default_per_page() -> u32 {
    2
}

fn default_true() -> bool {
    true
}

fn default_margin_tb() -> f64 {
    14.0
}

fn default_margin_lr() -> f64 {
    16.0
}

fn default_font_scale() -> f64 {
    1.0
}

fn default_opts_per_row() -> u8 {
    4
}

impl ExportPayload {
    pub fn is_workbook(&self) -> bool {
        self.doc == DOC_WORKBOOK
    }
}

/// Letter for a display position; positions beyond Z yield '?' instead of
/// wrapping around or overflowing the ASCII range.
fn letter_at(pos: usize) -> char {
    if pos < 26 {
        char::from(b'A' + pos as u8)
    } else {
        '?'
    }
}

/// True when `order` is a permutation of `0..len` (right length, in-range,
/// no repeats) — the only case where shuffling can be honored safely.
fn is_permutation(order: &[usize], len: usize) -> bool {
    if order.len() != len {
        return false;
    }
    let mut seen = vec![false; len];
    order
        .iter()
        .all(|&i| i < len && !std::mem::replace(&mut seen[i], true))
}

pub fn display_options(item: &PrintItem) -> Vec<(char, String)> {
    let opts = &item.question.options;
    let order = item
        .option_order
        .as_deref()
        .filter(|o| is_permutation(o, opts.len()));
    match order {
        Some(order) => order
            .iter()
            .enumerate()
            .map(|(pos, orig)| (letter_at(pos), opts[*orig].text.clone()))
            .collect(),
        None => opts
            .iter()
            .enumerate()
            .map(|(i, o)| (letter_at(i), o.text.clone()))
            .collect(),
    }
}

pub fn answer_letters(item: &PrintItem) -> Option<Vec<char>> {
    let ids: Vec<u8> = if item.question.correct_ids.is_empty() {
        vec![item.question.correct_id?]
    } else {
        item.question.correct_ids.clone()
    };
    if ids.is_empty() {
        return None;
    }
    let order = item.option_order.as_deref()?;
    if !is_permutation(order, item.question.options.len()) {
        return None;
    }
    let mut letters: Vec<char> = ids
        .iter()
        .map(|id| {
            let orig = item
                .question
                .options
                .iter()
                .position(|o| o.id == *id)
                .and_then(|idx| order.iter().position(|o| *o == idx));
            match orig {
                Some(pos) => letter_at(pos),
                None => letter_at(id.saturating_sub(1) as usize),
            }
        })
        .collect();
    letters.sort_unstable();
    Some(letters)
}

#[cfg(test)]
mod tests {
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
                "correct_id": 2, "correct_ids": [2], "answer_line": "",
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
            vec![99, 0, 0],   // out of range (used to panic)
            vec![0, 0, 1],    // duplicate entry
            vec![0, 1],       // length mismatch
            vec![0, 1, 2, 3], // length mismatch
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
        // id 255 is not among the options: position lookup fails and the
        // fallback letter must not overflow the alphabet
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
        // unknown styles keep the old solid fallback
        let spec: LineSpec = serde_json::from_str(r#"{"style":"fancy"}"#).unwrap();
        assert_eq!(spec.style, LineStyle::Solid);
    }

    fn json_str(s: &str) -> serde_json::Value {
        serde_json::Value::String(s.to_string())
    }
}
