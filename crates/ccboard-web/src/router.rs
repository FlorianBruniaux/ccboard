//! Web router using Axum

use axum::{Router, response::Html, routing::get};
use ccboard_core::DataStore;
use std::sync::Arc;
use tower_http::{
  cors::{Any, CorsLayer},
  services::ServeDir,
};

use crate::sse;

/// Create the web router
pub fn create_router(store: Arc<DataStore>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(index_handler))
        .route("/api/stats", get(stats_handler))
        .route("/api/sessions", get(sessions_handler))
        .route("/api/health", get(health_handler))
        .route("/api/events", get(sse_handler))
        .nest_service("/static", ServeDir::new("crates/ccboard-web/static"))
        .layer(cors)
        .with_state(store)
}

async fn index_handler() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>ccboard - Claude Code Dashboard</title>

  <!-- CSS Reset & Dark Theme -->
  <link rel="stylesheet" href="/static/reset.css">
  <link rel="stylesheet" href="/static/style.css">

  <!-- Leptos WASM -->
  <link data-trunk rel="rust" data-wasm-opt="z" />

  <style>
    .setup-card {
      max-width: 800px;
      margin: 2rem auto;
    }
    .setup-content {
      padding: var(--space-lg);
    }
    .setup-steps {
      margin-left: var(--space-lg);
      margin-top: var(--space-sm);
    }
    .setup-steps li {
      margin-bottom: var(--space-sm);
      color: var(--text-secondary);
    }
    .api-list {
      margin-left: var(--space-lg);
      margin-top: var(--space-sm);
    }
    .api-list li {
      margin-bottom: var(--space-xs);
    }
  </style>
</head>
<body>
  <div class="app">
    <div class="header">
      <div class="header-logo">ccboard</div>
    </div>
    <div class="content">
      <div class="card setup-card">
        <div class="card-header">
          <h1 class="card-title">🚧 ccboard Web UI - Build Required</h1>
        </div>
        <div class="card-body setup-content">
          <p>The Leptos WASM frontend needs to be compiled before the web UI can be displayed.</p>

          <h3 style="margin-top: var(--space-lg); margin-bottom: var(--space-sm); color: var(--text-primary);">Setup Instructions:</h3>
          <ol class="setup-steps">
            <li>Install Trunk: <code>cargo install trunk</code></li>
            <li>Add WASM target: <code>rustup target add wasm32-unknown-unknown</code></li>
            <li>Build frontend: <code>cd crates/ccboard-web && trunk build --release</code></li>
            <li>Restart server: <code>cargo run -- web --port 3333</code></li>
          </ol>

          <h3 style="margin-top: var(--space-lg); margin-bottom: var(--space-sm); color: var(--text-primary);">Current Status (W1.3):</h3>
          <ul class="setup-steps">
            <li><span class="badge badge-success">✓</span> Leptos App structure with SPA Router</li>
            <li><span class="badge badge-success">✓</span> Header + Sidebar components</li>
            <li><span class="badge badge-success">✓</span> Dark theme CSS system</li>
            <li><span class="badge badge-success">✓</span> Page stubs (Dashboard, Sessions, Analytics)</li>
            <li><span class="badge badge-warning">⏳</span> WASM build (run commands above)</li>
          </ul>

          <div style="margin-top: var(--space-xl); padding-top: var(--space-lg); border-top: 1px solid var(--border-color);">
            <h3 style="margin-bottom: var(--space-sm); color: var(--text-primary);">API Endpoints (available now):</h3>
            <ul class="api-list">
              <li><a href="/api/health">/api/health</a> - Health check</li>
              <li><a href="/api/stats">/api/stats</a> - Stats JSON</li>
              <li><a href="/api/sessions">/api/sessions</a> - Sessions preview JSON</li>
              <li><a href="/api/events">/api/events</a> - SSE stream</li>
              <li><a href="/static/style.css">/static/style.css</a> - Dark theme stylesheet</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</body>
</html>"#
            .to_string(),
    )
}

async fn stats_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let stats = store.stats();
    match stats {
        Some(s) => axum::Json(serde_json::to_value(s).unwrap_or(serde_json::Value::Null)),
        None => axum::Json(serde_json::json!({"error": "Stats not loaded"})),
    }
}

async fn sessions_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let count = store.session_count();
    let recent = store.recent_sessions(10);

    axum::Json(serde_json::json!({
        "count": count,
        "recent": recent.iter().map(|s| serde_json::json!({
            "id": s.id,
            "project": s.project_path,
            "tokens": s.total_tokens,
            "messages": s.message_count,
            "preview": s.first_user_message,
        })).collect::<Vec<_>>()
    }))
}

async fn health_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let state = store.degraded_state();
    axum::Json(serde_json::json!({
        "status": if state.is_healthy() { "healthy" } else { "degraded" },
        "sessions": store.session_count(),
        "stats_loaded": store.stats().is_some(),
    }))
}

/// SSE endpoint for live updates
async fn sse_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::response::Sse<
    impl futures::stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
> {
    // Clone EventBus to avoid lifetime issues (it's cheap - Arc internally)
    let event_bus = store.event_bus().clone();
    sse::create_sse_stream(event_bus)
}
