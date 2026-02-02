//! Web router using Axum

use axum::{Router, routing::get};
use ccboard_core::DataStore;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

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
        .layer(cors)
        .with_state(store)
}

async fn index_handler() -> &'static str {
    "ccboard web UI - Coming soon"
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
