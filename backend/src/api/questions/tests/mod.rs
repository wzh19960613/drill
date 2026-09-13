use super::*;
use crate::api::test_support::test_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

pub(super) async fn call(app: &App, method: &str, uri: &str, body: Option<serde_json::Value>) -> Response {
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

async fn create_conflicts_and_roundtrips(app: &App) {
    let resp = call(app, "POST", "/api/questions/save", Some(save_body("P9-9", "s1"))).await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{}", text(resp).await);

    let resp = call(app, "POST", "/api/questions/save", Some(save_body("P9-9", "s1"))).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT, "{}", text(resp).await);

    let resp = call(app, "GET", "/api/questions/P9-9/raw?source=s1", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    assert_eq!(v["dir"], "");
    assert_eq!(v["file"], "P9-9.md");
    assert!(v["markdown"].as_str().unwrap().contains("P9-9"));
}

pub(super) async fn stage_upload(app: &App, name: &str) -> String {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/uploads?name={name}"))
        .header("content-type", "image/png")
        .body(Body::from(vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']))
        .unwrap();
    let resp = crate::api::router(app.clone())
        .oneshot(req)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let up: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    up["id"].as_str().unwrap().to_string()
}

pub(super) async fn text(resp: Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

pub(super) fn save_body(filename: &str, source: &str) -> serde_json::Value {
    json!({
        "source": source,
        "dir": "",
        "filename": filename,
        "markdown": format!(
            "> @ {filename}\n\n题干______。\n\n## 答案\n\n42。\n"
        )
    })
}

#[tokio::test]
async fn save_raw_and_delete_a_question() {
    let app = test_app("save");

    let resp = call(
        &app,
        "POST",
        "/api/questions/save",
        Some(save_body("P9-9", "nope")),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    for bad in ["..", "a/b", ".hidden", ""] {
        let resp = call(
            &app,
            "POST",
            "/api/questions/save",
            Some(save_body(bad, "s1")),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "name {bad:?}");
    }

    create_conflicts_and_roundtrips(&app).await;

    let resp = call(&app, "DELETE", "/api/questions/P9-9?source=s1", None).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let resp = call(&app, "DELETE", "/api/questions/P9-9?source=s1", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn save_rejects_invalid_question_content() {
    let app = test_app("save-invalid");

    let body = json!({
        "source": "s1", "dir": "", "filename": "X1",
        "markdown": "> @ X1\n\n## 答案\n\n42。\n"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", text(resp).await);

    let body = json!({
        "source": "s1", "dir": "", "filename": "X1",
        "markdown": "> @ X1\n\n# 大标题\n\n题______。\n\n## 答案\n\n42。\n"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = json!({
        "source": "s1", "dir": "", "filename": "X1",
        "markdown": "> @ X1\n\n题干______。\n"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = json!({
        "source": "s1", "dir": "", "filename": "X1",
        "markdown": "> @ 其他\n\n题干______。\n\n## 答案\n\n42。\n"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", text(resp).await);
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
        Some(json!({ "source": "s1", "markdown": "> @ P1-1\n\n题干______。\n\n## 答案\n\n42。\n" })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    assert_eq!(v["id"], "P1-1");
    assert_eq!(v["source"], "s1");
    let resp = call(
        &app,
        "POST",
        "/api/questions/preview",
        Some(json!({ "source": "s1", "markdown": "no header at all" })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[cfg(test)]
mod save_flow;
