use std::collections::HashMap;
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
use crate::fsutil::{atomic_write, is_safe_component};
use crate::model::Question;
use crate::parsing;

pub async fn list(State(app): State<App>) -> Json<Vec<Question>> {
    let mut questions = Vec::new();
    for src in app.sources.list() {
        questions.extend(parsing::load_questions_from(&src.path, &src.id));
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

fn question_path(app: &App, id: &str, source: &str) -> Result<PathBuf, ApiError> {
    if !is_safe_component(id) {
        return Err(ApiError::bad_request("invalid question id"));
    }
    Ok(source_dir(app, source)?.join(format!("{id}.md")))
}

pub async fn raw(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(source) = params.get("source") else {
        return bad_request("source is required");
    };
    let path = match question_path(&app, &id, source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    match fs::read_to_string(&path) {
        Ok(md) => Json(json!({ "id": id, "source": source, "markdown": md })).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "question file not found").into_response(),
    }
}

#[derive(Deserialize)]
pub struct SaveQuestion {
    pub markdown: String,
    pub source: String,
}

fn parse_body(app: &App, body: &SaveQuestion) -> Result<Question, ApiError> {
    let mut q = parsing::parse_md(&body.markdown)
        .map_err(|e| ApiError::bad_request(format!("parse failed: {e:#}")))?;
    source_dir(app, &body.source)?;
    q.source = body.source.clone();
    Ok(q)
}

pub async fn preview(State(app): State<App>, Json(body): Json<SaveQuestion>) -> Response {
    match parse_body(&app, &body) {
        Ok(q) => Json(q).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn create(State(app): State<App>, Json(body): Json<SaveQuestion>) -> Response {
    let q = match parse_body(&app, &body) {
        Ok(q) => q,
        Err(e) => return e.into_response(),
    };
    let path = match question_path(&app, &q.id, &body.source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    if path.exists() {
        return (
            StatusCode::CONFLICT,
            format!("question {} already exists", q.id),
        )
            .into_response();
    }
    if let Err(e) = atomic_write(&path, body.markdown.as_bytes()) {
        return server_error(&format!("save failed: {e}"));
    }
    (StatusCode::CREATED, Json(q)).into_response()
}

pub async fn update(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Json(body): Json<SaveQuestion>,
) -> Response {
    let q = match parse_body(&app, &body) {
        Ok(q) => q,
        Err(e) => return e.into_response(),
    };
    if q.id != id {
        return bad_request(&format!(
            "header locate ({}) does not match target ({id})",
            q.id
        ));
    }
    let path = match question_path(&app, &id, &body.source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "question file not found").into_response();
    }
    if let Err(e) = atomic_write(&path, body.markdown.as_bytes()) {
        return server_error(&format!("save failed: {e}"));
    }
    (StatusCode::OK, Json(q)).into_response()
}

pub async fn remove(
    State(app): State<App>,
    AxPath(id): AxPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(source) = params.get("source") else {
        return bad_request("source is required");
    };
    let path = match question_path(&app, &id, source) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "question file not found").into_response();
    }
    let backup = path.with_extension("md.bak");
    let _ = fs::write(&backup, fs::read_to_string(&path).unwrap_or_default());
    match fs::remove_file(&path) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => server_error(&format!("delete failed: {e}")),
    }
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
        let mut req = Request::builder().method(method).uri(uri);
        let req = match body {
            Some(v) => req
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
                .unwrap(),
            None => {
                if method == "POST" || method == "PUT" {
                    req = req.header("content-type", "application/json");
                    req.body(Body::from("{}")).unwrap()
                } else {
                    req.body(Body::empty()).unwrap()
                }
            }
        };
        crate::api::router(app.clone()).oneshot(req).await.unwrap()
    }

    async fn text(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn save_body(locate: &str, source: &str) -> serde_json::Value {
        json!({
            "source": source,
            "markdown": format!(
                "> @ {locate}\n\n题干______。\n\n## 答案\n\n42。\n"
            )
        })
    }

    #[tokio::test]
    async fn create_reads_updates_and_deletes_a_question() {
        let app = test_app("crud");

        // unknown source is a 404
        let resp = call(
            &app,
            "POST",
            "/api/questions",
            Some(save_body("P9-9", "nope")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // path traversal ids are rejected
        for bad in ["..", "a/b"] {
            let resp = call(&app, "POST", "/api/questions", Some(save_body(bad, "s1"))).await;
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "id {bad:?}");
            // a multi-segment id cannot reach the handler through the router
            // ({id} never matches '/'), so call it directly
            let mut params = HashMap::new();
            params.insert("source".to_string(), "s1".to_string());
            let resp = raw(State(app.clone()), AxPath(bad.to_string()), Query(params)).await;
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "raw id {bad:?}");
        }

        // create
        let resp = call(
            &app,
            "POST",
            "/api/questions",
            Some(save_body("P9-9", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED, "{}", text(resp).await);

        // duplicate create conflicts
        let resp = call(
            &app,
            "POST",
            "/api/questions",
            Some(save_body("P9-9", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        // raw markdown round-trips
        let resp = call(&app, "GET", "/api/questions/P9-9/raw?source=s1", None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(text(resp).await.contains("P9-9"));

        // update with mismatched locate fails
        let resp = call(
            &app,
            "PUT",
            "/api/questions/P9-9",
            Some(save_body("P8-8", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // update with matching locate succeeds
        let resp = call(
            &app,
            "PUT",
            "/api/questions/P9-9",
            Some(save_body("P9-9", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // update of a missing question is a 404
        let resp = call(
            &app,
            "PUT",
            "/api/questions/P7-7",
            Some(save_body("P7-7", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // remove, then second remove is a 404
        let resp = call(&app, "DELETE", "/api/questions/P9-9?source=s1", None).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = call(&app, "DELETE", "/api/questions/P9-9?source=s1", None).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_returns_questions_without_stats() {
        let app = test_app("list");
        let resp = call(&app, "GET", "/api/questions", None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "P1-1");
        assert!(
            arr[0].get("stats").is_none(),
            "list items no longer carry stats"
        );
    }

    #[tokio::test]
    async fn preview_parses_without_writing() {
        let app = test_app("preview");
        let resp = call(
            &app,
            "POST",
            "/api/questions/preview",
            Some(save_body("P1-1", "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v["id"], "P1-1");
        assert_eq!(v["source"], "s1");
        // malformed markdown is a 400
        let resp = call(
            &app,
            "POST",
            "/api/questions/preview",
            Some(json!({ "source": "s1", "markdown": "no header at all" })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
