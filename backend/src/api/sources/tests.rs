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

async fn browse_json(app: &App, path: &str) -> serde_json::Value {
    let resp = call(app, "GET", path, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    serde_json::from_str(&text(resp).await).unwrap()
}

async fn sources_json(app: &App) -> serde_json::Value {
    let resp = call(app, "GET", "/api/sources", None).await;
    serde_json::from_str(&text(resp).await).unwrap()
}

#[tokio::test]
async fn browse_classifies_a_folder_tree() {
    let app = test_app("browse");
    let bank = app.sources.get("s1").unwrap().path.clone();
    std::fs::create_dir_all(bank.join("子卷")).unwrap();
    std::fs::write(
        bank.join("子卷/N1.md"),
        "> @ N1\n\n子题______。\n\n## 答案\n\n1。\n",
    )
    .unwrap();
    std::fs::write(bank.join("子卷/笔记.md"), "# 笔记\n\n内容。\n").unwrap();
    std::fs::write(bank.join("note.md"), "只有一段文字，没有答案。\n").unwrap();

    root_listing_matches(&app).await;
    nested_listing_matches(&app).await;
    unsafe_paths_are_rejected(&app).await;
}

async fn root_listing_matches(app: &App) {
    let v = browse_json(app, "/api/sources/s1/browse").await;
    assert_eq!(v["dirs"], json!(["子卷"]));
    assert_eq!(v["total"], 1, "non-recursive counts root only");
    assert_eq!(v["questions"][0]["id"], "P1-1");
    let other: Vec<&str> = v["other_md"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["file"].as_str().unwrap())
        .collect();
    assert_eq!(other, vec!["note.md"]);
}

async fn nested_listing_matches(app: &App) {
    let v = browse_json(app, "/api/sources/s1/browse?path=%E5%AD%90%E5%8D%B7").await;
    assert_eq!(v["questions"][0]["id"], "N1");
    assert_eq!(v["other_md"][0]["reason"], "含一级标题");
}

async fn unsafe_paths_are_rejected(app: &App) {
    for bad in ["..", "a/../b", "/abs"] {
        let resp = call(
            app,
            "GET",
            &format!("/api/sources/s1/browse?path={}", urlencode(bad)),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "path {bad:?}");
    }
    let resp = call(app, "GET", "/api/sources/nope/browse", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| format!("%{b:02X}"))
        .collect::<String>()
        .replace("%2F", "/")
        .replace("%2E", ".")
}

#[tokio::test]
async fn recursive_toggle_changes_count_and_total() {
    let app = test_app("recursive");
    let bank = app.sources.get("s1").unwrap().path.clone();
    std::fs::create_dir_all(bank.join("子卷")).unwrap();
    std::fs::write(
        bank.join("子卷/N1.md"),
        "> @ N1\n\n子题______。\n\n## 答案\n\n1。\n",
    )
    .unwrap();

    let v = sources_json(&app).await;
    assert_eq!(v[0]["count"], 1);
    assert_eq!(v[0]["recursive"], false);

    let resp = call(
        &app,
        "PUT",
        "/api/sources/s1",
        Some(json!({ "recursive": true })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);

    let v = sources_json(&app).await;
    assert_eq!(v[0]["count"], 2, "subfolder question now loaded");
    assert_eq!(v[0]["recursive"], true);

    let resp = call(&app, "GET", "/api/questions", None).await;
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 2);

    let resp = call(
        &app,
        "PUT",
        "/api/sources/s1",
        Some(json!({ "path": bank.to_string_lossy() })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    assert_eq!(v["recursive"], true, "path-only update keeps the flag");

    let resp = call(&app, "PUT", "/api/sources/s1", Some(json!({}))).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn resolve_images_reports_locations() {
    let app = test_app("resolve");
    let bank = app.sources.get("s1").unwrap().path.clone();
    std::fs::write(bank.join("图.svg"), "<svg/>").unwrap();

    let resp = call(
        &app,
        "POST",
        "/api/sources/s1/resolve-images",
        Some(json!({ "names": ["图.svg", "缺失.png"] })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr[0]["found"], true);
    assert_eq!(arr[0]["source"], "s1");
    assert_eq!(arr[0]["dir"], "");
    assert_eq!(arr[1]["found"], false);
}
