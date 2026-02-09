//! Config page - displays Claude Code settings in 4-column layout (Global | Project | Local | Merged)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// API base URL constant
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080";
#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "";

/// Merged config structure matching backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedConfigResponse {
    pub global: Option<Value>,
    pub project: Option<Value>,
    pub local: Option<Value>,
    pub merged: Value,
}

/// Fetch config from API
async fn fetch_config() -> Result<MergedConfigResponse, String> {
    let url = format!("{}/api/config/merged", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch config: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let config: MergedConfigResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(config)
}

/// Format JSON with syntax highlighting (simple version with CSS classes)
fn format_json_highlighted(value: &Value) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|_| "Error formatting JSON".to_string())
}

/// Config column component
#[component]
fn ConfigColumn(
    /// Column title
    title: &'static str,
    /// Badge color class
    badge_class: &'static str,
    /// File path
    file_path: &'static str,
    /// JSON content (None = not found)
    content: Option<Value>,
) -> impl IntoView {
    view! {
        <div class="config-column">
            <div class="config-column-header">
                <h3 class="config-column-title">
                    <span class={format!("config-badge {}", badge_class)}>{title}</span>
                </h3>
                <code class="config-file-path">{file_path}</code>
            </div>
            <div class="config-column-body">
                {match content {
                    Some(json) => {
                        view! {
                            <pre class="config-json-highlighted">
                                {format_json_highlighted(&json)}
                            </pre>
                        }
                            .into_any()
                    }
                    None => {
                        view! {
                            <div class="config-empty">
                                <p class="config-empty-text">"No configuration found"</p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Config page component
#[component]
pub fn Config() -> impl IntoView {
    let config_resource = LocalResource::new(move || async move { fetch_config().await });

    view! {
        <div class="page config-page">
            <div class="page-header">
                <h1 class="page-title">"Configuration"</h1>
                <p class="page-description">
                    "Claude Code uses cascading configuration: Local → Project → Global. Merged shows final active configuration."
                </p>
            </div>

            <Suspense fallback=move || {
                view! { <p class="loading">"Loading configuration..."</p> }
            }>
                {move || {
                    config_resource
                        .get()
                        .map(|result| match result.as_ref() {
                            Ok(config) => {
                                view! {
                                    <div class="config-grid">
                                        <ConfigColumn
                                            title="Global"
                                            badge_class="badge-global"
                                            file_path="~/.claude/settings.json"
                                            content=config.global.clone()
                                        />
                                        <ConfigColumn
                                            title="Project"
                                            badge_class="badge-project"
                                            file_path=".claude/settings.json"
                                            content=config.project.clone()
                                        />
                                        <ConfigColumn
                                            title="Local"
                                            badge_class="badge-local"
                                            file_path=".claude/settings.local.json"
                                            content=config.local.clone()
                                        />
                                        <ConfigColumn
                                            title="Merged"
                                            badge_class="badge-merged"
                                            file_path="(Active Configuration)"
                                            content=Some(config.merged.clone())
                                        />
                                    </div>

                                    <div class="config-help">
                                        <h3>"Priority Order"</h3>
                                        <p>
                                            "Settings are merged with priority: "
                                            <strong>"Local"</strong>
                                            " > "
                                            <strong>"Project"</strong>
                                            " > "
                                            <strong>"Global"</strong>
                                            ". The "
                                            <strong>"Merged"</strong>
                                            " column shows the final configuration after merging all levels."
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            }
                            Err(e) => {
                                let error_msg = e.clone();
                                view! {
                                    <div class="error-message">
                                        <p>
                                            <strong>"Error loading config: "</strong>
                                            {error_msg}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
