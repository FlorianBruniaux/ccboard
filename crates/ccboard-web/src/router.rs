//! Web router using Axum

use axum::{extract::Query, routing::get, Router};
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
        .route("/api/quota", get(quota_handler))
        .route("/api/sessions/recent", get(recent_sessions_handler)) // Must be before /api/sessions
        .route("/api/sessions/live", get(live_sessions_handler)) // Live sessions with CPU/RAM
        .route("/api/sessions", get(sessions_handler))
        .route("/api/config/merged", get(config_handler))
        .route("/api/hooks", get(hooks_handler))
        .route("/api/mcp", get(mcp_handler))
        .route("/api/agents", get(agents_handler))
        .route("/api/commands", get(commands_handler))
        .route("/api/skills", get(skills_handler))
        .route("/api/plugins", get(plugins_handler))
        .route("/api/task-graph", get(task_graph_handler))
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
                .entry(session.project_path.as_str().to_string())
                .or_insert(0.0) += cost;
        }

        let mut sorted: Vec<_> = project_costs.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
            let mut value = serde_json::to_value(&s).unwrap_or(serde_json::Value::Null);

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

                // Add cache hit ratio
                let cache_hit_ratio = s.cache_ratio();
                obj.insert(
                    "cacheHitRatio".to_string(),
                    serde_json::json!(cache_hit_ratio),
                );

                // Add MCP servers count
                let mcp_count = store.mcp_config().map(|c| c.servers.len()).unwrap_or(0);
                obj.insert("mcpServersCount".to_string(), serde_json::json!(mcp_count));
            }

            axum::Json(value)
        }
        None => axum::Json(serde_json::json!({"error": "Stats not loaded"})),
    }
}

