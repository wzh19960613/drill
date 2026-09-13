mod assets;
mod books;
mod export;
mod files;
pub(crate) mod export_cache;
mod progress;
mod questions;
mod records;
mod sources;
pub(crate) mod uploads;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::app::App;
use crate::fsutil::log_line;

pub(super) fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, msg.to_string()).into_response()
}

pub(super) fn server_error(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}

pub(super) enum ApiError {
    BadRequest(String),
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
        }
    }
}

impl ApiError {
    pub(super) fn bad_request(msg: impl Into<String>) -> ApiError {
        ApiError::BadRequest(msg.into())
    }

    pub(super) fn not_found(msg: impl Into<String>) -> ApiError {
        ApiError::NotFound(msg.into())
    }
}

pub fn router(app: App) -> Router {
    let max_upload = app.max_upload_bytes;
    question_routes()
        .merge(record_routes())
        .merge(book_routes())
        .merge(source_routes())
        .merge(upload_routes(max_upload))
        .merge(progress_routes())
        .merge(export_routes())
        .fallback(assets::static_route)
        .with_state(app)
        .layer(axum::middleware::from_fn(log_requests))
}

async fn log_requests(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let resp = next.run(req).await;
    log_line(&format!(
        "[http] {} {} -> {}",
        method,
        path,
        resp.status().as_u16()
    ));
    resp
}

fn question_routes() -> Router<App> {
    Router::new()
        .route("/api/questions", get(questions::list))
        .route("/api/questions/preview", post(questions::preview))
        .route("/api/questions/save", post(questions::save))
        .route(
            "/api/questions/{id}",
            delete(questions::remove),
        )
        .route("/api/questions/{id}/raw", get(questions::raw))
}

fn record_routes() -> Router<App> {
    Router::new()
        .route("/api/records", get(records::list).post(records::add))
        .route(
            "/api/records/{id}",
            put(records::update).delete(records::remove),
        )
}

fn book_routes() -> Router<App> {
    Router::new()
        .route("/api/books", get(books::list).post(books::save))
        .route(
            "/api/books/active",
            get(books::get_active).put(books::set_active),
        )
        .route(
            "/api/books/favorite",
            get(books::get_favorite).put(books::set_favorite),
        )
        .route("/api/books/{id}", delete(books::remove))
}

fn source_routes() -> Router<App> {
    Router::new()
        .route("/api/sources", get(sources::list).post(sources::add))
        .route(
            "/api/sources/{id}",
            put(sources::update).delete(sources::remove),
        )
        .route("/api/sources/{id}/browse", get(sources::browse))
        .route("/api/sources/{id}/resolve-images", post(sources::resolve_images))
        .route(
            "/api/sources/{id}/file",
            get(files::get).put(files::put).delete(files::delete),
        )
        .route("/api/sources/{id}/mark", put(files::mark))
        .route("/api/sources/{id}/mark-folder", put(files::mark_folder))
}

fn upload_routes(max_upload: usize) -> Router<App> {
    Router::new()
        .route("/api/uploads", post(uploads::upload))
        .route("/api/uploads/{id}", delete(uploads::discard))
        .route("/api/tmp/{id}", get(uploads::temp_image))

        .layer(axum::extract::DefaultBodyLimit::max(
            max_upload.saturating_mul(2).max(4 * 1024 * 1024),
        ))
}

fn progress_routes() -> Router<App> {
    Router::new()
        .route(
            "/api/mastery",
            get(progress::get_mastery).put(progress::put_mastery),
        )
        .route(
            "/api/paused",
            get(progress::get_paused)
                .put(progress::put_paused)
                .delete(progress::delete_paused),
        )
}

fn export_routes() -> Router<App> {
    Router::new()
        .route("/api/export/pdf", post(export::export_pdf))
        .route("/api/preview/pdf", post(export::preview_pdf))
        .route("/api/exports/{name}", get(export::export_file))
        .route("/api/assets/{name}", get(assets::asset))
}

#[cfg(test)]
pub(super) mod test_support {
    use std::sync::Arc;

    use tokio::sync::Mutex as AsyncMutex;

    use crate::api::export_cache::ExportCache;
    use crate::app::App;
    use crate::store::{BookStore, MasteryStore, PausedStore, RecordStore, SourceStore};

    pub fn test_app(tag: &str) -> App {
        app_with_book_cap(tag, BookStore::max_books())
    }

    pub fn test_app_capped(tag: &str, cap: usize) -> App {
        app_with_book_cap(tag, cap)
    }

    fn app_with_book_cap(tag: &str, cap: usize) -> App {
        let dir = std::env::temp_dir().join(format!("drill-api-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let bank = dir.join("bank");
        std::fs::create_dir_all(&bank).unwrap();
        std::fs::write(
            bank.join("P1-1.md"),
            "> [数学] 测试 | 第1章 @ P1-1 > 选择题\n\n题干（　）。\n\n(A) 甲\n\n(B) 乙\n\n## 答案\n\n**(B)**。\n",
        )
        .unwrap();
        let sources = SourceStore::load(dir.join("sources.json"), None);
        let s1 = sources.add(bank.to_str().unwrap(), &dir).unwrap();
        assert_eq!(s1.id, "s1");
        App {
            root: dir.clone(),
            dist: dir.join("dist"),
            data_dir: dir.clone(),
            host: "127.0.0.1".into(),
            port: 0,
            records: Arc::new(RecordStore::load(dir.join("records.json"))),
            books: Arc::new(BookStore::load_with_cap(dir.join("books.json"), cap)),
            sources: Arc::new(sources),
            mastery: Arc::new(MasteryStore::load(dir.join("mastery.json"))),
            paused: Arc::new(PausedStore::load(dir.join("paused.json"))),
            export_cache: Arc::new(ExportCache::new(dir.join("exports"))),
            pdf_lock: Arc::new(AsyncMutex::new(())),
            tmp_dir: dir.join("tmp-uploads"),
            max_upload_bytes: super::uploads::MAX_UPLOAD_BYTES,
        }
    }
}
