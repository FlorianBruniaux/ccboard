//! Plugins page - displays plugin usage analytics (Skills, MCP, Agents, Commands, Native Tools)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// API base URL constant (empty = relative URL, same origin)
const API_BASE_URL: &str = "";

/// Plugin classification by origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    Skill,
    Command,
    Agent,
    McpServer,
    NativeTool,
}

impl PluginType {
    /// Icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Skill => "🎓",
            Self::Command => "⚡",
            Self::Agent => "🤖",
            Self::McpServer => "🔌",
            Self::NativeTool => "🛠️",
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Skill => "Skill",
            Self::Command => "Command",
            Self::Agent => "Agent",
            Self::McpServer => "MCP Server",
            Self::NativeTool => "Native Tool",
        }
    }
}

/// Usage statistics for a single plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUsage {
    pub name: String,
    pub plugin_type: PluginType,
    pub total_invocations: usize,
    pub sessions_used: Vec<String>,
    pub total_cost: f64,
    pub avg_tokens_per_invocation: u64,
    pub first_seen: String, // ISO8601 timestamp
    pub last_seen: String,  // ISO8601 timestamp
}

/// Complete plugin analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAnalytics {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub dead_plugins: Vec<String>,
    pub plugins: Vec<PluginUsage>,
    pub top_by_usage: Vec<PluginUsage>,
    pub top_by_cost: Vec<PluginUsage>,
    pub computed_at: String, // ISO8601 timestamp
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsResponse {
    pub analytics: PluginAnalytics,
    pub generated_at: String,
}

/// Fetch plugins from API
async fn fetch_plugins() -> Result<PluginsResponse, String> {
    let url = format!("{}/api/plugins", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch plugins: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let plugins_response: PluginsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(plugins_response)
}

/// Stats card component
#[component]
fn StatsCard(label: &'static str, value: String, color: &'static str) -> impl IntoView {
    let card_class = format!("stats-card stats-card--{}", color);

    view! {
        <div class=card_class>
            <div class="stats-card__label">{label}</div>
            <div class="stats-card__value">{value}</div>
        </div>
    }
}

/// Plugin list item component
#[component]
fn PluginListItem(plugin: PluginUsage, rank: usize) -> impl IntoView {
    let icon = plugin.plugin_type.icon();
    let type_label = plugin.plugin_type.label();

    view! {
        <div class="plugin-list-item">
            <div class="plugin-list-item__rank">{rank}"."</div>
            <div class="plugin-list-item__icon">{icon}</div>
            <div class="plugin-list-item__content">
                <div class="plugin-list-item__name">{plugin.name.clone()}</div>
                <div class="plugin-list-item__type">{type_label}</div>
            </div>
            <div class="plugin-list-item__stats">
                <span class="plugin-list-item__invocations">{plugin.total_invocations}" uses"</span>
            </div>
        </div>
    }
}

/// Plugin cost item component
#[component]
fn PluginCostItem(plugin: PluginUsage, rank: usize) -> impl IntoView {
    view! {
        <div class="plugin-cost-item">
            <div class="plugin-cost-item__rank">{rank}"."</div>
            <div class="plugin-cost-item__name">{plugin.name.clone()}</div>
            <div class="plugin-cost-item__cost">"$"{format!("{:.2}", plugin.total_cost)}</div>
        </div>
    }
}

/// Dead code item component
#[component]
fn DeadCodeItem(name: String) -> impl IntoView {
    view! {
        <div class="dead-code-item">
            <div class="dead-code-item__icon">"⚠️"</div>
            <div class="dead-code-item__name">{name}" (0 uses)"</div>
        </div>
    }
}

/// Plugins page component
#[component]
pub fn PluginsPage() -> impl IntoView {
    let plugin_data = LocalResource::new(move || async move { fetch_plugins().await });

    view! {
        <div class="plugins-page">
            <div class="plugins-page__header">
                <h1 class="plugins-page__title">"🎁 Plugin Analytics"</h1>
                <p class="plugins-page__subtitle">
                    "Analyze plugin usage across Skills, MCP Servers, Agents, and Commands"
                </p>
            </div>

            <Suspense fallback=move || view! { <div class="loading-spinner">"Loading plugin analytics..."</div> }>
                {move || Suspend::new(async move {
                    match plugin_data.await {
                        Ok(response) => {
                            let analytics = response.analytics;
                            let active_pct = if analytics.total_plugins > 0 {
                                (analytics.active_plugins as f64 / analytics.total_plugins as f64) * 100.0
                            } else {
                                0.0
                            };

                            view! {
                                <div class="plugins-page__content">
                                    // Stats cards
                                    <div class="stats-grid">
                                        <StatsCard
                                            label="Total Plugins"
                                            value=analytics.total_plugins.to_string()
                                            color="blue"
                                        />
                                        <StatsCard
                                            label="Active"
                                            value=format!("{} ({:.0}%)", analytics.active_plugins, active_pct)
                                            color="green"
                                        />
                                        <StatsCard
                                            label="Dead Code"
                                            value=analytics.dead_plugins.len().to_string()
                                            color="red"
                                        />
                                    </div>

                                    // Three-column layout
                                    <div class="plugins-columns">
                                        // Top 10 Most Used
                                        <div class="plugins-column">
                                            <div class="plugins-column__header">
                                                <h2 class="plugins-column__title">"Top 10 Most Used"</h2>
                                            </div>
                                            <div class="plugins-column__content">
                                                {if analytics.top_by_usage.is_empty() {
                                                    view! {
                                                        <div class="empty-state">
                                                            <p>"No plugin usage data available"</p>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    analytics.top_by_usage.into_iter().enumerate().map(|(i, plugin)| {
                                                        view! {
                                                            <PluginListItem plugin=plugin rank=i+1 />
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </div>
                                        </div>

                                        // Top 10 By Cost
                                        <div class="plugins-column">
                                            <div class="plugins-column__header">
                                                <h2 class="plugins-column__title">"Top 10 By Cost"</h2>
                                            </div>
                                            <div class="plugins-column__content">
                                                {if analytics.top_by_cost.is_empty() {
                                                    view! {
                                                        <div class="empty-state">
                                                            <p>"No cost data available"</p>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    analytics.top_by_cost.into_iter().enumerate().map(|(i, plugin)| {
                                                        view! {
                                                            <PluginCostItem plugin=plugin rank=i+1 />
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </div>
                                        </div>

                                        // Dead Code
                                        <div class="plugins-column">
                                            <div class="plugins-column__header">
                                                <h2 class="plugins-column__title">"Dead Code (Never Used)"</h2>
                                            </div>
                                            <div class="plugins-column__content">
                                                {if analytics.dead_plugins.is_empty() {
                                                    view! {
                                                        <div class="empty-state empty-state--success">
                                                            <div class="empty-state__icon">"🎉"</div>
                                                            <p>"No dead code detected!"</p>
                                                            <p class="empty-state__subtitle">"All plugins are being used."</p>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    analytics.dead_plugins.into_iter().map(|name| {
                                                        view! {
                                                            <DeadCodeItem name=name />
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="error-state">
                                    <div class="error-state__icon">"⚠️"</div>
                                    <h2 class="error-state__title">"Error loading plugins"</h2>
                                    <p class="error-state__message">{e}</p>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
