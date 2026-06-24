//! Brain page — cross-session knowledge base + optional claude-mem session summaries

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "";

/// Insight type constants for display
const TYPE_COLORS: &[(&str, &str)] = &[
    ("progress", "#22c55e"), // green
    ("decision", "#06b6d4"), // cyan
    ("blocked", "#ef4444"),  // red
    ("pattern", "#a855f7"),  // purple
    ("fix", "#f59e0b"),      // amber
    ("context", "#6b7280"),  // gray
];

fn type_color(t: &str) -> &'static str {
    TYPE_COLORS
        .iter()
        .find(|(k, _)| *k == t)
        .map(|(_, v)| *v)
        .unwrap_or("#6b7280")
}

fn type_icon(t: &str) -> &'static str {
    match t {
        "progress" => "●",
        "decision" => "◆",
        "blocked" => "▲",
        "pattern" => "◉",
        "fix" => "✦",
        "context" => "○",
        _ => "·",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InsightData {
    pub id: i64,
    #[serde(default)]
    pub session_id: Option<String>,
    pub project: String,
    pub r#type: String,
    pub content: String,
    #[serde(default)]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub archived: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResponse {
    pub insights: Vec<InsightData>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeMemSummary {
    pub id: i64,
    pub memory_session_id: String,
    pub project: String,
    #[serde(default)]
    pub request: Option<String>,
    #[serde(default)]
    pub completed: Option<String>,
    #[serde(default)]
    pub next_steps: Option<String>,
    #[serde(default)]
    pub files_edited: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMemResponse {
    pub enabled: bool,
    pub summaries: Vec<ClaudeMemSummary>,
    pub total: u64,
}

async fn fetch_insights(filter_type: Option<String>) -> Result<InsightsResponse, String> {
    let mut url = format!("{}/api/insights?limit=200", API_BASE_URL);
    if let Some(t) = filter_type {
        url.push_str(&format!("&type={t}"));
    }

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<InsightsResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

async fn fetch_claude_mem() -> Result<ClaudeMemResponse, String> {
    let url = format!("{}/api/claude-mem/summaries", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<ClaudeMemResponse>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

async fn toggle_claude_mem(enabled: bool) -> Result<bool, String> {
    let url = format!("{}/api/claude-mem/toggle", API_BASE_URL);
    let body = serde_json::json!({ "enabled": enabled }).to_string();
    let response = gloo_net::http::Request::post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| format!("Request error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    Ok(enabled)
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn format_date(iso: &str) -> String {
    if iso.len() >= 16 {
        let date_part = &iso[5..10];
        let time_part = &iso[11..16];
        let month = match &iso[5..7] {
            "01" => "Jan",
            "02" => "Feb",
            "03" => "Mar",
            "04" => "Apr",
            "05" => "May",
            "06" => "Jun",
            "07" => "Jul",
            "08" => "Aug",
            "09" => "Sep",
            "10" => "Oct",
            "11" => "Nov",
            "12" => "Dec",
            _ => "",
        };
        format!("{} {} {}", month, &date_part[3..], time_part)
    } else {
        iso.to_string()
    }
}

/// Brain page component
#[component]
pub fn Brain() -> impl IntoView {
    let (filter, set_filter) = signal::<Option<String>>(None);
    let (selected_insight, set_selected_insight) = signal::<Option<InsightData>>(None);
    let (selected_summary, set_selected_summary) = signal::<Option<ClaudeMemSummary>>(None);
    let (mem_refresh, set_mem_refresh) = signal::<u32>(0);

    let insights_res = LocalResource::new(move || {
        let f = filter.get();
        async move { fetch_insights(f).await }
    });

    let claude_mem_res = LocalResource::new(move || {
        let _ = mem_refresh.get();
        async move { fetch_claude_mem().await }
    });

    let filter_tabs = [
        "all", "progress", "decision", "blocked", "pattern", "fix", "context",
    ];

    view! {
        <div class="brain-page">
            <div class="brain-header">
                <h1 class="page-title">{"🧠 Brain"}</h1>
                <p class="page-subtitle">{"Cross-session knowledge base captured by session-stop hook"}</p>
            </div>

            // Filter tabs + claude-mem toggle
            <div class="brain-filter-bar">
                {filter_tabs.iter().map(|&t| {
                    let label = if t == "all" { "All".to_string() } else {
                        let mut s = t.to_string();
                        if let Some(c) = s.get_mut(0..1) { c.make_ascii_uppercase(); }
                        s
                    };
                    let is_active = move || {
                        if t == "all" { filter.get().is_none() }
                        else { filter.get().as_deref() == Some(t) }
                    };
                    let type_val = t.to_string();
                    view! {
                        <button
                            class=move || if is_active() { "filter-btn filter-btn-active" } else { "filter-btn" }
                            on:click=move |_| {
                                if type_val == "all" { set_filter.set(None); }
                                else { set_filter.set(Some(type_val.clone())); }
                                set_selected_insight.set(None);
                            }
                        >
                            {if t != "all" { format!("{} {}", type_icon(t), &label) } else { label }}
                        </button>
                    }
                }).collect_view()}

                // claude-mem toggle button
                <div class="brain-mem-toggle-wrap">
                    {move || claude_mem_res.get().map(|result| {
                        let enabled = result.as_ref().map(|r| r.enabled).unwrap_or(false);
                        view! {
                            <button
                                class=if enabled { "mem-toggle-btn mem-toggle-btn--on" } else { "mem-toggle-btn" }
                                on:click=move |_| {
                                    let new_state = !enabled;
                                    leptos::task::spawn_local(async move {
                                        let _ = toggle_claude_mem(new_state).await;
                                        set_mem_refresh.update(|n| *n += 1);
                                    });
                                }
                                title=if enabled { "Disable claude-mem integration" } else { "Enable claude-mem integration" }
                            >
                                {if enabled { "◈ claude-mem ON" } else { "◈ claude-mem OFF" }}
                            </button>
                        }
                    })}
                </div>
            </div>

            // Main content — ccboard insights
            <div class="brain-content">
                <Suspense fallback=|| view! { <div class="loading">{"Loading insights..."}</div> }>
                    {move || insights_res.get().map(|result| match &*result {
                        Err(e) => view! {
                            <div class="error-state">
                                <p>{format!("Error: {e}")}</p>
                                <p class="error-hint">{"insights.db will be created automatically when session-stop.sh fires."}</p>
                            </div>
                        }.into_any(),
                        Ok(data) if data.insights.is_empty() => view! {
                            <div class="empty-state">
                                <p class="empty-icon">{"🧠"}</p>
                                <p>{"No insights captured yet."}</p>
                                <p class="empty-hint">{"Session-stop hook will auto-capture progress, decisions, and blockers after each meaningful session."}</p>
                            </div>
                        }.into_any(),
                        Ok(data) => {
                            let count = data.insights.len();
                            view! {
                                <div class="brain-body">
                                    <div class="brain-list-panel">
                                        <div class="brain-list-header">
                                            <span class="list-count">{format!("{count} insights")}</span>
                                        </div>
                                        <div class="brain-list">
                                            {data.insights.iter().map(|insight| {
                                                let insight = insight.clone();
                                                let is_selected = {
                                                    let id = insight.id;
                                                    move || selected_insight.get().map(|s| s.id) == Some(id)
                                                };
                                                let insight_click = insight.clone();
                                                view! {
                                                    <div
                                                        class=move || if is_selected() { "brain-list-item selected" } else { "brain-list-item" }
                                                        on:click=move |_| set_selected_insight.set(Some(insight_click.clone()))
                                                    >
                                                        <span class="insight-icon" style=format!("color:{}", type_color(&insight.r#type))>
                                                            {type_icon(&insight.r#type)}
                                                        </span>
                                                        <span class="insight-meta">
                                                            <span class="insight-date">{format_date(&insight.created_at)}</span>
                                                            <span class="insight-project">{basename(&insight.project).to_string()}</span>
                                                        </span>
                                                        <span class="insight-content-preview">{
                                                            if insight.content.len() > 70 {
                                                                format!("{}…", &insight.content[..70])
                                                            } else {
                                                                insight.content.clone()
                                                            }
                                                        }</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    // Insight detail panel
                                    <div class="brain-detail-panel">
                                        {move || selected_insight.get().map(|s| view! {
                                            <div class="insight-detail">
                                                <div class="insight-detail-header">
                                                    <span class="insight-type-badge" style=format!("color:{}", type_color(&s.r#type))>
                                                        {format!("{} {}", type_icon(&s.r#type), s.r#type.to_uppercase())}
                                                    </span>
                                                    <span class="insight-detail-date">{format_date(&s.created_at)}</span>
                                                </div>
                                                <div class="insight-detail-project">{s.project.clone()}</div>
                                                <div class="insight-detail-content">{s.content.clone()}</div>
                                                {s.reasoning.as_ref().map(|r| view! {
                                                    <div class="insight-detail-reasoning">
                                                        <span class="reasoning-label">{"Reasoning:"}</span>
                                                        <span class="reasoning-text">{r.clone()}</span>
                                                    </div>
                                                })}
                                                {s.session_id.as_ref().map(|sid| view! {
                                                    <div class="insight-detail-session">
                                                        <span class="session-label">{"Session:"}</span>
                                                        <code class="session-id">{sid.clone()}</code>
                                                    </div>
                                                })}
                                            </div>
                                        })}
                                        {move || selected_insight.get().is_none().then(|| view! {
                                            <div class="detail-placeholder">
                                                <p>{"← Select an insight to view details"}</p>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>

            // claude-mem section (shown only when enabled)
            {move || claude_mem_res.get().map(|result| {
                match &*result {
                    Ok(data) if data.enabled => {
                        let summaries = data.summaries.clone();
                        view! {
                            <div class="brain-mem-section">
                                <div class="brain-mem-header">
                                    <span class="brain-mem-title">{"◈ claude-mem session summaries"}</span>
                                    <span class="brain-mem-count">{format!("{} sessions", summaries.len())}</span>
                                </div>
                                {if summaries.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <p>{"No session summaries in claude-mem DB yet."}</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="brain-mem-body">
                                            <div class="brain-mem-list">
                                                {summaries.iter().map(|s| {
                                                    let s = s.clone();
                                                    let is_selected = {
                                                        let id = s.id;
                                                        move || selected_summary.get().map(|x| x.id) == Some(id)
                                                    };
                                                    let s_click = s.clone();
                                                    let headline: String = s.request.as_deref()
                                                        .filter(|t| !t.is_empty())
                                                        .unwrap_or("(no summary)")
                                                        .chars().take(70).collect();
                                                    view! {
                                                        <div
                                                            class=move || if is_selected() { "mem-list-item selected" } else { "mem-list-item" }
                                                            on:click=move |_| set_selected_summary.set(Some(s_click.clone()))
                                                        >
                                                            <span class="mem-icon">{"◈"}</span>
                                                            <span class="mem-meta">
                                                                <span class="mem-date">{format_date(&s.created_at)}</span>
                                                                <span class="mem-project">{basename(&s.project).to_string()}</span>
                                                            </span>
                                                            <span class="mem-headline">{headline}</span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>

                                            // Summary detail panel
                                            <div class="mem-detail-panel">
                                                {move || selected_summary.get().map(|s| view! {
                                                    <div class="mem-detail">
                                                        <div class="mem-detail-header">
                                                            <span class="mem-detail-project">{format!("◈  {}", basename(&s.project))}</span>
                                                            <span class="mem-detail-date">{format_date(&s.created_at)}</span>
                                                        </div>
                                                        {s.request.as_ref().map(|r| view! {
                                                            <div class="mem-detail-block">
                                                                <span class="mem-block-label">{"Asked"}</span>
                                                                <p class="mem-block-text">{r.clone()}</p>
                                                            </div>
                                                        })}
                                                        {s.completed.as_ref().map(|c| view! {
                                                            <div class="mem-detail-block mem-detail-block--done">
                                                                <span class="mem-block-label">{"Done"}</span>
                                                                <p class="mem-block-text">{c.clone()}</p>
                                                            </div>
                                                        })}
                                                        {s.next_steps.as_ref().map(|n| view! {
                                                            <div class="mem-detail-block mem-detail-block--next">
                                                                <span class="mem-block-label">{"Next"}</span>
                                                                <p class="mem-block-text">{n.clone()}</p>
                                                            </div>
                                                        })}
                                                        {s.files_edited.as_ref().filter(|f| !f.is_empty()).map(|f| view! {
                                                            <div class="mem-detail-block">
                                                                <span class="mem-block-label">{"Files"}</span>
                                                                <p class="mem-block-text mem-block-text--mono">{f.clone()}</p>
                                                            </div>
                                                        })}
                                                    </div>
                                                })}
                                                {move || selected_summary.get().is_none().then(|| view! {
                                                    <div class="detail-placeholder">
                                                        <p>{"← Select a session to view details"}</p>
                                                    </div>
                                                })}
                                            </div>
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    }
                    _ => view! { <div></div> }.into_any(),
                }
            })}
        </div>
    }
}
