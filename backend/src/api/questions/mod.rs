mod save;
mod save_fs;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use super::{bad_request, server_error, ApiError};
use crate::app::App;
use crate::model::Question;
use crate::parsing;

pub use save::save;

pub async fn list(State(app): State<App>) -> Json<Vec<Question>> {
    let mut questions = Vec::new();
    for src in app.sources.list() {
        let excluded: HashSet<String> = src.excluded.iter().cloned().collect();
        questions.extend(parsing::load_questions_from(
            &src.path,
            &src.id,
            src.recursive,
            &excluded,
        ));
    }
    questions.sort_by(|a, b| {
        parsing::nat_cmp(&a.source, &b.source).then_with(|| parsing::nat_cmp(&a.id, &b.id))
    });
    Json(questions)
}

fn source_dir(app: &App, id: &str) -> Result<PathBuf, ApiError> {
    app.sources
        .get(id)
        .map(|s| s.path)
        .ok_or_else(|| ApiError::not_found("unknown source id"))
}

pub(crate) fn find_question_file(app: &App, id: &str, source: &str) -> Result<PathBuf, ApiError> {
    if !crate::fsutil::is_safe_component(id) {
        return Err(ApiError::bad_request("invalid question id"));
    }
    let src = app
        .sources
        .get(source)
        .ok_or_else(|| ApiError::not_found("unknown source id"))?;
    let excluded: HashSet<String> = src.excluded.iter().cloned().collect();
    parsing::find_question_file(&src.path, id, &excluded)
        .or_else(|| parsing::find_question_by_id(&src.path, id, &excluded))
        .ok_or_else(|| ApiError::not_found("question file not found"))
}

pub(crate) fn rel_under_source(root: &std::path::Path, file: &std::path::Path) -> String {
    let dir = file
        .parent()
        .and_then(|d| d.strip_prefix(root).ok())
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    dir.trim_end_matches('/').to_string()
}

pub async fn raw(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(source) = params.get("source") else {
        return bad_request("source is required");
    };
    let path = match find_question_file(&app, &id, source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let root = match source_dir(&app, source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match fs::read_to_string(&path) {
        Ok(md) => Json(json!({
            "id": id,
            "source": source,
            "markdown": md,
            "dir": rel_under_source(&root, &path),
            "file": path.file_name().and_then(|n| n.to_str()).unwrap_or_default(),
        }))
        .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "question file not found").into_response(),
    }
}

#[derive(Deserialize)]
pub struct PreviewQuestion {
    pub markdown: String,
    pub source: String,
}

pub async fn preview(State(app): State<App>, Json(body): Json<PreviewQuestion>) -> Response {
    let mut q = match parsing::parse_md(&body.markdown) {
        Ok(q) => q,
        Err(e) => return bad_request(&format!("parse failed: {e:#}")),
    };
    if let Err(e) = source_dir(&app, &body.source) {
        return e.into_response();
    }
    q.source = body.source.clone();
    Json(q).into_response()
}

pub async fn remove(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(source) = params.get("source") else {
        return bad_request("source is required");
    };
    let path = match find_question_file(&app, &id, source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match fs::remove_file(&path) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => server_error(&format!("delete failed: {e}")),
    }
}
