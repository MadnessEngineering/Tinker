//! HTTP API server

use std::net::SocketAddr;
use axum::{
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::info;
use anyhow::Result;

pub async fn start_api_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Tinker API Server" }));

    info!("API server listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
