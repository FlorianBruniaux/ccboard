//! ccboard-web - Web frontend for ccboard using Leptos + Axum

#![recursion_limit = "2048"]

pub mod api;
pub mod app;
pub mod components;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod router;
#[cfg(feature = "ssr")]
pub mod sse;

pub mod sse_hook;
pub mod utils;

pub use app::App;

#[cfg(feature = "ssr")]
pub use router::create_router;

// Server-side only code (backend with tokio/axum)
#[cfg(feature = "ssr")]
pub async fn run(store: std::sync::Arc<ccboard_core::DataStore>, port: u16) -> anyhow::Result<()> {
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tracing::info;

    let router = create_router(store);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;

    info!("Web server listening on http://{}", addr);
    println!("Web server listening on http://{}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
