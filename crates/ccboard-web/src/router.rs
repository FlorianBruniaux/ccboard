//! Web router using Axum

use axum::{Router, extract::Query, routing::get};
use ccboard_core::DataStore;
use serde::Deserialize;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::sse;

/// Query parameters for sessions pagination
#[derive(Debug, Deserialize)]
struct SessionsQuery {
    #[serde(default)]
    page: usize,
    #[serde(default = "default_page_size")]
    limit: usize,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    since: Option<String>, // e.g., "7d", "30d"
    #[serde(default = "default_sort")]
    sort: String, // "date", "tokens", "cost"
    #[serde(default = "default_order")]
    order: String, // "asc", "desc"
}

fn default_page_size() -> usize {
    50
}
fn default_sort() -> String {
    "date".to_string()
}
fn default_order() -> String {
    "desc".to_string()
}

/// Query parameters for recent sessions
#[derive(Debug, Deserialize)]
struct RecentQuery {
    #[serde(default = "default_recent_limit")]
    limit: usize,
}

fn default_recent_limit() -> usize {
    5
}

/// Create the web router
pub fn create_router(store: Arc<DataStore>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve WASM frontend files (JS, WASM, CSS) from dist/
    // SPA routing: fallback to index.html for client-side routes
    let dist_dir = ServeDir::new("crates/ccboard-web/dist")
        .not_found_service(ServeFile::new("crates/ccboard-web/dist/index.html"));

    Router::new()
        // API routes (must be before catch-all static files)
        .route("/api/stats", get(stats_handler))
        .route("/api/sessions/recent", get(recent_sessions_handler)) // Must be before /api/sessions
        .route("/api/sessions", get(sessions_handler))
        .route("/api/health", get(health_handler))
        .route("/api/events", get(sse_handler))
        // Static assets from static/ folder
        .nest_service("/static", ServeDir::new("crates/ccboard-web/static"))
        // Serve WASM frontend + SPA fallback to index.html
        .fallback_service(dist_dir)
        .layer(cors)
        .with_state(store)
}

async fn stats_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let stats = store.stats();

    // Compute analytics for last 30 days
    let sessions = store.all_sessions();
    let analytics = ccboard_core::analytics::AnalyticsData::compute(
        &sessions,
        ccboard_core::analytics::Period::last_30d(),
    );

    // Extract forecast data points for chart
    let historical_tokens: Vec<u64> = analytics.trends.daily_tokens.clone();
    let forecast_tokens: Vec<u64> = {
        let mut forecast = Vec::new();
        if analytics.trends.dates.len() >= 7 {
            // Extend with 30 days forecast using linear projection
            for i in 1..=30 {
                // Simple linear extrapolation from last value + trend
                let projected = analytics.forecast.next_30_days_tokens as f64 / 30.0 * i as f64;
                forecast.push(projected as u64);
            }
        }
        forecast
    };

    // Get top 5 projects by cost
    let projects_by_cost: Vec<serde_json::Value> = {
        let mut project_costs: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        for session in &sessions {
            let cost = calculate_session_cost(
                session.input_tokens,
                session.output_tokens,
                session.cache_creation_tokens,
                session.cache_read_tokens,
                &session.models_used,
            );
            *project_costs
                .entry(session.project_path.clone())
                .or_insert(0.0) += cost;
        }

        let mut sorted: Vec<_> = project_costs.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let total_cost: f64 = sorted.iter().map(|(_, c)| c).sum();

        sorted
            .iter()
            .take(5)
            .map(|(project, cost)| {
                let percentage = if total_cost > 0.0 {
                    cost / total_cost * 100.0
                } else {
                    0.0
                };
                serde_json::json!({
                    "project": project,
                    "cost": cost,
                    "percentage": percentage,
                })
            })
            .collect()
    };

    // Get most used model
    let most_used_model = analytics
        .trends
        .model_usage_over_time
        .iter()
        .max_by_key(|(_, counts)| counts.iter().sum::<usize>())
        .map(|(model, counts)| {
            let total: usize = counts.iter().sum();
            serde_json::json!({
                "name": model,
                "count": total,
            })
        });

    match stats {
        Some(s) => {
            let mut value = serde_json::to_value(s).unwrap_or(serde_json::Value::Null);

            // Inject analytics data
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "dailyTokens30d".to_string(),
                    serde_json::json!(historical_tokens),
                );
                obj.insert(
                    "forecastTokens30d".to_string(),
                    serde_json::json!(forecast_tokens),
                );
                obj.insert(
                    "forecastConfidence".to_string(),
                    serde_json::json!(analytics.forecast.confidence),
                );
                obj.insert(
                    "forecastCost30d".to_string(),
                    serde_json::json!(analytics.forecast.next_30_days_cost),
                );
                obj.insert(
                    "projectsByCost".to_string(),
                    serde_json::json!(projects_by_cost),
                );
                obj.insert(
                    "mostUsedModel".to_string(),
                    serde_json::json!(most_used_model),
                );

                // Add aggregated totals for current month
                let total_cost: f64 = sessions
                    .iter()
                    .map(|s| {
                        calculate_session_cost(
                            s.input_tokens,
                            s.output_tokens,
                            s.cache_creation_tokens,
                            s.cache_read_tokens,
                            &s.models_used,
                        )
                    })
                    .sum();
                let avg_session_cost = if sessions.len() > 0 {
                    total_cost / sessions.len() as f64
                } else {
                    0.0
                };

                obj.insert("thisMonthCost".to_string(), serde_json::json!(total_cost));
                obj.insert(
                    "avgSessionCost".to_string(),
                    serde_json::json!(avg_session_cost),
                );
            }

            axum::Json(value)
        }
        None => axum::Json(serde_json::json!({"error": "Stats not loaded"})),
    }
}

