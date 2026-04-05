use axum::{Router, routing::get, Json, response::IntoResponse};
use std::sync::Arc;
use serde_json::json;
use crate::data::Data;

pub fn routes() -> Router<Arc<Data>> {
    Router::new()
        .route("/health", get(get_health))
}

async fn get_health() -> impl IntoResponse {
    Json::<serde_json::Value>(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
