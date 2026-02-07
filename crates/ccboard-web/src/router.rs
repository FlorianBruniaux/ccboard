//! Web router using Axum

use axum::{Router, response::Html, routing::get};
use ccboard_core::DataStore;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

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
        .layer(cors)
        .with_state(store)
}

async fn index_handler() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ccboard - Claude Code Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: system-ui, -apple-system, sans-serif;
            background: #f5f5f5;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
        }
        .setup-message {
            max-width: 600px;
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 {
            font-size: 2rem;
            margin-bottom: 1rem;
            color: #1a1a1a;
        }
        p {
            margin-bottom: 1rem;
            color: #333;
            line-height: 1.6;
        }
        code {
            background: #f0f0f0;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-family: monospace;
        }
        .step {
            margin: 1.5rem 0;
            padding: 1rem;
            background: #f8f8f8;
            border-left: 3px solid #333;
        }
        .api-links {
            margin-top: 2rem;
            padding-top: 1.5rem;
            border-top: 1px solid #ddd;
        }
        a {
            color: #0066cc;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="setup-message">
        <h1>🚧 ccboard Web UI - Build Required</h1>
        <p>The Leptos WASM frontend needs to be compiled before the web UI can be displayed.</p>

        <div class="step">
            <strong>Setup Instructions:</strong>
            <ol style="margin-left: 1.5rem; margin-top: 0.5rem;">
                <li>Install Trunk: <code>cargo install trunk</code></li>
                <li>Add WASM target: <code>rustup target add wasm32-unknown-unknown</code></li>
                <li>Build frontend: <code>cd crates/ccboard-web && trunk build --release</code></li>
                <li>Restart server: <code>cargo run -- web --port 3333</code></li>
            </ol>
        </div>

        <p><strong>Current Status (W1.1):</strong></p>
        <ul style="margin-left: 1.5rem;">
            <li>✅ Leptos App structure with SPA Router</li>
            <li>✅ Header + Sidebar components</li>
            <li>✅ Page stubs (Dashboard, Sessions, Analytics)</li>
            <li>⏳ WASM build (run commands above)</li>
        </ul>

        <div class="api-links">
            <p><strong>API Endpoints (available now):</strong></p>
            <ul style="margin-left: 1.5rem;">
                <li><a href="/api/health">/api/health</a> - Health check</li>
                <li><a href="/api/stats">/api/stats</a> - Stats JSON</li>
                <li><a href="/api/sessions">/api/sessions</a> - Sessions JSON</li>
            </ul>
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
