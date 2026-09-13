use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::bad_request;
use crate::app::App;
use crate::fsutil::is_safe_rel_path;
use crate::parsing;
use crate::parsing::DirScan;

#[derive(Serialize)]
pub struct SourceOut {
    id: String,
    name: String,
    path: String,
    exists: bool,
    recursive: bool,
    count: usize,
    excluded: Vec<String>,
}

pub async fn list(State(app): State<App>) -> Json<Vec<SourceOut>> {
    let out = app
        .sources
        .list()
        .into_iter()
        .map(|s| {
            let excluded: std::collections::HashSet<String> = s.excluded.iter().cloned().collect();
            SourceOut {
                count: parsing::load_questions_from(&s.path, &s.id, s.recursive, &excluded).len(),
                exists: s.path.is_dir(),
                recursive: s.recursive,
                excluded: s.excluded.clone(),
                path: s.path.to_string_lossy().into_owned(),
                id: s.id,
                name: s.name,
            }
        })
        .collect();
    Json(out)
}

#[derive(Deserialize)]
pub struct SourceUpdate {
    pub path: Option<String>,
    pub recursive: Option<bool>,
}

fn source_json(s: &crate::store::Source) -> serde_json::Value {
    json!({ "id": s.id, "name": s.name, "path": s.path, "recursive": s.recursive })
}

#[derive(Deserialize)]
pub struct SourcePath {
    pub path: String,
}

pub async fn add(State(app): State<App>, Json(body): Json<SourcePath>) -> Response {
    match app.sources.add(&body.path, &app.root) {
        Ok(s) => (StatusCode::CREATED, Json(source_json(&s))).into_response(),
        Err(e) => bad_request(&e),
    }
}

pub async fn update(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<SourceUpdate>,
) -> Response {
    if body.path.is_none() && body.recursive.is_none() {
        return bad_request("nothing to update: path or recursive is required");
    }
    if let Some(path) = &body.path {
        if let Err(e) = app.sources.relocate(&id, path, &app.root) {
            return update_error(e);
        }
    }
    if let Some(recursive) = body.recursive {
        if let Err(e) = app.sources.set_recursive(&id, recursive) {
            return update_error(e);
        }
    }
    match app.sources.get(&id) {
        Some(s) => (StatusCode::OK, Json(source_json(&s))).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown source id").into_response(),
    }
}

fn update_error(e: String) -> Response {
    if e == "unknown source id" {
        (StatusCode::NOT_FOUND, e).into_response()
    } else {
        bad_request(&e)
    }
}

pub async fn remove(State(app): State<App>, AxPath(id): AxPath<String>) -> Response {
    match app.sources.remove(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) if e == "unknown source id" => (StatusCode::NOT_FOUND, e).into_response(),
        Err(e) => bad_request(&e),
    }
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    #[serde(default)]
    pub path: String,
}

#[derive(Serialize)]
pub struct DirInfoOut {
    pub name: String,
    pub questions: usize,
    pub other_md: usize,
    #[serde(rename = "questionsAll")]
    pub questions_subtree: usize,
}

#[derive(Serialize)]
pub struct BrowseOut {
    pub path: String,
    pub dirs: Vec<String>,
    #[serde(rename = "dirInfo")]
    pub dir_info: Vec<DirInfoOut>,
    pub questions: Vec<QuestionFileInfo>,
    pub other_md: Vec<OtherFileInfo>,
    pub files: Vec<String>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct QuestionFileInfo {
    pub file: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct OtherFileInfo {
    pub file: String,
    pub reason: String,
}

pub async fn browse(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(q): Query<BrowseQuery>,
) -> Response {
    let Some(src) = app.sources.get(&id) else {
        return (StatusCode::NOT_FOUND, "unknown source id").into_response();
    };
    if !is_safe_rel_path(&q.path) {
        return bad_request("invalid folder path");
    }
    let dir = src.path.join(&q.path);
    if !dir.is_dir() {
        return (StatusCode::NOT_FOUND, "folder not found").into_response();
    }
    let excluded: std::collections::HashSet<String> = src.excluded.iter().cloned().collect();
    let scan = parsing::scan_dir(&src.path, &dir, &excluded);
    let total = if src.recursive {
        parsing::load_questions_from(&src.path, &src.id, true, &excluded).len()
    } else {
        scan.questions.len()
    };
    Json(browse_out(q.path, scan, total)).into_response()
}

fn browse_out(path: String, scan: DirScan, total: usize) -> BrowseOut {
    BrowseOut {
        path,
        dirs: scan.dirs,
        dir_info: scan
            .dir_info
            .into_iter()
            .map(|d| DirInfoOut {
                name: d.name,
                questions: d.questions,
                other_md: d.other_md,
                questions_subtree: d.questions_subtree,
            })
            .collect(),
        questions: scan
            .questions
            .into_iter()
            .map(|qf| QuestionFileInfo {
                file: qf.file,
                id: qf.id,
            })
            .collect(),
        other_md: scan
            .other_md
            .into_iter()
            .map(|o| OtherFileInfo {
                file: o.file,
                reason: o.reason,
            })
            .collect(),
        files: scan.files,
        total,
    }
}

#[derive(Deserialize)]
pub struct ResolveImages {
    pub names: Vec<String>,
}

#[derive(Serialize)]
pub struct ResolvedImage {
    pub name: String,
    pub found: bool,
    pub source: Option<String>,
    pub dir: String,
}

pub async fn resolve_images(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<ResolveImages>,
) -> Response {
    if app.sources.get(&id).is_none() {
        return (StatusCode::NOT_FOUND, "unknown source id").into_response();
    }
    let sources = app.sources.list();
    let out: Vec<ResolvedImage> = body
        .names
        .into_iter()
        .map(|name| resolve_image(&sources, name))
        .collect();
    Json(out).into_response()
}

fn resolve_image(sources: &[crate::store::Source], name: String) -> ResolvedImage {
    for s in sources {
        if let Some(p) = crate::fsutil::find_recursive(&s.path, &name) {
            let dir = p
                .parent()
                .and_then(|d| d.strip_prefix(&s.path).ok())
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default();
            return ResolvedImage {
                name,
                found: true,
                source: Some(s.id.clone()),
                dir,
            };
        }
    }
    ResolvedImage {
        name,
        found: false,
        source: None,
        dir: String::new(),
    }
}

#[cfg(test)]
mod tests;
