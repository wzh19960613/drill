use std::fs;

use axum::extract::{Path as AxPath, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use super::{bad_request, server_error, ApiError};
use crate::app::App;
use crate::fsutil::{atomic_write, is_safe_component, log_line, log_warn};
use crate::pdf::{self, ExportPayload, DOC_ANSWERS};

struct ExportSpec {
    name: String,
    book: Option<String>,
}

fn spec_for(payload: &ExportPayload) -> ExportSpec {
    let label = if payload.is_workbook() {
        "题本"
    } else {
        "答案本"
    };
    let title = if payload.title.is_empty() {
        "题本"
    } else {
        &payload.title
    };
    ExportSpec {
        name: format!(
            "{}-{label}-{}.pdf",
            pdf::safe_filename(title),
            payload.paper
        ),
        book: payload.book_id.clone(),
    }
}

pub async fn export_pdf(State(app): State<App>, Json(payload): Json<Value>) -> Response {

    let canonical = serde_json::to_string(&payload).unwrap_or_default();
    let parsed = match parse_payload(&canonical) {
        Ok(p) if p.is_workbook() || p.doc == DOC_ANSWERS => p,
        Ok(_) => return bad_request("doc must be workbook or answers"),
        Err(e) => return e.into_response(),
    };
    let spec = spec_for(&parsed);
    let hash = content_hash(&canonical);
    if let Some(path) = app.export_cache.lookup(&hash) {
        let bytes = fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0);
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        return created(&name, bytes, true);
    }
    render_and_store(&app, &parsed, &spec, &hash).await
}

pub async fn preview_pdf(State(app): State<App>, Json(payload): Json<Value>) -> Response {
    let canonical = serde_json::to_string(&payload).unwrap_or_default();
    let parsed = match parse_payload(&canonical) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let _guard = app.pdf_lock.lock().await;
    let source_dirs: Vec<_> = app.sources.list().iter().map(|s| s.path.clone()).collect();
    match pdf::render(&parsed, &source_dirs) {
        Ok(bytes) => {
            let mut resp = Response::new(bytes.into());
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/pdf"),
            );
            resp
        }
        Err(e) => server_error(&format!("render failed: {e:#}")),
    }
}

async fn render_and_store(
    app: &App,
    payload: &ExportPayload,
    spec: &ExportSpec,
    hash: &str,
) -> Response {
    let _guard = app.pdf_lock.lock().await;
    let source_dirs: Vec<_> = app.sources.list().iter().map(|s| s.path.clone()).collect();
    let bytes = match pdf::render(payload, &source_dirs) {
        Ok(b) => b,
        Err(e) => {
            log_warn(&format!("[warn] background pdf failed: {e:#}"));
            return server_error(&format!("render failed: {e:#}"));
        }
    };
    store_export(app, hash, spec, bytes)
}

fn parse_payload(json: &str) -> Result<ExportPayload, ApiError> {
    serde_json::from_str(json).map_err(|e| ApiError::bad_request(format!("invalid payload: {e}")))
}

fn store_export(app: &App, hash: &str, spec: &ExportSpec, bytes: Vec<u8>) -> Response {
    let exports = app.data_dir.join("exports");
    if let Err(e) = fs::create_dir_all(&exports) {
        return server_error(&format!("write failed: {e}"));
    }
    let mut name = spec.name.clone();
    let plain = exports.join(&name);
    let plain_is_ours = app.export_cache.lookup(hash).as_deref() == Some(plain.as_path());
    if plain.exists() && !plain_is_ours {

        let stem = name.strip_suffix(".pdf").unwrap_or(&name);
        name = format!("{stem}-{}.pdf", &hash[..8]);
    }
    let path = exports.join(&name);
    if let Err(e) = atomic_write(&path, &bytes) {
        return server_error(&format!("write failed: {e}"));
    }
    app.export_cache.insert(hash, &name, spec.book.as_deref());
    log_line(&format!(
        "[pdf] wrote {} ({} bytes)",
        path.display(),
        bytes.len()
    ));
    created(&name, bytes.len(), false)
}

fn created(name: &str, bytes: usize, cached: bool) -> Response {
    (
        StatusCode::CREATED,
        Json(json!({ "url": format!("/api/exports/{name}"), "name": name, "bytes": bytes, "cached": cached })),
    )
        .into_response()
}

fn content_hash(s: &str) -> String {
    let (mut h1, mut h2) = (0xcbf29ce484222325u64, 0x9e3779b97f4a7c15u64);
    for b in s.as_bytes() {
        h1 ^= *b as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_add(*b as u64).rotate_left(5) ^ h2.wrapping_mul(0x2545F4914F6CDD1D);
    }
    format!("{h1:016x}{h2:016x}{}", s.len())
}

pub async fn export_file(State(app): State<App>, AxPath(name): AxPath<String>) -> Response {
    if !is_safe_component(&name) || !name.ends_with(".pdf") {
        return bad_request("invalid file name");
    }
    let path = app.data_dir.join("exports").join(&name);
    match fs::read(&path) {
        Ok(bytes) => pdf_response(name, bytes),
        Err(_) => (StatusCode::NOT_FOUND, "file not found").into_response(),
    }
}

fn pdf_response(name: String, bytes: Vec<u8>) -> Response {
    let disposition = format!("attachment; filename*=UTF-8''{}", percent_encode(&name));
    let mut resp = Response::new(bytes.into());
    for (k, v) in [
        (header::CONTENT_TYPE, "application/pdf".to_string()),
        (header::CONTENT_DISPOSITION, disposition),
        (header::CACHE_CONTROL, "no-cache".to_string()),
    ] {
        if let Ok(v) = header::HeaderValue::from_str(&v) {
            resp.headers_mut().insert(k, v);
        }
    }
    resp
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests;
