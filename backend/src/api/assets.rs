use std::fs;
use std::path::Path;

use axum::extract::{Path as AxPath, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

use super::bad_request;
use crate::app::App;
use crate::fsutil::is_safe_component;
use crate::images::{image_ext, mime_of};
use crate::pdf::svg::strip_dark_media;

pub async fn asset(State(app): State<App>, AxPath(name): AxPath<String>) -> Response {
    if !is_safe_component(&name) || image_ext(&name).is_none() {
        return bad_request("only svg / png / jpg / jpeg / webp / gif assets are served");
    }
    let ext = image_ext(&name).unwrap();
    for src in app.sources.list() {
        if let Some(path) = crate::fsutil::find_recursive(&src.path, &name) {
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            return image_response(ext, bytes);
        }
    }
    (StatusCode::NOT_FOUND, "asset not found").into_response()
}

pub(super) fn image_response(ext: &str, bytes: Vec<u8>) -> Response {
    if ext == "svg" {
        let svg = strip_dark_media(&String::from_utf8_lossy(&bytes));
        (
            [
                (header::CONTENT_TYPE, mime_of(ext)),
                (header::CACHE_CONTROL, "no-cache"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CONTENT_SECURITY_POLICY, "sandbox"),
            ],
            svg,
        )
            .into_response()
    } else {
        (
            [
                (header::CONTENT_TYPE, mime_of(ext)),
                (header::CACHE_CONTROL, "no-cache"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response()
    }
}

pub async fn static_route(State(app): State<App>, uri: Uri) -> Response {
    if !app.dist.join("index.html").is_file() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "frontend not built: run cd frontend && npm install && npm run build",
        )
            .into_response();
    }
    let rel = uri.path().trim_start_matches('/');
    if !rel.is_empty() && !rel.contains("..") {
        let f = app.dist.join(rel);
        if f.is_file() {
            return serve_file(&f);
        }

        if rel.starts_with("assets/") {
            return stale_asset(rel);
        }
    }
    serve_file(&app.dist.join("index.html"))
}

fn stale_asset(rel: &str) -> Response {
    match rel.rsplit('.').next().unwrap_or("") {
        "js" | "mjs" => (
            [
                (header::CONTENT_TYPE, "text/javascript"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            "location.replace('/?asset-refresh=' + Date.now())",
        )
            .into_response(),
        "css" => (
            [
                (header::CONTENT_TYPE, "text/css"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            "/* stale asset */",
        )
            .into_response(),
        _ => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}

fn serve_file(p: &Path) -> Response {
    let Ok(bytes) = fs::read(p) else {
        return (StatusCode::NOT_FOUND, "file not found").into_response();
    };
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime = match ext {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript",
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "json" | "map" => "application/json",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };

    let in_assets = p.components().any(|c| c.as_os_str() == "assets");
    let cache = if in_assets && matches!(ext, "js" | "mjs" | "css" | "woff2" | "woff" | "ttf") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache)],
        bytes,
    )
        .into_response()
}
