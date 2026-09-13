use super::*;
use crate::api::test_support::test_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

async fn post(app: &App, uri: &str, body: &str) -> Response {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    crate::api::router(app.clone()).oneshot(req).await.unwrap()
}

#[test]
fn content_hash_is_deterministic_and_distinguishes_neighbors() {
    assert_eq!(content_hash(""), content_hash(""));
    assert_eq!(content_hash("abc"), content_hash("abc"));
    assert_ne!(content_hash("abc"), content_hash("abd"));
    assert_ne!(content_hash("abc"), content_hash("abcd"));
    assert_ne!(content_hash("a"), content_hash("aa"));
    assert_ne!(content_hash("题"), content_hash("题 "));
    assert!(content_hash("").len() >= 32);
}

#[test]
fn percent_encode_cjk_by_utf8_bytes() {

    assert_eq!(percent_encode("题本.pdf"), "%E9%A2%98%E6%9C%AC.pdf");
    assert_eq!(percent_encode("a b"), "a%20b");
    assert_eq!(percent_encode(""), "");
    assert_eq!(percent_encode("A-Z_09.txt"), "A-Z_09.txt");
    assert!(percent_encode("题").is_ascii(), "output stays ASCII");
}

#[tokio::test]
async fn unsafe_file_names_are_rejected() {
    let app = test_app("exportfetch");

    for name in ["..", "a/b.pdf", ".hidden.pdf", "notes.txt", ""] {
        let resp = export_file(State(app.clone()), AxPath(name.to_string())).await;
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::BAD_REQUEST,
            "{name:?}"
        );
    }
}

#[tokio::test]
async fn non_object_payloads_are_rejected() {
    let app = test_app("export400");
    for body in ["[1,2]", "\"string\"", "42", "null"] {
        let resp = post(&app, "/api/export/pdf", body).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "export body {body}");
        let resp = post(&app, "/api/preview/pdf", body).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "preview body {body}"
        );
    }

    let resp = post(&app, "/api/export/pdf", r#"{ "doc": "diary" }"#).await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "doc must be workbook/answers"
    );
}

async fn json_of(resp: Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn same_display_name_different_content_does_not_overwrite() {
    let app = test_app("exportname");
    let spec = ExportSpec {
        name: "题本-题本-A4.pdf".into(),
        book: Some("b1".into()),
    };
    let hash1 = "aaaaaaaabbbbbbbbccccccccdddddddd0";
    let hash2 = "bbbbbbbbccccccccddddddddeeeeeeee0";

    let r1 = store_export(&app, hash1, &spec, b"first pdf".to_vec());
    assert_eq!(r1.status(), StatusCode::CREATED);
    let r2 = store_export(&app, hash2, &spec, b"second pdf".to_vec());
    assert_eq!(r2.status(), StatusCode::CREATED);

    let v1 = json_of(r1).await;
    let v2 = json_of(r2).await;
    assert_eq!(v1["name"], "题本-题本-A4.pdf");
    assert_ne!(
        v1["name"], v2["name"],
        "collision must pick a distinct name"
    );
    assert!(v2["name"].as_str().unwrap().starts_with("题本-题本-A4-"));
    assert!(v2["name"]
        .as_str()
        .unwrap()
        .ends_with(&format!("-{}.pdf", &hash2[..8])));

    each_hash_resolves_to_its_own_file(&app, hash1, hash2, &v1, &v2).await;
}

async fn each_hash_resolves_to_its_own_file(
    app: &App,
    hash1: &str,
    hash2: &str,
    v1: &serde_json::Value,
    v2: &serde_json::Value,
) {
    let exports = app.data_dir.join("exports");
    assert_eq!(
        std::fs::read(exports.join(v1["name"].as_str().unwrap())).unwrap(),
        b"first pdf"
    );
    assert_eq!(
        std::fs::read(exports.join(v2["name"].as_str().unwrap())).unwrap(),
        b"second pdf"
    );
    assert_eq!(
        app.export_cache.lookup(hash1).unwrap(),
        exports.join("题本-题本-A4.pdf")
    );
    assert_eq!(
        app.export_cache.lookup(hash2).unwrap(),
        exports.join(v2["name"].as_str().unwrap())
    );
}

#[tokio::test]
async fn identical_content_hits_the_cache_file() {
    let app = test_app("exportcache");
    let spec = ExportSpec {
        name: "题本-题本-B5.pdf".into(),
        book: None,
    };
    let hash = content_hash("{\"doc\":\"workbook\"}");
    let r1 = store_export(&app, &hash, &spec, b"same pdf".to_vec());
    assert_eq!(r1.status(), StatusCode::CREATED);

    let r2 = store_export(&app, &hash, &spec, b"same pdf".to_vec());
    assert_eq!(r2.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(r2.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["name"], "题本-题本-B5.pdf");
}
