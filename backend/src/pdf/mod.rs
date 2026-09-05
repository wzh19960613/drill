//! Content, option shuffling and layout mirror the browser print page.

mod document;
mod fonts;
mod jsslots;
mod latex;
mod layout;
mod markup;
mod payload;
pub mod svg;

use std::sync::{Arc, LazyLock};

use anyhow::{anyhow, Result};

use crate::fsutil::{log_line, log_warn};
use typst::foundations::{Dict, Value};
use typst_as_lib::TypstEngine;

pub use payload::{ExportPayload, DOC_ANSWERS};

static REGISTRY: LazyLock<Arc<fonts::SourceRegistry>> =
    LazyLock::new(|| Arc::new(fonts::SourceRegistry::default()));
static ENGINE: LazyLock<TypstEngine> = LazyLock::new(|| fonts::build_engine(REGISTRY.clone()));
static RENDER_SEQ: LazyLock<std::sync::atomic::AtomicU64> =
    LazyLock::new(|| std::sync::atomic::AtomicU64::new(0));

fn debug_typst_enabled() -> bool {
    std::env::var("DRILL_DEBUG_TYPST").is_ok()
}

pub fn render(payload: &ExportPayload, source_dirs: &[std::path::PathBuf]) -> Result<Vec<u8>> {
    let names = document::collect_images(payload);
    let (slots, inputs) = load_images(&names, source_dirs);
    let heights = measure_question_heights(payload, &slots, &inputs);
    let pages = layout::repack(&heights, payload);
    let source = document::build_document(payload, &slots, &pages, &heights);
    if debug_typst_enabled() {
        if let Err(e) = std::fs::write("/tmp/drill-final.typ", &source) {
            log_warn(&format!("[warn] cannot write /tmp/drill-final.typ: {e}"));
        }
    }
    let doc = compile(&source, inputs)?;
    typst_pdf::pdf(&doc, &Default::default()).map_err(|e| anyhow!("typst pdf export failed: {e:?}"))
}

/// Measure one height per question, falling back to zero heights (static
/// per-page grouping) when the measuring document fails.
fn measure_question_heights(
    payload: &ExportPayload,
    slots: &document::ImageSlots,
    inputs: &Dict,
) -> Vec<f64> {
    let source = layout::build_measure_source(payload, slots);
    let debug = debug_typst_enabled();
    match compile(&source, inputs.clone()) {
        Ok(doc) => {
            let h = layout::extract_heights(&doc);
            if debug {
                log_line(&format!(
                    "[debug] measured {} heights for {} items: {:?}",
                    h.len(),
                    payload.items.len(),
                    h.iter()
                        .enumerate()
                        .map(|(i, v)| (i + 1, (v * 10.0).round() / 10.0))
                        .collect::<Vec<_>>()
                ));
            }
            h
        }
        Err(e) => {
            log_warn(&format!(
                "[warn] height measuring failed, using static pagination: {e:#}"
            ));
            vec![0.0; payload.items.len()]
        }
    }
}

/// Compile one generated source with a unique virtual path. The virtual
/// source is removed from the registry afterwards so repeated exports do not
/// leak the full document text.
fn compile(source: &str, inputs: Dict) -> Result<typst_layout::PagedDocument> {
    // unique virtual path per compile so parallel exports never clash
    let n = RENDER_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let main_path = format!("exports/gen-{n}/main.typ");
    REGISTRY.set(&main_path, source.to_string());
    let warned =
        ENGINE.compile_with_input::<_, _, typst_layout::PagedDocument>(main_path.as_str(), inputs);
    REGISTRY.remove(&main_path);
    if !warned.warnings.is_empty() {
        log_warn(&format!(
            "[warn] typst: {} warning(s) during compile",
            warned.warnings.len()
        ));
    }
    warned
        .output
        .map_err(|e| anyhow!("typst compile failed: {e}"))
}

pub(crate) fn load_images(
    names: &[String],
    source_dirs: &[std::path::PathBuf],
) -> (document::ImageSlots, Dict) {
    let mut slots = document::ImageSlots::default();
    let mut inputs = Dict::new();
    for name in names {
        let Some(svg) = svg::load_svg(name, source_dirs) else {
            log_warn(&format!("[warn] export svg not found in sources: {name}"));
            continue;
        };
        let key = format!("img{}", inputs.len());
        inputs.insert(
            key.clone().into(),
            Value::Bytes(typst::foundations::Bytes::new(svg.as_bytes().to_vec())),
        );
        slots.insert(name.clone(), key);
    }
    (slots, inputs)
}

pub fn safe_filename(s: &str) -> String {
    let out: String = s
        .chars()
        .map(|ch| {
            let ok = ch.is_alphanumeric()
                || matches!(ch, '-' | '_')
                || ('\u{4e00}'..='\u{9fff}').contains(&ch);
            if ok {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "export".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use payload::PrintItem;

    fn payload(doc: &str, items: Vec<PrintItem>) -> ExportPayload {
        serde_json::from_value(serde_json::json!({
            "doc": doc, "paper": "A4", "perPage": 2,
            "date": "2026-09-03", "title": "测试题本", "bookId": null,
            "fontScale": 1, "optsPerRow": 4, "items": items_value(items),
        }))
        .unwrap()
    }

    fn items_value(items: Vec<PrintItem>) -> serde_json::Value {
        serde_json::to_value(items).unwrap()
    }

    fn sample_item() -> PrintItem {
        serde_json::from_value(serde_json::json!({
            "id": "P29-10", "source": "s1", "subject": "数学", "origin": "示例题库",
            "locate": "P29-10", "chapter": "第3章 概念", "qtype": "选择题",
            "stem": ["设 $f(x)=\\begin{cases}x>0\\\\ x\\leqslant 0\\end{cases}$，则（　）。"],
            "options": [
                {"id": 1, "text": "不连续"},
                {"id": 2, "text": "连续，但不可导"},
                {"id": 3, "text": "可导，但导函数不连续"},
                {"id": 4, "text": "可导，且导函数连续"}],
            "correct_id": 4, "correct_ids": [4], "answer_line": "**(D)**。",
            "solution": ["连续性：$\\lim\\limits_{x\\to0}x^2=0$。"], "notes": [],
            "seq": 1, "optionOrder": [2, 0, 3, 1]
        }))
        .unwrap()
    }

    #[test]
    fn safe_filename_keeps_cjk() {
        assert_eq!(safe_filename("题本/1:*.pdf"), "题本-1---pdf");
        assert_eq!(safe_filename("///"), "export");
    }

    #[test]
    fn renders_workbook_pdf() {
        let payload = payload("workbook", vec![sample_item()]);
        let bytes = render(&payload, &[]).expect("render should succeed");
        assert!(bytes.starts_with(b"%PDF"), "must be a pdf");
    }

    #[test]
    fn renders_answers_pdf() {
        let payload = payload("answers", vec![sample_item()]);
        let bytes = render(&payload, &[]).expect("render should succeed");
        assert!(bytes.starts_with(b"%PDF"), "must be a pdf");
    }
}
