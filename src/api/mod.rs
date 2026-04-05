use axum::{Router, routing::get, response::IntoResponse, Json};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use serde_json::json;

use crate::data::Data;

pub mod music;
pub mod status;

pub async fn start(data: Data) {
    let state = Arc::new(data);
    let app: Router<()> = Router::new()
        .route("/", get(root))
        .nest("/music", music::routes())
        .nest("/status", status::routes())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("[API] Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    Json(json!({
        "name": "Hearth API",
        "version": "1.0.0",
        "status": "online"
    }))
}
