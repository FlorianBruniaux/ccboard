//! Dashboard page component

use crate::api::{fetch_recent_sessions, fetch_stats, format_cost, format_number};
use crate::components::{use_toast, CardColor, Sparkline, StatsCard};
use crate::sse_hook::{use_sse, SseEvent};
use crate::utils::export_as_json;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LiveSession {
    pid: Option<u32>,
    #[serde(rename = "startTime")]
    start_time: Option<String>,
    #[serde(rename = "workingDirectory")]
    working_directory: Option<String>,
    #[serde(rename = "cpuPercent")]
    cpu_percent: Option<f64>,
    #[serde(rename = "memoryMb")]
    memory_mb: Option<f64>,
    tokens: Option<u64>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "sessionName")]
    session_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LiveSessionsResponse {
    sessions: Vec<LiveSession>,
    total: u64,
    #[serde(default)]
    truncated: bool,
}

async fn fetch_live_sessions() -> Result<LiveSessionsResponse, String> {
    let response = gloo_net::http::Request::get("/api/sessions/live")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }

    response
        .json::<LiveSessionsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

fn project_name(working_dir: &Option<String>) -> String {
    working_dir
        .as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("unknown")
        .to_string()
}

fn elapsed(start_time: &Option<String>) -> String {
    let Some(ts) = start_time else {
        return String::new();
    };
    let Ok(start) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return String::new();
    };
    let now = chrono::Utc::now();
    let secs = (now - start.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0) as u64;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Dashboard page - main overview with live stats
