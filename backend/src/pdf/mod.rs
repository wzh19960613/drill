mod document;
mod fonts;
mod jsslots;
mod latex;
mod layout;
mod markup;
mod payload;
pub mod svg;
#[cfg(test)]
mod tests;

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

fn compile(source: &str, inputs: Dict) -> Result<typst_layout::PagedDocument> {
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
        let Some(fmt) = svg::load_image(name, source_dirs) else {
            log_warn(&format!("[warn] export image not found in sources: {name}"));
            continue;
        };
        let key = format!("img{}", inputs.len());
        inputs.insert(
            key.clone().into(),
            Value::Bytes(typst::foundations::Bytes::new(fmt.0)),
        );
        slots.insert(name.clone(), (key, fmt.1.to_string()));
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
