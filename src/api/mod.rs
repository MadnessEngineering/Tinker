//! HTTP API server

use std::net::SocketAddr;
use anyhow::Result;

#[cfg(feature = "api")]
use axum::{
    routing::get,
    Router,
};

#[cfg(feature = "api")]
use tokio::net::TcpListener;

#[cfg(feature = "tracing")]
use tracing::info;

pub async fn start_api_server(addr: SocketAddr) -> Result<()> {
    #[cfg(feature = "api")]
    {
        let app = Router::new()
            .route("/", get(|| async { "Tinker API Server" }));

        #[cfg(feature = "tracing")]
        info!("API server listening on {}", addr);
        
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    #[cfg(not(feature = "api"))]
    {
        #[cfg(feature = "tracing")]
        info!("API server disabled - enable with --features api");
        Ok(())
    }
}

#[cfg(feature = "metrics")]
pub mod metrics {
    use prometheus::{Registry, Counter, register_counter};
    use anyhow::Result;

    lazy_static::lazy_static! {
        pub static ref REGISTRY: Registry = Registry::new();
        pub static ref REQUEST_COUNT: Counter = register_counter!(
            "tinker_request_total",
            "Total number of requests received"
        ).unwrap();
    }

    pub fn init() -> Result<()> {
        REGISTRY.register(Box::new(REQUEST_COUNT.clone()))?;
        Ok(())
    }
}
