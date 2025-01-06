//! HTTP API server

use axum::{
    Router,
    routing::get,
    Json,
    extract::State,
};

pub async fn start_api_server() {
    let app: Router = Router::new()
        .route("/health", get(health_check));

    // TODO: Add more routes and implement server
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
} 
