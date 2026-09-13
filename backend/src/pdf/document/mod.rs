mod blocks;
mod page;
mod preamble;
mod question;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use super::markup::{paragraph, Paragraph};
use super::payload::{ExportPayload, Paper};

pub(crate) use page::{measure_width, page_capacity};
pub(crate) use preamble::{preamble, qblock_override};
pub(crate) use question::question_content;

pub fn paper_size(paper: &super::payload::Paper) -> (f64, f64) {
    match paper {
        Paper::A4 => (210.0, 297.0),
        Paper::B5 => (176.0, 250.0),
    }
}

pub type ImageSlots = HashMap<String, (String, String)>;

pub fn collect_images(payload: &ExportPayload) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for item in &payload.items {
        for src in item
            .question
            .stem
            .iter()
            .chain(item.question.answer.iter())
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
    doc.push_str(&super::layout::text_env(payload));
    doc.push_str(&qblock_override(payload));
    doc.push_str(&custom_style(payload));
    let capacity = page_capacity(payload);
    let continuous =
        payload.per_page == 0 && !payload.binding_long && page::hf_constant(payload, total);
    for (i, group) in pages.iter().enumerate() {
        append_group(
            &mut doc, payload, images, group, heights, i, total, capacity, continuous, w, h,
        );
    }
    doc
}

fn custom_style(payload: &ExportPayload) -> String {
    match &payload.custom_style {
        Some(style) if !style.trim().is_empty() => format!("\n{style}\n"),
        _ => String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn append_group(
    doc: &mut String,
    payload: &ExportPayload,
    images: &ImageSlots,
    group: &[usize],
    heights: &[f64],
    i: usize,
    total: usize,
    capacity: f64,
    continuous: bool,
    w: f64,
    h: f64,
) {
    let page_no = i + 1;
    let texts = super::jsslots::page_texts(payload, page_no, total);
    let bind_left = page::binding_left(payload, page_no);
    let (open, close) = group_frame(payload, &texts, bind_left, w, h, i, page_no, continuous);
    doc.push_str(&open);
    for (index, height) in group
        .iter()
        .zip(question_heights(group, heights, capacity, continuous))
    {
        doc.push_str(&question::question_block(
            &payload.items[*index],
            payload,
            images,
            height,
            capacity,
        ));
    }
    doc.push_str(&close);
}

#[allow(clippy::too_many_arguments)]
fn group_frame(
    payload: &ExportPayload,
    texts: &super::jsslots::PageTexts,
    bind_left: bool,
    w: f64,
    h: f64,
    i: usize,
    page_no: usize,
    continuous: bool,
) -> (String, String) {
    if i > 0 && continuous {
        return (String::new(), String::new());
    }
    let open = format!(
        "\n{}",
        page::page_def(payload, texts, bind_left, w, h, i == 0, page_no)
    );
    if i == 0 {
        (open, String::new())
    } else {
        let mut open = open.trim_end().to_string();
        open.push('[');
        (open, "]\n".to_string())
    }
}

fn question_heights(
    group: &[usize],
    heights: &[f64],
    capacity: f64,
    continuous: bool,
) -> Vec<f64> {
    if continuous {
        return vec![f64::INFINITY; group.len()];
    }
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
    let bump = ((capacity - used) / n).max(0.0);
    needs.iter().map(|need| need + bump).collect()
}
