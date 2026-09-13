use axum::extract::{Path as AxPath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use super::bad_request;
use crate::app::App;
use crate::model::Record;

pub async fn list(State(app): State<App>) -> Json<Vec<Record>> {
    Json(app.records.list())
}

#[derive(Deserialize)]
pub struct NewRecord {
    #[serde(rename = "questionId")]
    question_id: String,

    source: String,
    correct: bool,
    #[serde(default)]
    ms: Option<u64>,
}

pub async fn add(State(app): State<App>, Json(body): Json<NewRecord>) -> Response {
    if body.question_id.trim().is_empty() {
        return bad_request("questionId is required");
    }
    if body.source.trim().is_empty() {
        return bad_request("source is required");
    }
    let record = app
        .records
        .add(body.source, body.question_id, body.correct, body.ms);
    (StatusCode::CREATED, Json(json!({ "record": record }))).into_response()
}

#[derive(Deserialize)]
pub struct UpdateRecord {
    correct: bool,
    #[serde(default)]
    ms: Option<u64>,
}

pub async fn update(
    State(app): State<App>,
    AxPath(id): AxPath<u64>,
    Json(body): Json<UpdateRecord>,
) -> Response {
    resolve(app.records.update(id, body.correct, body.ms))
}

pub async fn remove(State(app): State<App>, AxPath(id): AxPath<u64>) -> Response {
    resolve(app.records.delete(id))
}

fn resolve(ok: bool) -> Response {
    if ok {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::NOT_FOUND, "record not found").into_response()
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

    async fn post(app: &App, body: serde_json::Value) -> axum::response::Response {
        let req = Request::builder()
            .method("POST")
            .uri("/api/records")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        crate::api::router(app.clone()).oneshot(req).await.unwrap()
    }

    async fn text(resp: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn add_requires_source_and_returns_only_the_record() {
        let app = test_app("records");

        let resp = post(
            &app,
            json!({ "questionId": "P1-1", "correct": true, "ms": 900 }),
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing source is rejected by serde"
        );

        let resp = post(
            &app,
            json!({ "questionId": "P1-1", "source": "", "correct": true }),
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "empty source is rejected"
        );

        created_record_carries_no_stats(&app).await;
        list_round_trips_the_source(&app).await;
    }

    async fn created_record_carries_no_stats(app: &App) {
        let resp = post(
            app,
            json!({ "questionId": "P1-1", "source": "s1", "correct": true, "ms": 900 }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v["record"]["questionId"], "P1-1");
        assert_eq!(v["record"]["source"], "s1");
        assert_eq!(v["record"]["ms"], 900);
        assert!(
            v.get("stats").is_none(),
            "stats field removed from the wire"
        );
    }

    async fn list_round_trips_the_source(app: &App) {
        let req = Request::builder()
            .uri("/api/records")
            .body(Body::empty())
            .unwrap();
        let resp = crate::api::router(app.clone()).oneshot(req).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        assert_eq!(v[0]["source"], "s1");
    }
}
