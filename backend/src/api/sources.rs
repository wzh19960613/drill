use axum::extract::{Path as AxPath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::bad_request;
use crate::app::App;
use crate::parsing;

#[derive(Serialize)]
pub struct SourceOut {
    id: String,
    name: String,
    path: String,
    exists: bool,
    count: usize,
}

pub async fn list(State(app): State<App>) -> Json<Vec<SourceOut>> {
    let out = app
        .sources
        .list()
        .into_iter()
        .map(|s| SourceOut {
            count: parsing::load_questions_from(&s.path, &s.id).len(),
            exists: s.path.is_dir(),
            path: s.path.to_string_lossy().into_owned(),
            id: s.id,
            name: s.name,
        })
        .collect();
    Json(out)
}

#[derive(Deserialize)]
pub struct SourcePath {
    pub path: String,
}

fn source_json(s: &crate::store::Source) -> serde_json::Value {
    json!({ "id": s.id, "name": s.name, "path": s.path })
}

pub async fn add(State(app): State<App>, Json(body): Json<SourcePath>) -> Response {
    match app.sources.add(&body.path, &app.root) {
        Ok(s) => (StatusCode::CREATED, Json(source_json(&s))).into_response(),
        Err(e) => bad_request(&e),
    }
}

pub async fn relocate(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<SourcePath>,
) -> Response {
    match app.sources.relocate(&id, &body.path, &app.root) {
        Ok(s) => (StatusCode::OK, Json(source_json(&s))).into_response(),
        Err(e) if e == "unknown source id" => (StatusCode::NOT_FOUND, e).into_response(),
        Err(e) => bad_request(&e),
    }
}

pub async fn remove(State(app): State<App>, AxPath(id): AxPath<String>) -> Response {
    match app.sources.remove(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) if e == "unknown source id" => (StatusCode::NOT_FOUND, e).into_response(),
        Err(e) => bad_request(&e),
    }
}
