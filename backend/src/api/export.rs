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
    // serde_json sorts object keys, so the serialization is a stable cache key
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
        // cache miss but the display name is taken: another export with the
        // same title/paper already lives there, so disambiguate with a hash
        // suffix instead of overwriting (and cache-poisoning) it
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

/// Two-round FNV-style hash with the length mixed in; stable within a process
/// and enough to distinguish near-identical payloads.
fn content_hash(s: &str) -> String {
    let (mut h1, mut h2) = (0xcbf29ce484222325u64, 0x9e3779b97f4a7c15u64);
    for b in s.as_bytes() {
        h1 ^= *b as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 = h2.wrapping_add(*b as u64).rotate_left(5) ^ h2.wrapping_mul(0x2545F4914F6CDD1D);
    }
    format!("{h1:016x}{h2:016x}{}", s.len())
}

/// Serve a previously rendered export file by name (download endpoint).
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

/// RFC 5987 percent-encoding over UTF-8 bytes (not code points: encoding a
/// CJK char as a single `%9898`-style escape would produce an invalid header).
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
mod tests {
    use super::*;
    use crate::api::test_support::test_app;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn post(app: &App, uri: &str, body: &str) -> Response {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        crate::api::router(app.clone()).oneshot(req).await.unwrap()
    }

    #[test]
    fn content_hash_is_deterministic_and_distinguishes_neighbors() {
        assert_eq!(content_hash(""), content_hash(""));
        assert_eq!(content_hash("abc"), content_hash("abc"));
        assert_ne!(content_hash("abc"), content_hash("abd"));
        assert_ne!(content_hash("abc"), content_hash("abcd"));
        assert_ne!(content_hash("a"), content_hash("aa"));
        assert_ne!(content_hash("题"), content_hash("题 "));
        assert!(content_hash("").len() >= 32);
    }

    #[test]
    fn percent_encode_cjk_by_utf8_bytes() {
        // 题 = E9 A2 98, 本 = E6 9C AC
        assert_eq!(percent_encode("题本.pdf"), "%E9%A2%98%E6%9C%AC.pdf");
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode(""), "");
        assert_eq!(percent_encode("A-Z_09.txt"), "A-Z_09.txt");
        assert!(percent_encode("题").is_ascii(), "output stays ASCII");
    }

    #[tokio::test]
    async fn unsafe_file_names_are_rejected() {
        let app = test_app("exportfetch");
        // call the handler directly: multi-segment names never reach it
        // through the router ({name} does not match '/')
        for name in ["..", "a/b.pdf", ".hidden.pdf", "notes.txt", ""] {
            let resp = export_file(State(app.clone()), AxPath(name.to_string())).await;
            assert_eq!(
                resp.status(),
                axum::http::StatusCode::BAD_REQUEST,
                "{name:?}"
            );
        }
    }

    #[tokio::test]
    async fn non_object_payloads_are_rejected() {
        let app = test_app("export400");
        for body in ["[1,2]", "\"string\"", "42", "null"] {
            let resp = post(&app, "/api/export/pdf", body).await;
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "export body {body}");
            let resp = post(&app, "/api/preview/pdf", body).await;
            assert_eq!(
                resp.status(),
                StatusCode::BAD_REQUEST,
                "preview body {body}"
            );
        }
        // object bodies still pass validation and reach the payload checks
        let resp = post(&app, "/api/export/pdf", r#"{ "doc": "diary" }"#).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "doc must be workbook/answers"
        );
    }

    #[tokio::test]
    async fn same_display_name_different_content_does_not_overwrite() {
        let app = test_app("exportname");
        let spec = ExportSpec {
            name: "题本-题本-A4.pdf".into(),
            book: Some("b1".into()),
        };
        let hash1 = "aaaaaaaabbbbbbbbccccccccdddddddd0";
        let hash2 = "bbbbbbbbccccccccddddddddeeeeeeee0";

        let r1 = store_export(&app, hash1, &spec, b"first pdf".to_vec());
        assert_eq!(r1.status(), StatusCode::CREATED);
        let r2 = store_export(&app, hash2, &spec, b"second pdf".to_vec());
        assert_eq!(r2.status(), StatusCode::CREATED);

        async fn json_of(resp: Response) -> serde_json::Value {
            let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap();
            serde_json::from_slice(&bytes).unwrap()
        }
        let v1 = json_of(r1).await;
        let v2 = json_of(r2).await;
        assert_eq!(v1["name"], "题本-题本-A4.pdf");
        assert_ne!(
            v1["name"], v2["name"],
            "collision must pick a distinct name"
        );
        assert!(v2["name"].as_str().unwrap().starts_with("题本-题本-A4-"));
        assert!(v2["name"]
            .as_str()
            .unwrap()
            .ends_with(&format!("-{}.pdf", &hash2[..8])));

        // both files survive, each hash resolves to its own file
        let exports = app.data_dir.join("exports");
        assert_eq!(
            std::fs::read(exports.join(v1["name"].as_str().unwrap())).unwrap(),
            b"first pdf"
        );
        assert_eq!(
            std::fs::read(exports.join(v2["name"].as_str().unwrap())).unwrap(),
            b"second pdf"
        );
        assert_eq!(
            app.export_cache.lookup(hash1).unwrap(),
            exports.join("题本-题本-A4.pdf")
        );
        assert_eq!(
            app.export_cache.lookup(hash2).unwrap(),
            exports.join(v2["name"].as_str().unwrap())
        );
    }

    #[tokio::test]
    async fn identical_content_hits_the_cache_file() {
        let app = test_app("exportcache");
        let spec = ExportSpec {
            name: "题本-题本-B5.pdf".into(),
            book: None,
        };
        let hash = content_hash("{\"doc\":\"workbook\"}");
        let r1 = store_export(&app, &hash, &spec, b"same pdf".to_vec());
        assert_eq!(r1.status(), StatusCode::CREATED);
        // same content again: no suffix, cache reports the same file
        let r2 = store_export(&app, &hash, &spec, b"same pdf".to_vec());
        assert_eq!(r2.status(), StatusCode::CREATED);
        let bytes = axum::body::to_bytes(r2.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["name"], "题本-题本-B5.pdf");
    }
}
