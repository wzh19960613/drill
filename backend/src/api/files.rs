use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::fs;

use super::{bad_request, ApiError};
use crate::app::App;
use crate::fsutil::{atomic_write, is_safe_rel_path};
use crate::parsing::{self, Classified};

fn resolve_md(app: &App, id: &str, rel: &str) -> Result<std::path::PathBuf, ApiError> {
    let src = app
        .sources
        .get(id)
        .ok_or_else(|| ApiError::not_found("unknown source id"))?;
    if !is_safe_rel_path(rel) || !rel.ends_with(".md") {
        return Err(ApiError::bad_request("invalid file path (md files only)"));
    }
    let path = src.path.join(rel);
    if !path.is_file() {
        return Err(ApiError::not_found("file not found"));
    }
    Ok(path)
}

fn classify_info(app: &App, id: &str, rel: &str) -> Response {
    let Some(src) = app.sources.get(id) else {
        return (StatusCode::NOT_FOUND, "unknown source id").into_response();
    };
    let marked = src.excluded.iter().any(|p| p == rel);
    let path = src.path.join(rel);
    let (question, reason) = match parsing::classify_file_cached(&path) {
        Classified::Question(_) => (true, None),
        Classified::Rejected(reason) => (false, Some(reason)),
    };
    let question = question && !marked;
    Json(json!({
        "path": rel,
        "question": question,
        "marked": marked,

        "reason": if marked && question_would_pass(&path) { Some("已标记为非题目".to_string()) } else { reason },
    }))
    .into_response()
}

fn question_would_pass(path: &std::path::Path) -> bool {
    matches!(
        parsing::classify_file_cached(path),
        Classified::Question(_)
    )
}

#[derive(Deserialize)]
pub struct FilePath {
    pub path: String,
}

pub async fn get(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(q): Query<FilePath>,
) -> Response {
    let path = match resolve_md(&app, &id, &q.path) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match fs::read_to_string(&path) {
        Ok(md) => Json(json!({ "path": q.path, "markdown": md })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("read failed: {e}")).into_response(),
    }
}

#[derive(Deserialize)]
pub struct FileSave {
    pub path: String,
    pub markdown: String,
}

pub async fn put(State(app): State<App>, AxPath(id): AxPath<String>, Json(body): Json<FileSave>) -> Response {
    let src = match app.sources.get(&id) {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "unknown source id").into_response(),
    };
    if !is_safe_rel_path(&body.path) || !body.path.ends_with(".md") {
        return bad_request("invalid file path (md files only)");
    }
    let path = src.path.join(&body.path);
    if !path.is_file() {
        return (StatusCode::NOT_FOUND, "file not found").into_response();
    }
    if let Err(e) = atomic_write(&path, body.markdown.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("save failed: {e}"),
        )
            .into_response();
    }

    if let Classified::Question(_) = parsing::classify_file_cached(&path) {
        if src.excluded.iter().any(|p| p == &body.path) {
            let _ = app.sources.mark_excluded(&id, &body.path, false);
        }
    }
    classify_info(&app, &id, &body.path)
}

