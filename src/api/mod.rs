//! HTTP API server

use axum::{
    routing::get,
    Router,
    Json,
};

pub async fn start_api_server() {
    let _router: Router = Router::new()
        .route("/health", get(health_check));
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
} 
