use axum::body::Bytes;
use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use super::bad_request;
use crate::app::App;
use crate::fsutil::{is_safe_component, log_warn};
use crate::images::{image_ext, magic_matches};

pub(crate) const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
const SWEEP_AGE: Duration = Duration::from_secs(24 * 3600);

static UPLOAD_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
pub struct UploadQuery {
    pub name: String,
}

pub async fn upload(State(app): State<App>, Query(q): Query<UploadQuery>, body: Bytes) -> Response {
    sweep(&app.tmp_dir);
    let original = q.name.trim();
    let Some(ext) = image_ext(original) else {
        return bad_request("only svg / png / jpg / jpeg / webp / gif images are allowed");
    };
    if body.len() > app.max_upload_bytes {
        return bad_request(&format!(
            "image larger than {} MB",
            app.max_upload_bytes / (1024 * 1024)
        ));
    }
    if !magic_matches(ext, &body) {
        return bad_request("file content does not match its extension");
    }
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = UPLOAD_SEQ.fetch_add(1, Ordering::Relaxed);
    let id = format!("u{nanos:x}-{seq}.{ext}");
    let path = app.tmp_dir.join(&id);
    if let Err(e) = fs::create_dir_all(&app.tmp_dir).and_then(|()| fs::write(&path, &body)) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("cannot store upload: {e}"),
        )
            .into_response();
    }
    Json(json!({ "id": id, "name": original })).into_response()
}

pub async fn temp_image(State(app): State<App>, AxPath(id): AxPath<String>) -> Response {
    if !is_safe_component(&id) {
        return bad_request("invalid upload id");
    }
    let Some(ext) = image_ext(&id) else {
        return bad_request("not an image");
    };
    let path = app.tmp_dir.join(&id);
    let Ok(bytes) = fs::read(&path) else {
        return (StatusCode::NOT_FOUND, "upload not found").into_response();
    };
    super::assets::image_response(ext, bytes)
}

pub async fn discard(State(app): State<App>, AxPath(id): AxPath<String>) -> Response {
    if !is_safe_component(&id) || image_ext(&id).is_none() {
        return bad_request("invalid upload id");
    }
    match fs::remove_file(app.tmp_dir.join(&id)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "upload not found").into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("cannot remove upload: {e}"),
        )
            .into_response(),
    }
}

pub fn sweep(dir: &std::path::Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let cutoff = SystemTime::now()
        .checked_sub(SWEEP_AGE)
        .unwrap_or(SystemTime::UNIX_EPOCH);
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let stale = meta.modified().map(|m| m < cutoff).unwrap_or(false);
        if stale {
            if let Err(e) = fs::remove_file(entry.path()) {
                log_warn(&format!(
                    "[warn] cannot sweep stale upload {}: {e}",
                    entry.path().display()
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::test_app;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 1, 2];

    fn urlencode(s: &str) -> String {
        s.bytes().map(|b| format!("%{b:02X}")).collect()
    }

    async fn upload(app: &App, name: &str, bytes: &[u8]) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/uploads?name={}", urlencode(name)))
            .header("content-type", "application/octet-stream")
            .body(Body::from(bytes.to_vec()))
            .unwrap();
        let resp = crate::api::router(app.clone()).oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, v)
    }

    async fn plain(app: &App, method: &str, uri: &str) -> StatusCode {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        crate::api::router(app.clone())
            .oneshot(req)
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn upload_serve_discard_lifecycle() {
        let app = test_app("uploads");

        assert_eq!(upload(&app, "a.md", b"x").await.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            upload(&app, "fake.png", b"not a png").await.0,
            StatusCode::BAD_REQUEST
        );

        let (status, v) = upload(&app, "截图 1.png", PNG).await;
        assert_eq!(status, StatusCode::OK, "upload accepted");
        let id = v["id"].as_str().unwrap().to_string();
        assert!(id.ends_with(".png"), "id carries the ext: {id}");
        assert_eq!(v["name"], "截图 1.png", "original name echoed back");
        assert!(app.tmp_dir.join(&id).is_file(), "stored on disk");

        assert_eq!(
            plain(&app, "GET", &format!("/api/tmp/{}", urlencode(&id))).await,
            StatusCode::OK
        );

        assert_eq!(plain(&app, "GET", "/api/tmp/..").await, StatusCode::BAD_REQUEST);

        assert_eq!(
            plain(&app, "DELETE", &format!("/api/uploads/{}", urlencode(&id))).await,
            StatusCode::NO_CONTENT
        );
        assert!(!app.tmp_dir.join(&id).exists());
        assert_eq!(
            plain(&app, "DELETE", &format!("/api/uploads/{}", urlencode(&id))).await,
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn sweep_removes_only_stale_files() {
        let dir = std::env::temp_dir().join(format!("drill-sweep-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("fresh.png"), b"x").unwrap();
        let stale = dir.join("stale.png");
        fs::write(&stale, b"x").unwrap();
        let old = SystemTime::now() - Duration::from_secs(48 * 3600);
        fs::File::options()
            .write(true)
            .open(&stale)
            .unwrap()
            .set_modified(old)
            .unwrap();

        sweep(&dir);
        assert!(dir.join("fresh.png").exists());
        assert!(!stale.exists(), "old upload swept");
        let _ = fs::remove_dir_all(&dir);
    }
}