/// Recent sessions handler (lightweight, for dashboard)
async fn recent_sessions_handler(
    Query(params): Query<RecentQuery>,
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let mut all_sessions = store.all_sessions();

    // Sort by date desc
    all_sessions.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

    let sessions: Vec<_> = all_sessions
        .iter()
        .take(params.limit)
        .map(|s| session_to_json(s))
        .collect();

    axum::Json(serde_json::json!({
        "sessions": sessions,
        "total": all_sessions.len() as u64,
    }))
}

/// Sessions handler with pagination and filters
async fn sessions_handler(
    Query(params): Query<SessionsQuery>,
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let mut all_sessions = store.all_sessions();

    // Filter by search
    if let Some(ref search) = params.search {
        let search_lower = search.to_lowercase();
        all_sessions.retain(|s| {
            s.id.to_lowercase().contains(&search_lower)
                || s.project_path.to_lowercase().contains(&search_lower)
                || s.first_user_message
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&search_lower))
                    .unwrap_or(false)
        });
    }

    // Filter by project
    if let Some(ref project) = params.project {
        all_sessions.retain(|s| s.project_path.contains(project));
    }

    // Filter by model
    if let Some(ref model) = params.model {
        all_sessions.retain(|s| s.models_used.iter().any(|m| m.contains(model)));
    }

    // Filter by time range (since)
    if let Some(ref since) = params.since {
        if let Some(cutoff) = parse_since(since) {
            all_sessions.retain(|s| s.last_timestamp.map(|t| t >= cutoff).unwrap_or(false));
        }
    }

    // Sort
    match params.sort.as_str() {
        "date" => all_sessions.sort_by(|a, b| {
            if params.order == "asc" {
                a.last_timestamp.cmp(&b.last_timestamp)
            } else {
                b.last_timestamp.cmp(&a.last_timestamp)
            }
        }),
        "tokens" => all_sessions.sort_by(|a, b| {
            if params.order == "asc" {
                a.total_tokens.cmp(&b.total_tokens)
            } else {
                b.total_tokens.cmp(&a.total_tokens)
            }
        }),
        "cost" => all_sessions.sort_by(|a, b| {
            let cost_a = calculate_session_cost(
                a.input_tokens,
                a.output_tokens,
                a.cache_creation_tokens,
                a.cache_read_tokens,
                &a.models_used,
            );
            let cost_b = calculate_session_cost(
                b.input_tokens,
                b.output_tokens,
                b.cache_creation_tokens,
                b.cache_read_tokens,
                &b.models_used,
            );
            if params.order == "asc" {
                cost_a
                    .partial_cmp(&cost_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                cost_b
                    .partial_cmp(&cost_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        _ => {} // Keep current order
    }

    let total = all_sessions.len();
    let page_size = params.limit.min(100); // Cap at 100
    let offset = params.page * page_size;

    let sessions: Vec<_> = all_sessions
        .iter()
        .skip(offset)
        .take(page_size)
        .map(|s| session_to_json(s))
        .collect();

    axum::Json(serde_json::json!({
        "sessions": sessions,
        "total": total as u64,
        "page": params.page,
        "page_size": page_size,
    }))
}

/// Convert session to JSON (shared helper)
fn session_to_json(s: &ccboard_core::models::SessionMetadata) -> serde_json::Value {
    let cost = calculate_session_cost(
        s.input_tokens,
        s.output_tokens,
        s.cache_creation_tokens,
        s.cache_read_tokens,
        &s.models_used,
    );

    serde_json::json!({
        "id": s.id,
        "date": s.last_timestamp.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
        "project": s.project_path,
        "model": s.models_used.first().unwrap_or(&"unknown".to_string()),
        "messages": s.message_count,
        "tokens": s.total_tokens,
        "input_tokens": s.input_tokens,
        "output_tokens": s.output_tokens,
        "cache_creation_tokens": s.cache_creation_tokens,
        "cache_read_tokens": s.cache_read_tokens,
        "cost": cost,
        "status": "completed",
        "first_timestamp": s.first_timestamp.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
        "duration_seconds": s.duration_seconds,
        "preview": s.first_user_message,
    })
}

/// Parse "since" parameter (e.g., "7d", "30d", "1h")
fn parse_since(since: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let now = chrono::Utc::now();
    if let Some(days) = since.strip_suffix('d') {
        if let Ok(d) = days.parse::<i64>() {
            return Some(now - chrono::Duration::days(d));
        }
    }
    if let Some(hours) = since.strip_suffix('h') {
        if let Ok(h) = hours.parse::<i64>() {
            return Some(now - chrono::Duration::hours(h));
        }
    }
    None
}

/// Calculate rough cost estimate for a session
fn calculate_session_cost(
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    models: &[String],
) -> f64 {
    // Default to Sonnet 4.5 pricing if unknown model
    // Input: $3/MTok, Output: $15/MTok, Cache write: $3.75/MTok, Cache read: $0.30/MTok
    let is_opus = models.iter().any(|m| m.contains("opus"));
    let is_haiku = models.iter().any(|m| m.contains("haiku"));

    let (input_price, output_price, cache_write_price, cache_read_price) = if is_opus {
        (15.0, 75.0, 18.75, 1.5) // Opus 4 pricing
    } else if is_haiku {
        (0.8, 4.0, 1.0, 0.08) // Haiku 4 pricing
    } else {
        (3.0, 15.0, 3.75, 0.3) // Sonnet 4.5 pricing (default)
    };

    let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;
    let cache_write_cost = (cache_creation_tokens as f64 / 1_000_000.0) * cache_write_price;
    let cache_read_cost = (cache_read_tokens as f64 / 1_000_000.0) * cache_read_price;

    input_cost + output_cost + cache_write_cost + cache_read_cost
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
