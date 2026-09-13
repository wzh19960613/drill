use std::collections::HashMap;

use axum::extract::{Path as AxPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use super::bad_request;
use crate::app::App;
use crate::model::BookDef;

pub async fn list(State(app): State<App>) -> Json<Vec<BookDef>> {
    Json(app.books.list())
}

pub async fn save(State(app): State<App>, Json(mut def): Json<BookDef>) -> Response {
    if def.id.trim().is_empty() || def.name.trim().is_empty() {
        return bad_request("book needs an id and a name");
    }
    def.name = def.name.trim().chars().take(40).collect();
    match app.books.upsert(def.clone()) {
        Ok(()) => (StatusCode::CREATED, Json(def)).into_response(),
        Err(e) => bad_request(&e),
    }
}

pub async fn remove(State(app): State<App>, AxPath(id): AxPath<String>) -> Response {
    if app.books.delete(&id) {

        app.export_cache.remove_book(&id);
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::NOT_FOUND, "book not found").into_response()
    }
}

pub async fn get_active(
    State(app): State<App>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let subject = params.get("subject").cloned().unwrap_or_default();
    Json(json!({ "id": app.books.get_active(&subject) }))
}

#[derive(Deserialize)]
pub struct ActiveBook {
    #[serde(default)]
    subject: String,
    id: Option<String>,
}

pub async fn set_active(State(app): State<App>, Json(body): Json<ActiveBook>) -> Response {
    if let Some(resp) = unknown_book_response(&app, &body.id) {
        return resp;
    }
    app.books.set_active(&body.subject, body.id);
    StatusCode::NO_CONTENT.into_response()
}

pub async fn get_favorite(State(app): State<App>) -> Json<serde_json::Value> {
    Json(json!({ "id": app.books.favorite() }))
}

#[derive(Deserialize)]
pub struct FavoriteBook {
    id: Option<String>,
}

pub async fn set_favorite(State(app): State<App>, Json(body): Json<FavoriteBook>) -> Response {
    if let Some(resp) = unknown_book_response(&app, &body.id) {
        return resp;
    }
    app.books.set_favorite(body.id);
    StatusCode::NO_CONTENT.into_response()
}

fn unknown_book_response(app: &App, id: &Option<String>) -> Option<Response> {
    let unknown = id
        .as_ref()
        .is_some_and(|id| !app.books.list().iter().any(|b| &b.id == id));
    unknown.then(|| bad_request("unknown book id"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::{test_app, test_app_capped};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    fn book(id: &str) -> serde_json::Value {
        json!({
            "id": id, "name": format!("n{id}"), "seed": "", "date": "2026-09-05",
            "createdAt": 1,
            "items": [{ "id": "P1-1", "source": "s1", "optionOrder": null }]
        })
    }

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

    #[tokio::test]
    async fn save_roundtrips_item_source_field() {
        let app = test_app("books");
        let resp = call(&app, "POST", "/api/books", Some(book("b1"))).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp = call(&app, "GET", "/api/books", None).await;
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v[0]["items"][0]["source"], "s1", "item source is persisted");
    }

    #[tokio::test]
    async fn posting_beyond_the_book_cap_is_a_400() {

        let app = test_app_capped("bookcap", 3);
        for i in 0..3 {
            app.books
                .upsert(serde_json::from_value::<BookDef>(book(&format!("full{i}"))).unwrap())
                .unwrap();
        }
        let resp = call(&app, "POST", "/api/books", Some(book("one-too-many"))).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let msg = String::from_utf8_lossy(&bytes);
        assert!(msg.contains("题本数量已达上限"), "{msg}");
    }

    #[tokio::test]
    async fn set_active_and_favorite_reject_unknown_ids() {
        let app = test_app("bookrefs");
        call(&app, "POST", "/api/books", Some(book("b1"))).await;

        let resp = call(
            &app,
            "PUT",
            "/api/books/active",
            Some(json!({ "subject": "", "id": "nope" })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp = call(
            &app,
            "PUT",
            "/api/books/favorite",
            Some(json!({ "id": "nope" })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let resp = call(
            &app,
            "PUT",
            "/api/books/active",
            Some(json!({ "subject": "", "id": "b1" })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = call(
            &app,
            "PUT",
            "/api/books/favorite",
            Some(json!({ "id": null })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}