/// Quota status handler (budget tracking)
async fn quota_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    match store.quota_status() {
        Some(quota) => {
            // Serialize QuotaStatus + add alert_level as string for easier frontend handling
            let alert_level_str = match quota.alert_level {
                ccboard_core::quota::AlertLevel::Safe => "safe",
                ccboard_core::quota::AlertLevel::Warning => "warning",
                ccboard_core::quota::AlertLevel::Critical => "critical",
                ccboard_core::quota::AlertLevel::Exceeded => "exceeded",
            };

            axum::Json(serde_json::json!({
                "current_cost": quota.current_cost,
                "budget_limit": quota.budget_limit,
                "usage_pct": quota.usage_pct,
                "projected_monthly_cost": quota.projected_monthly_cost,
                "projected_overage": quota.projected_overage,
                "alert_level": alert_level_str,
            }))
        }
        None => axum::Json(serde_json::json!({
            "error": "No budget configured or stats not loaded",
        })),
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

/// Live sessions handler - returns active Claude Code processes with CPU/RAM
async fn live_sessions_handler() -> axum::Json<serde_json::Value> {
    use ccboard_core::detect_live_sessions;

    const MAX_LIVE_SESSIONS: usize = 20;

    match detect_live_sessions() {
        Ok(live_sessions) => {
            let total = live_sessions.len();
            let truncated = total > MAX_LIVE_SESSIONS;

            let sessions: Vec<_> = live_sessions
                .iter()
                .take(MAX_LIVE_SESSIONS)
                .map(|ls| {
                    serde_json::json!({
                        "pid": ls.pid,
                        "startTime": ls.start_time.to_rfc3339(),
                        "workingDirectory": ls.working_directory,
                        "command": ls.command,
                        "cpuPercent": ls.cpu_percent,
                        "memoryMb": ls.memory_mb,
                        "tokens": ls.tokens,
                        "sessionId": ls.session_id,
                        "sessionName": ls.session_name,
                    })
                })
                .collect();

            axum::Json(serde_json::json!({
                "sessions": sessions,
                "total": total,
                "displayed": sessions.len(),
                "truncated": truncated,
            }))
        }
        Err(e) => axum::Json(serde_json::json!({
            "sessions": [],
            "total": 0,
            "error": format!("Failed to detect live sessions: {}", e),
        })),
    }
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
        "model": s.models_used.first().map(|s| s.as_str()).unwrap_or("unknown"),
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

/// Plugin analytics handler
async fn plugins_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    use ccboard_core::analytics::aggregate_plugin_usage;
    use chrono::Utc;

    // Get all sessions (limit to recent 10000 for performance)
    let sessions = store.recent_sessions(10000);

    // Extract skill and command names from invocation stats
    let invocation_stats = store.invocation_stats();
    let skills: Vec<String> = invocation_stats.skills.keys().cloned().collect();
    let commands: Vec<String> = invocation_stats.commands.keys().cloned().collect();

    // Compute plugin analytics
    let analytics = aggregate_plugin_usage(&sessions, &skills, &commands);

    // Return JSON response
    axum::Json(serde_json::json!({
        "analytics": analytics,
        "generated_at": Utc::now().to_rfc3339(),
    }))
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

/// Config handler - returns merged settings (global + project + local)
async fn config_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let settings = store.settings();
    axum::Json(serde_json::to_value(&settings).unwrap_or_default())
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

/// Hooks handler - returns all hooks from merged settings
async fn hooks_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    let settings = store.settings();

    let mut hooks_list = Vec::new();

    if let Some(hooks_map) = &settings.merged.hooks {
        for (event_name, hook_groups) in hooks_map {
            for (group_idx, hook_group) in hook_groups.iter().enumerate() {
                for (hook_idx, hook_def) in hook_group.hooks.iter().enumerate() {
                    // Generate unique hook name
                    let hook_name = if hook_groups.len() == 1 && hook_group.hooks.len() == 1 {
                        event_name.clone()
                    } else {
                        format!("{}-{}-{}", event_name, group_idx, hook_idx)
                    };

                    // Try to read script content if command points to a .sh file
                    let (script_path, script_content) = if hook_def.command.ends_with(".sh") {
                        let path = std::path::Path::new(&hook_def.command);
                        let content = std::fs::read_to_string(path).ok();
                        (Some(hook_def.command.clone()), content)
                    } else {
                        (None, None)
                    };

                    hooks_list.push(serde_json::json!({
                        "name": hook_name,
                        "event": event_name,
                        "command": hook_def.command,
                        "description": extract_description(&hook_def.command, script_content.as_deref()),
                        "async": hook_def.r#async.unwrap_or(false),
                        "timeout": hook_def.timeout,
                        "cwd": hook_def.cwd,
                        "matcher": hook_group.matcher,
                        "scriptPath": script_path,
                        "scriptContent": script_content,
                    }));
                }
            }
        }
    }

    axum::Json(serde_json::json!({
        "hooks": hooks_list,
        "total": hooks_list.len(),
    }))
}

/// Extract description from script content (look for # Description: comment)
fn extract_description(command: &str, script_content: Option<&str>) -> Option<String> {
    if let Some(content) = script_content {
        for line in content.lines().take(20) {
            let trimmed = line.trim();
            if trimmed.starts_with("# Description:") {
                return Some(
                    trimmed
                        .trim_start_matches("# Description:")
                        .trim()
                        .to_string(),
                );
            }
        }
    }

    // Fallback: use command as description
    Some(command.to_string())
}

/// MCP handler - returns MCP server configuration
async fn mcp_handler(
    axum::extract::State(store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    match store.mcp_config() {
        Some(config) => {
            let mut servers_list = Vec::new();

            for (name, server) in &config.servers {
                servers_list.push(serde_json::json!({
                    "name": name,
                    "command": server.display_command(),
                    "serverType": if server.is_http() { "http" } else { "stdio" },
                    "url": server.url,
                    "args": server.args,
                    "env": server.env,
                    "hasEnv": !server.env.is_empty(),
                }));
            }

            axum::Json(serde_json::json!({
                "servers": servers_list,
                "total": servers_list.len(),
            }))
        }
        None => axum::Json(serde_json::json!({
            "servers": [],
            "total": 0,
        })),
    }
}

/// Helper to scan markdown files from a directory
fn scan_markdown_files(dir_path: &std::path::Path) -> Vec<serde_json::Value> {
    let mut items = Vec::new();

    if !dir_path.exists() {
        return items;
    }

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let (frontmatter, body) = parse_frontmatter(&content);
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    items.push(serde_json::json!({
                        "name": name,
                        "frontmatter": frontmatter,
                        "body": body,
                        "path": path.to_string_lossy(),
                    }));
                }
            }
        }
    }

    // Sort by name
    items.sort_by(|a, b| {
        let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    items
}

/// Parse frontmatter (YAML between ---) and body
fn parse_frontmatter(content: &str) -> (serde_json::Value, String) {
    let lines: Vec<&str> = content.lines().collect();

    // Check if starts with ---
    if lines.first() != Some(&"---") {
        return (serde_json::json!({}), content.to_string());
    }

    // Find closing ---
    if let Some(end_idx) = lines[1..].iter().position(|&line| line == "---") {
        let yaml_lines = &lines[1..=end_idx];
        let body_lines = &lines[end_idx + 2..];

        let yaml_str = yaml_lines.join("\n");
        let frontmatter: serde_json::Value =
            serde_yaml::from_str(&yaml_str).unwrap_or_else(|_| serde_json::json!({}));

        let body = body_lines.join("\n");

        (frontmatter, body)
    } else {
        (serde_json::json!({}), content.to_string())
    }
}

/// Agents handler - returns agents from ~/.claude/agents/
async fn agents_handler() -> axum::Json<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let agents_dir = std::path::Path::new(&home).join(".claude/agents");
    let agents = scan_markdown_files(&agents_dir);

    axum::Json(serde_json::json!({
        "items": agents,
        "total": agents.len(),
    }))
}

