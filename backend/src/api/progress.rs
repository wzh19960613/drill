use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::app::App;

pub async fn get_paused(State(app): State<App>) -> Json<Value> {
    Json(app.paused.get().unwrap_or(Value::Null))
}

pub async fn put_paused(State(app): State<App>, Json(body): Json<Value>) -> Response {
    app.paused.set(body);
    StatusCode::NO_CONTENT.into_response()
}

pub async fn delete_paused(State(app): State<App>) -> Response {
    app.paused.clear();
    StatusCode::NO_CONTENT.into_response()
}

pub async fn get_mastery(State(app): State<App>) -> Json<Value> {
    serde_json::json!({ "ids": app.mastery.list() }).into()
}

#[derive(Deserialize)]
pub struct SetMastery {
    ids: Vec<String>,
}

pub async fn put_mastery(State(app): State<App>, Json(body): Json<SetMastery>) -> Response {
    app.mastery.set(body.ids);
    StatusCode::NO_CONTENT.into_response()
}
