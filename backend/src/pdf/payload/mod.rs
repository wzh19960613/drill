use serde::{Deserialize, Serialize};
use std::fmt;

use crate::model::Question;

pub const DOC_WORKBOOK: &str = "workbook";
pub const DOC_ANSWERS: &str = "answers";

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

    pub fn is_double(&self) -> bool {
        matches!(self.style, LineStyle::Double | LineStyle::ThickThin)
    }

    pub fn double_widths(&self) -> (f64, f64) {
        match self.style {
            LineStyle::Double => (self.width, self.width),

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

fn letter_at(pos: usize) -> char {
    if pos < 26 {
        char::from(b'A' + pos as u8)
    } else {
        '?'
    }
}

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
mod tests;