/// Commands handler - returns commands from ~/.claude/commands/
async fn commands_handler() -> axum::Json<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let commands_dir = std::path::Path::new(&home).join(".claude/commands");
    let commands = scan_markdown_files(&commands_dir);

    axum::Json(serde_json::json!({
        "items": commands,
        "total": commands.len(),
    }))
}

/// Skills handler - returns skills from ~/.claude/skills/ subdirectories
async fn skills_handler() -> axum::Json<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let skills_dir = std::path::Path::new(&home).join(".claude/skills");
    let skills = scan_skills_recursive(&skills_dir);

    axum::Json(serde_json::json!({
        "items": skills,
        "total": skills.len(),
    }))
}

/// Helper to scan skills from subdirectories (looks for SKILL.md files)
fn scan_skills_recursive(dir_path: &std::path::Path) -> Vec<serde_json::Value> {
    let mut items = Vec::new();

    if !dir_path.exists() {
        return items;
    }

    // Scan subdirectories for SKILL.md files
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&skill_file) {
                        let (frontmatter, body) = parse_frontmatter(&content);
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        items.push(serde_json::json!({
                            "name": name,
                            "frontmatter": frontmatter,
                            "body": body,
                            "path": skill_file.to_string_lossy(),
                        }));
                    }
                }
            }
        }
    }

    // Sort by name
    items.sort_by(|a, b| {
        let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    items
}

/// Task graph handler - returns task dependency graph from PLAN.md
async fn task_graph_handler(
    axum::extract::State(_store): axum::extract::State<Arc<DataStore>>,
) -> axum::Json<serde_json::Value> {
    use ccboard_core::graph::TaskGraph;
    use ccboard_core::models::plan::PhaseStatus;
    use ccboard_core::parsers::PlanParser;

    // Try multiple PLAN.md locations (prioritize working dir)
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let possible_paths = vec![
        // Current project claudedocs/PLAN.md (highest priority - dev dir)
        cwd.join("claudedocs/PLAN.md"),
        // Current project .claude/PLAN.md
        cwd.join(".claude/PLAN.md"),
        // Claude home claudedocs/PLAN.md
        std::path::PathBuf::from(&home).join(".claude/claudedocs/PLAN.md"),
    ];

    let plan_path = possible_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| cwd.join("PLAN.md"));

    let plan_result = PlanParser::parse_file(&plan_path);

    match plan_result {
        Ok(Some(plan)) => {
            // Build task graph
            let mut graph = TaskGraph::new();

            // Add all tasks to graph
            for phase in &plan.phases {
                for task in &phase.tasks {
                    graph.add_task(task.clone());
                }
            }

            // Parse dependencies from PLAN.md content directly
            // Read the file again to extract dependency information
            if let Ok(plan_content) = std::fs::read_to_string(&plan_path) {
                // Look for task sections and extract dependencies
                for phase in &plan.phases {
                    for task in &phase.tasks {
                        // Find this task in the content
                        let task_header = format!("#### Task {}:", task.id);
                        if let Some(pos) = plan_content.find(&task_header) {
                            // Extract content until next task or section
                            let rest = &plan_content[pos..];
                            // Use char boundary-safe slicing
                            let skip_offset = rest
                                .char_indices()
                                .nth(100)
                                .map(|(i, _)| i)
                                .unwrap_or(rest.len());
                            let end_pos = rest
                                .get(skip_offset..)
                                .and_then(|s| s.find("\n####").map(|p| p + skip_offset))
                                .or_else(|| {
                                    rest.get(skip_offset..)
                                        .and_then(|s| s.find("\n###").map(|p| p + skip_offset))
                                })
                                .or_else(|| {
                                    rest.get(skip_offset..)
                                        .and_then(|s| s.find("\n##").map(|p| p + skip_offset))
                                })
                                .unwrap_or(rest.len());

                            let task_content = &rest[..end_pos];

                            // Extract dependencies
                            let deps = extract_dependencies(task_content);
                            for dep_id in deps {
                                // Add dependency: dep_id blocks task_id
                                let _ = graph.add_dependency(&dep_id, &task.id);
                            }
                        }
                    }
                }
            }

            // Convert to JSON format for D3.js
            let nodes: Vec<_> = plan
                .phases
                .iter()
                .flat_map(|phase| {
                    phase.tasks.iter().map(move |task| {
                        let status = match phase.status {
                            PhaseStatus::Complete => "Complete",
                            PhaseStatus::InProgress => "InProgress",
                            PhaseStatus::Future => "Future",
                        };

                        serde_json::json!({
                            "id": task.id,
                            "label": task.title,
                            "phase": phase.id,
                            "status": status,
                            "duration": task.duration,
                            "description": task.description,
                            "priority": task.priority,
                            "difficulty": task.difficulty,
                            "crateName": task.crate_name,
                            "issue": task.issue,
                        })
                    })
                })
                .collect();

            // Extract edges from graph
            let mut edges = Vec::new();
            for task in graph.tasks() {
                let dependents = graph.dependents(&task.id);
                for dependent in dependents {
                    edges.push(serde_json::json!({
                        "source": task.id,
                        "target": dependent,
                        "type": "blocks",
                    }));
                }
            }

            axum::Json(serde_json::json!({
                "nodes": nodes,
                "edges": edges,
            }))
        }
        Ok(None) => {
            // PLAN.md not found
            axum::Json(serde_json::json!({
                "nodes": [],
                "edges": [],
                "error": "PLAN.md not found",
            }))
        }
        Err(e) => {
            // Parse error
            axum::Json(serde_json::json!({
                "nodes": [],
                "edges": [],
                "error": format!("Failed to parse PLAN.md: {}", e),
            }))
        }
    }
}

