//! ccboard-web - Web frontend for ccboard using Leptos + Axum

#![recursion_limit = "1024"]

pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod router;
pub mod sse;

pub use app::App;
pub use router::create_router;

use anyhow::Result;
use ccboard_core::DataStore;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

/// Run the web server
pub async fn run(store: Arc<DataStore>, port: u16) -> Result<()> {
    let router = create_router(store);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;

    info!("Web server listening on http://{}", addr);
    println!("Web server listening on http://{}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