#[component]
pub fn Dashboard() -> impl IntoView {
    // Stats resource for initial load and manual refresh
    let (stats_version, set_stats_version) = signal(0u32);
    let stats = LocalResource::new(move || {
        let _ = stats_version.get(); // Track this to trigger refetch
        async move { fetch_stats().await }
    });

    // Sessions resource for recent sessions list (limit 5 for dashboard)
    let sessions = LocalResource::new(move || {
        let _ = stats_version.get(); // Track this to trigger refetch
        async move { fetch_recent_sessions(5).await }
    });

    // Live sessions — auto-refresh every 5s
    let (live_version, set_live_version) = signal(0u32);
    let live_sessions = LocalResource::new(move || {
        let _ = live_version.get();
        async move { fetch_live_sessions().await }
    });

    // setInterval every 5s to refresh live sessions
    {
        use wasm_bindgen::JsCast;
        let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            set_live_version.update(|v| *v += 1);
        }) as Box<dyn Fn()>);
        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                5_000,
            )
            .unwrap();
        cb.forget();
    }

    // Toast notifications
    let toast = use_toast();

    // Navigation for clickable cards
    let navigate = use_navigate();

    // SSE setup for live updates
    let sse_event = use_sse();

    Effect::new(move |_| {
        if let Some(event) = sse_event.get() {
            match event {
                SseEvent::StatsUpdated => {
                    set_stats_version.update(|v| *v += 1);
                    toast.info("Stats updated".to_string());
                }
                SseEvent::SessionCreated { .. } => {
                    set_stats_version.update(|v| *v += 1);
                    toast.info("New session detected".to_string());
                }
                SseEvent::AnalyticsUpdated => {
                    set_stats_version.update(|v| *v += 1);
                    toast.info("Analytics refreshed".to_string());
                }
                SseEvent::WatcherError { message } => {
                    toast.error(format!("Watcher error: {}", message));
                }
                _ => {}
            }
        }
    });

    view! {
        <div class="page dashboard-page">
            <div class="page-header">
                <h2>"Dashboard"</h2>
                <div class="page-actions">
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(Ok(data)) = stats.get().as_ref().map(|r| r.as_ref()) {
                                export_as_json(&data, "ccboard-stats");
                            }
                        }
                    >
                        "📥 Export JSON"
                    </button>
                </div>
            </div>
            <div class="page-content">
                <Suspense fallback=move || view! { <div class="loading">"Loading stats..."</div> }>
                    {move || match stats.get().as_ref().map(|r| r.as_ref()) {
                        Some(Ok(data)) => {
                            let total_tokens = data.total_tokens();
                            let total_cost = data.total_cost();
                            let avg_cost = data.avg_session_cost();
                            let this_month = data.this_month_sessions();
                            let this_week = data.this_week_tokens();
                            let daily_tokens = data.daily_tokens_30d();

                            // Additional KPIs
                            let total_messages = data.total_messages;
                            let cache_hit = data.cache_hit_ratio * 100.0; // Convert to percentage
                            let mcp_servers = data.mcp_servers_count;

                            // Determine color based on cost (example thresholds)
                            let cost_color = if total_cost > 100.0 {
                                CardColor::Red
                            } else if total_cost > 50.0 {
                                CardColor::Yellow
                            } else {
                                CardColor::Green
                            };

                            // Clone navigate for each closure
                            let nav1 = navigate.clone();
                            let nav2 = navigate.clone();
                            let nav3 = navigate.clone();
                            let nav4 = navigate.clone();
                            let nav5 = navigate.clone();
                            let nav6 = navigate.clone();

                            view! {
                                <div>
                                    <div class="stats-grid">
                                        <StatsCard
                                            label="Total Sessions".to_string()
                                            value=data.total_sessions.to_string()
                                            icon="📊".to_string()
                                            color=CardColor::Default
                                            on_click=Box::new(move || {
                                                nav1("/sessions", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="Total Tokens".to_string()
                                            value=format_number(total_tokens)
                                            icon="🔢".to_string()
                                            color=CardColor::Default
                                        />
                                        <StatsCard
                                            label="Total Cost".to_string()
                                            value=format_cost(total_cost)
                                            icon="💰".to_string()
                                            color=cost_color
                                            on_click=Box::new(move || {
                                                nav2("/costs", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="Avg Session Cost".to_string()
                                            value=format_cost(avg_cost)
                                            icon="📈".to_string()
                                            color=CardColor::Default
                                            on_click=Box::new(move || {
                                                nav3("/analytics", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="This Month Sessions".to_string()
                                            value=this_month.to_string()
                                            icon="📅".to_string()
                                            color=CardColor::Default
                                            on_click=Box::new(move || {
                                                nav4("/sessions", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="This Week Tokens".to_string()
                                            value=format_number(this_week)
                                            icon="🔥".to_string()
                                            color=CardColor::Default
                                            on_click=Box::new(move || {
                                                nav5("/sessions", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="Total Messages".to_string()
                                            value=format_number(total_messages)
                                            icon="💬".to_string()
                                            color=CardColor::Default
                                            on_click=Box::new(move || {
                                                nav6("/sessions", Default::default());
                                            })
                                        />
                                        <StatsCard
                                            label="Cache Hit Rate".to_string()
                                            value=format!("{:.1}%", cache_hit)
                                            icon="⚡".to_string()
                                            color={if cache_hit > 90.0 { CardColor::Green } else if cache_hit > 70.0 { CardColor::Yellow } else { CardColor::Red }}
                                        />
                                        <StatsCard
                                            label="MCP Servers".to_string()
                                            value=mcp_servers.to_string()
                                            icon="🔌".to_string()
                                            color=CardColor::Default
                                        />
                                    </div>

                                    <div class="sparkline-section">
                                        <Sparkline
                                            data=daily_tokens
                                            width=800
                                            height=100
                                            label="Token Usage - Last 30 Days".to_string()
                                        />
                                    </div>

                                    // Live sessions panel
                                    <div class="live-sessions-section">
                                        <div class="live-sessions-header">
                                            <h3>
                                                {move || {
                                                    let count = live_sessions.get()
                                                        .and_then(|r| r.as_ref().ok().map(|d| d.total))
                                                        .unwrap_or(0);
                                                    if count > 0 {
                                                        view! {
                                                            <span>
                                                                <span class="live-dot live-dot--active"></span>
                                                                {format!("Live Sessions ({})", count)}
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span>
                                                                <span class="live-dot"></span>
                                                                "Live Sessions"
                                                            </span>
                                                        }.into_any()
                                                    }
                                                }}
                                            </h3>
                                        </div>
                                        {move || live_sessions.get().map(|result| match &*result {
                                            Ok(data) if data.sessions.is_empty() => view! {
                                                <p class="live-sessions-empty">{"No active Claude sessions detected."}</p>
                                            }.into_any(),
                                            Ok(data) => {
                                                let sessions = data.sessions.clone();
                                                view! {
                                                    <div class="live-sessions-list">
                                                        {sessions.into_iter().map(|s| {
                                                            let project = project_name(&s.working_directory);
                                                            let time_elapsed = elapsed(&s.start_time);
                                                            let tokens = s.tokens.unwrap_or(0);
                                                            let cpu = s.cpu_percent.unwrap_or(0.0);
                                                            let mem = s.memory_mb.unwrap_or(0.0);
                                                            view! {
                                                                <div class="live-session-row">
                                                                    <span class="live-session-project">{project}</span>
                                                                    <span class="live-session-elapsed">{time_elapsed}</span>
                                                                    <span class="live-session-tokens">{format_number(tokens)} " tok"</span>
                                                                    <span class="live-session-cpu">{format!("{cpu:.0}%")} " CPU"</span>
                                                                    <span class="live-session-mem">{format!("{mem:.0} MB")}</span>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            },
                                            Err(_) => view! { <p class="live-sessions-empty">{"—"}</p> }.into_any(),
                                        })}
                                    </div>

                                    <div class="recent-sessions-section">
                                        <h3>"Recent Sessions"</h3>
                                        <Suspense fallback=move || view! { <div class="loading">"Loading sessions..."</div> }>
                                            {move || match sessions.get().as_ref().map(|r| r.as_ref()) {
                                                Some(Ok(session_data)) => {
                                                    view! {
                                                        <div class="table-container">
                                                            <table class="sessions-table">
                                                                <thead>
                                                                    <tr>
                                                                        <th>"ID"</th>
                                                                        <th>"Project"</th>
                                                                        <th>"Tokens"</th>
                                                                        <th>"Messages"</th>
                                                                        <th>"Preview"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    {session_data.sessions.iter().map(|s| {
                                                                        let id = s.id.clone();
                                                                        let project = s.project.clone();
                                                                        let tokens = s.tokens;
                                                                        let messages = s.messages;
                                                                        let preview = s.preview.clone().unwrap_or_default();
                                                                        let cost = s.cost;

                                                                        // Clone for tooltip
                                                                        let project_tooltip = project.clone();
                                                                        let preview_tooltip = preview.clone();

                                                                        view! {
                                                                            <tr class="session-row">
                                                                                <td class="session-id">{id}</td>
                                                                                <td>{project}</td>
                                                                                <td class="tokens">{format_number(tokens)}</td>
                                                                                <td>{messages}</td>
                                                                                <td class="preview preview--with-tooltip">
                                                                                    <span class="preview-text">{preview}</span>
                                                                                    <div class="session-preview-tooltip">
                                                                                        <div class="preview-header">
                                                                                            <strong>"Project: "</strong>
                                                                                            {project_tooltip}
                                                                                        </div>
                                                                                        <div class="preview-stats">
                                                                                            <span>{format_number(tokens)} " tokens"</span>
                                                                                            <span>{format_cost(cost)}</span>
                                                                                            <span>{messages} " messages"</span>
                                                                                        </div>
                                                                                        <div class="preview-snippet">
                                                                                            {preview_tooltip}
                                                                                        </div>
                                                                                        <div class="preview-cta">
                                                                                            "Click for details →"
                                                                                        </div>
                                                                                    </div>
                                                                                </td>
                                                                            </tr>
                                                                        }
                                                                    }).collect::<Vec<_>>()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    }.into_any()
                                                },
                                                Some(Err(e)) => {
                                                    let err = e.clone();
                                                    view! { <div class="error">"Failed to load sessions: " {err}</div> }.into_any()
                                                },
                                                None => {
                                                    view! { <div class="loading">"Loading sessions..."</div> }.into_any()
                                                }
                                            }}
                                        </Suspense>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        Some(Err(e)) => {
                            let err = e.clone();
                            view! { <div class="error">"Failed to load stats: " {err}</div> }.into_any()
                        },
                        None => {
                            view! { <div class="loading">"Loading stats..."</div> }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