/// Extract dependency task IDs from description text
/// Looks for patterns like "Depends on: F.1, F.2" or "Blocked by: #6, #7"
fn extract_dependencies(description: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // Pattern 1: "Depends on: F.1, F.2"
    if let Some(start) = description.find("Depends on:") {
        let rest = &description[start + "Depends on:".len()..];
        // Take until newline or end
        let dep_text = rest.lines().next().unwrap_or("");

        // Extract task IDs like F.1, H.2, etc.
        for part in dep_text.split(',') {
            let trimmed = part.trim();
            // Match pattern: Letter.Number (e.g., F.1, H.2)
            if trimmed
                .chars()
                .next()
                .map(|c| c.is_alphabetic())
                .unwrap_or(false)
            {
                if let Some(task_id) = trimmed.split_whitespace().next() {
                    deps.push(task_id.to_string());
                }
            }
        }
    }

    // Pattern 2: "Blocked by: #6, #7" - extract task numbers
    if let Some(start) = description.find("Blocked by:") {
        let rest = &description[start + "Blocked by:".len()..];
        let dep_text = rest.lines().next().unwrap_or("");

        for part in dep_text.split(',') {
            let trimmed = part.trim();
            if let Some(task_num) = trimmed.strip_prefix('#') {
                if let Ok(_num) = task_num.parse::<u32>() {
                    // For now, skip numeric task IDs (would need task ID mapping)
                    // We'll focus on letter.number format (F.1, H.2, etc.)
                }
            }
        }
    }

    deps
}
