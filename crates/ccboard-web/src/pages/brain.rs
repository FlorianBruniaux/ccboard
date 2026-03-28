//! Brain page — cross-session knowledge base from ~/.ccboard/insights.db

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

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn format_date(iso: &str) -> String {
    // Parse "2025-03-28T14:23:45Z" → "Mar 28 14:23"
    if iso.len() >= 16 {
        let date_part = &iso[5..10]; // "03-28"
        let time_part = &iso[11..16]; // "14:23"
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
    let (selected, set_selected) = signal::<Option<InsightData>>(None);

    let insights_res = LocalResource::new(move || {
        let f = filter.get();
        async move { fetch_insights(f).await }
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

            // Filter tabs
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
                                set_selected.set(None);
                            }
                        >
                            {if t != "all" { format!("{} {}", type_icon(t), &label) } else { label }}
                        </button>
                    }
                }).collect_view()}
            </div>

            // Main content
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
                                                    move || selected.get().map(|s| s.id) == Some(id)
                                                };
                                                let insight_click = insight.clone();
                                                view! {
                                                    <div
                                                        class=move || if is_selected() { "brain-list-item selected" } else { "brain-list-item" }
                                                        on:click=move |_| set_selected.set(Some(insight_click.clone()))
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

                                    // Detail panel
                                    <div class="brain-detail-panel">
                                        {move || selected.get().map(|s| view! {
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
                                        {move || selected.get().is_none().then(|| view! {
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
        </div>
    }
}
