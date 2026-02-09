//! Config page - displays merged Claude Code settings

use leptos::prelude::*;
use serde::Deserialize;
use serde_json::Value;

/// API base URL constant
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080";
#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "";

/// Merged config response from API
#[derive(Debug, Clone, Deserialize)]
struct ConfigResponse {
    #[serde(flatten)]
    settings: Value,
}

/// Fetch config from API
async fn fetch_config() -> Result<Value, String> {
    let url = format!("{}/api/config/merged", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch config: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let config: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(config)
}

/// Config page component
#[component]
pub fn Config() -> impl IntoView {
    let config_resource = LocalResource::new(move || async move { fetch_config().await });

    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"Configuration"</h1>
                <p class="page-description">
                    "Merged Claude Code settings from global, project, and local levels"
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
                                    <div class="config-container">
                                        <div class="config-card">
                                            <h2 class="config-section-title">
                                                "Merged Settings"
                                            </h2>
                                            <pre class="config-json">
                                                {serde_json::to_string_pretty(&config)
                                                    .unwrap_or_else(|_| "Error formatting JSON".to_string())}
                                            </pre>
                                        </div>

                                        <div class="config-info">
                                            <h3>"Priority Order"</h3>
                                            <ol>
                                                <li>
                                                    <strong>"Local"</strong>
                                                    " - "
                                                    <code>".claude/settings.local.json"</code>
                                                </li>
                                                <li>
                                                    <strong>"Project"</strong>
                                                    " - "
                                                    <code>".claude/settings.json"</code>
                                                </li>
                                                <li>
                                                    <strong>"Global"</strong>
                                                    " - "
                                                    <code>"~/.claude/settings.json"</code>
                                                </li>
                                            </ol>
                                        </div>
                                    </div>
                                }.into_any()
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
                                }.into_any()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