pub async fn delete(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(q): Query<FilePath>,
) -> Response {
    let path = match resolve_md(&app, &id, &q.path) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    if app.sources.get(&id).is_some_and(|s| s.excluded.iter().any(|p| p == &q.path)) {
        let _ = app.sources.mark_excluded(&id, &q.path, false);
    }
    match fs::remove_file(&path) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("delete failed: {e}"),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct MarkFile {
    pub path: String,
    pub non_question: bool,
}

#[derive(Deserialize)]
pub struct MarkFolder {
    pub path: String,
    pub non_question: bool,
    /// dry run: report the affected questions/books without marking
    #[serde(default)]
    pub check: bool,
}

/// Mark/unmark a whole folder as non-questions: every file in its subtree
/// stops loading (and browsing reports them as manually marked).
pub async fn mark_folder(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<MarkFolder>,
) -> Response {
    let Some(src) = app.sources.get(&id) else {
        return (StatusCode::NOT_FOUND, "unknown source id").into_response();
    };
    if !is_safe_rel_path(&body.path) || body.path.is_empty() {
        return bad_request("invalid folder path");
    }
    let dir = src.path.join(&body.path);
    if !dir.is_dir() {
        return (StatusCode::NOT_FOUND, "folder not found").into_response();
    }
    if body.check && body.non_question {
        return folder_mark_check(&app, &id, &src.path, &body.path);
    }
    if let Err(e) = app.sources.mark_excluded(&id, &body.path, body.non_question) {
        return match e.as_str() {
            "unknown source id" => (StatusCode::NOT_FOUND, e).into_response(),
            _ => bad_request(&e),
        };
    }
    Json(json!({ "path": body.path, "marked": body.non_question })).into_response()
}

/// Impact preview for marking a folder: questions under its subtree and the
/// books that reference them (mirrors the recursive-toggle confirmation).
fn folder_mark_check(app: &App, id: &str, src_path: &std::path::Path, rel: &str) -> Response {
    let src = app.sources.get(id).expect("source checked by caller");
    let excluded: std::collections::HashSet<String> = src.excluded.iter().cloned().collect();
    let ids: std::collections::HashSet<String> =
        parsing::load_questions_from(&src_path.join(rel), "", true, &excluded)
            .into_iter()
            .map(|q| q.id)
            .collect();
    let impact: Vec<serde_json::Value> = app
        .books
        .list()
        .into_iter()
        .filter_map(|b| {
            let n = b
                .items
                .iter()
                .filter(|it| it.source == id && ids.contains(&it.id))
                .count();
            (n > 0).then(|| json!({ "name": b.name, "count": n }))
        })
        .collect();
    Json(json!({ "path": rel, "count": ids.len(), "impact": impact })).into_response()
}

pub async fn mark(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<MarkFile>,
) -> Response {
    if app.sources.get(&id).is_none() {
        return (StatusCode::NOT_FOUND, "unknown source id").into_response();
    }
    if !is_safe_rel_path(&body.path) || !body.path.ends_with(".md") {
        return bad_request("invalid file path (md files only)");
    }
    if !app.sources.get(&id).map(|s| s.path.join(&body.path).is_file()).unwrap_or(false) {
        return (StatusCode::NOT_FOUND, "file not found").into_response();
    }
    if let Err(e) = app.sources.mark_excluded(&id, &body.path, body.non_question) {
        return match e.as_str() {
            "unknown source id" => (StatusCode::NOT_FOUND, e).into_response(),
            _ => bad_request(&e),
        };
    }
    classify_info(&app, &id, &body.path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::test_app;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    async fn call(app: &App, method: &str, uri: &str, body: Option<serde_json::Value>) -> Response {
        let req = Request::builder().method(method).uri(uri);
        let req = match body {
            Some(v) => req
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
                .unwrap(),
            None => req.body(Body::empty()).unwrap(),
        };
        crate::api::router(app.clone()).oneshot(req).await.unwrap()
    }

    async fn text(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn url(path: &str) -> String {
        format!("/api/sources/s1/file?path={}", urlencode(path))
    }

    fn urlencode(s: &str) -> String {
        s.bytes().map(|b| format!("%{b:02X}")).collect()
    }

    #[tokio::test]
    async fn mark_hides_question_but_keeps_file() {
        let app = test_app("mark");
        let bank = app.sources.get("s1").unwrap().path.clone();

        assert!(question_ids(&app).await.iter().any(|q| q == "P1-1"));
        mark_hides_the_question(&app).await;
        assert!(bank.join("P1-1.md").is_file(), "source file untouched");
        unmark_restores_the_question(&app).await;
    }

    async fn question_ids(app: &App) -> Vec<String> {
        let v: Vec<serde_json::Value> = serde_json::from_str(
            &text(call(app, "GET", "/api/questions", None).await).await,
        )
        .unwrap();
        v.into_iter()
            .map(|q| q["id"].as_str().unwrap().to_string())
            .collect()
    }

    async fn mark_json(app: &App, non_question: bool) -> serde_json::Value {
        let resp = call(
            app,
            "PUT",
            "/api/sources/s1/mark",
            Some(json!({ "path": "P1-1.md", "non_question": non_question })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);
        serde_json::from_str(&text(resp).await).unwrap()
    }

    async fn mark_hides_the_question(app: &App) {
        let v = mark_json(app, true).await;
        assert_eq!(v["question"], false);
        assert_eq!(v["marked"], true);
        assert!(
            !question_ids(app).await.iter().any(|q| q == "P1-1"),
            "marked file not loaded"
        );

        let resp = call(app, "GET", "/api/sources/s1/browse", None).await;
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        let other = v["other_md"].as_array().unwrap();
        assert_eq!(other[0]["file"], "P1-1.md");
        assert_eq!(other[0]["reason"], "已标记为非题目");
    }

    async fn unmark_restores_the_question(app: &App) {
        let v = mark_json(app, false).await;
        assert_eq!(v["question"], true);
        assert!(question_ids(app).await.iter().any(|q| q == "P1-1"));
    }

    #[tokio::test]
    async fn edit_file_reidentifies_and_unmarks() {
        let app = test_app("file-edit");
        let bank = app.sources.get("s1").unwrap().path.clone();

        std::fs::write(bank.join("note.md"), "# 笔记\n\n内容。\n").unwrap();

        let resp = call(
            &app,
            "PUT",
            "/api/sources/s1/mark",
            Some(json!({ "path": "note.md", "non_question": true })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = call(
            &app,
            "PUT",
            "/api/sources/s1/file",
            Some(json!({ "path": "note.md", "markdown": "> @ N1\n\n题干______。\n\n## 答案\n\n42。\n" })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v["question"], true, "{}", v);
        assert_eq!(v["marked"], false, "qualifying edit auto-unmarks");

        let resp = call(
            &app,
            "PUT",
            "/api/sources/s1/file",
            Some(json!({ "path": "note.md", "markdown": "# 又变回笔记\n" })),
        )
        .await;
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v["question"], false);
        assert_eq!(v["reason"], "含一级标题");
    }

    #[tokio::test]
    async fn mark_folder_hides_the_whole_subtree() {
        let app = test_app("markdir");
        let bank = app.sources.get("s1").unwrap().path.clone();
        std::fs::create_dir_all(bank.join("卷/深处")).unwrap();
        std::fs::write(bank.join("卷/P2-1.md"), "> @ P2-1\n\n题______。\n\n## 答案\n\n2。\n").unwrap();
        std::fs::write(bank.join("卷/深处/P3-1.md"), "> @ P3-1\n\n题______。\n\n## 答案\n\n3。\n").unwrap();

        async fn ids(app: &App) -> Vec<String> {
            let v: Vec<serde_json::Value> =
                serde_json::from_str(&text(call(app, "GET", "/api/questions", None).await).await).unwrap();
            v.into_iter().map(|q| q["id"].as_str().unwrap().to_string()).collect()
        }
        // recursive on so the subtree loads
        let resp = call(&app, "PUT", "/api/sources/s1", Some(json!({ "recursive": true }))).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(ids(&app).await.iter().any(|q| q == "P3-1"));

        // mark the folder: every file under it disappears, the file stays on disk
        let resp = call(
            &app,
            "PUT",
            "/api/sources/s1/mark-folder",
            Some(json!({ "path": "卷", "non_question": true })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);
        assert!(!ids(&app).await.iter().any(|q| q == "P2-1" || q == "P3-1"));
        assert!(bank.join("卷/深处/P3-1.md").is_file());

        // browsing reports the files as manually marked
        let resp = call(&app, "GET", "/api/sources/s1/browse?path=%E5%8D%B7", None).await;
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert!(v["other_md"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["reason"] == "已标记为非题目"));

        // unmark restores
        let resp = call(
            &app,
            "PUT",
            "/api/sources/s1/mark-folder",
            Some(json!({ "path": "卷", "non_question": false })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(ids(&app).await.iter().any(|q| q == "P3-1"));

        // non-folder paths are rejected
        for bad in ["", "..", "不存在"] {
            let resp = call(
                &app,
                "PUT",
                "/api/sources/s1/mark-folder",
                Some(json!({ "path": bad, "non_question": true })),
            )
            .await;
            assert!(
                resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::NOT_FOUND,
                "path {bad:?}"
            );
        }
    }

    #[tokio::test]
    async fn get_and_delete_file() {
        let app = test_app("file-del");
        let bank = app.sources.get("s1").unwrap().path.clone();
        std::fs::write(bank.join("note.md"), "# x\n").unwrap();

        let resp = call(&app, "GET", &url("note.md"), None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v["markdown"], "# x\n");

        assert_eq!(
            call(&app, "GET", &url("图.svg"), None).await.status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(&app, "GET", &url("../x.md"), None).await.status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(&app, "GET", &url("missing.md"), None).await.status(),
            StatusCode::NOT_FOUND
        );

        let resp = call(&app, "DELETE", &url("note.md"), None).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(!bank.join("note.md").exists());
        assert!(
            !bank.join("note.md.bak").exists(),
            "deletion leaves no backup clutter"
        );
    }
}
