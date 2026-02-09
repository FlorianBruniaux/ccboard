//! Config page - displays Claude Code settings in 4-column layout (Global | Project | Local | Merged)

use leptos::prelude::*;
use leptos::ev;
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
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "Error formatting JSON".to_string())
}

/// Highlight search matches in JSON text
fn highlight_json_matches(json_text: &str, search_query: &str) -> String {
    if search_query.is_empty() {
        return json_text.to_string();
    }

    let lower_query = search_query.to_lowercase();
    let mut result = String::with_capacity(json_text.len() + 100);
    let mut last_end = 0;

    for (idx, _) in json_text.match_indices(&search_query) {
        result.push_str(&json_text[last_end..idx]);
        result.push_str(&format!(
            "<mark>{}</mark>",
            &json_text[idx..idx + search_query.len()]
        ));
        last_end = idx + search_query.len();
    }

    // Case-insensitive fallback if no exact matches
    if last_end == 0 {
        let lower_text = json_text.to_lowercase();
        for (idx, _) in lower_text.match_indices(&lower_query) {
            result.push_str(&json_text[last_end..idx]);
            result.push_str(&format!(
                "<mark>{}</mark>",
                &json_text[idx..idx + search_query.len()]
            ));
            last_end = idx + search_query.len();
        }
    }

    result.push_str(&json_text[last_end..]);
    result
}

/// Copy text to clipboard
fn copy_to_clipboard(text: &str) {
    use wasm_bindgen::JsValue;
    let window = web_sys::window().expect("window");
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();

    let text_js = JsValue::from_str(text);
    let _ = clipboard.write_text(&text_js.as_string().unwrap_or_default());
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
    /// Search query for highlighting
    #[prop(into)]
    search_query: Signal<String>,
    /// Click handler to open modal
    #[prop(optional)]
    on_expand: Option<Box<dyn Fn(String, String) + 'static>>,
) -> impl IntoView {
    let json_text = content.as_ref().map(|json| format_json_highlighted(json));
    let json_text_for_copy = json_text.clone();
    let json_text_for_modal = json_text.clone();
    let has_json = json_text.is_some();

    let copy_handler = move |_| {
        if let Some(ref text) = json_text_for_copy {
            copy_to_clipboard(text);
            // TODO: Show toast notification "Copied!"
        }
    };

    let title_str = title.to_string();
    let file_path_str = file_path.to_string();
    let expand_handler = move |_ev: ev::MouseEvent| {
        if let (Some(handler), Some(text)) = (&on_expand, &json_text_for_modal) {
            handler(format!("{} - {}", title_str, file_path_str), text.clone());
        }
    };

    view! {
        <div class="config-column">
            <div class="config-column-header">
                <div class="config-column-title-row">
                    <h3 class="config-column-title">
                        <span class={format!("config-badge {}", badge_class)}>{title}</span>
                    </h3>
                    {has_json.then(|| {
                        view! {
                            <button class="btn-icon btn-expand" on:click=expand_handler title="View Fullscreen">
                                "📖"
                            </button>
                            <button class="btn-icon btn-copy" on:click=copy_handler title="Copy JSON">
                                "📋"
                            </button>
                        }
                    })}
                </div>
                <code class="config-file-path">{file_path}</code>
            </div>
            <div class="config-column-body">
                {match json_text {
                    Some(text) => {
                        let highlighted = move || {
                            let query = search_query.get();
                            if query.is_empty() {
                                text.clone()
                            } else {
                                highlight_json_matches(&text, &query)
                            }
                        };

                        view! {
                            <pre class="config-json-highlighted" inner_html=highlighted></pre>
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
    let (search_query, set_search_query) = signal(String::new());
    let (diff_mode, set_diff_mode) = signal(false);

    // Modal state
    let (modal_open, set_modal_open) = signal(false);
    let (modal_title, set_modal_title) = signal(String::new());
    let (modal_content, set_modal_content) = signal(String::new());

    let search_input_handler = move |ev| {
        let value = event_target_value(&ev);
        set_search_query.set(value);
    };

    let toggle_diff_mode = move |_| {
        set_diff_mode.update(|mode| *mode = !*mode);
    };

    // Expand handler to open modal
    let expand_callback = Box::new(move |title: String, content: String| {
        set_modal_title.set(title);
        set_modal_content.set(content);
        set_modal_open.set(true);
    });

    // Close modal handler
    let close_modal = move |_ev: ev::MouseEvent| {
        set_modal_open.set(false);
    };

    view! {
        <div class="page config-page">
            <div class="page-header">
                <h1 class="page-title">"Configuration"</h1>
                <p class="page-description">
                    "Claude Code uses cascading configuration: Local → Project → Global. Merged shows final active configuration."
                </p>
            </div>

            // Search bar controls
            <div class="config-controls">
                <div class="config-search-bar">
                    <input
                        type="text"
                        class="config-search-input"
                        placeholder="Search in config... (e.g., 'hooks', 'model', 'api_key')"
                        on:input=search_input_handler
                        value=move || search_query.get()
                    />
                    <div class="config-search-results">
                        {move || {
                            let query = search_query.get();
                            if !query.is_empty() {
                                format!("Searching for: '{}'", query)
                            } else {
                                String::new()
                            }
                        }}
                    </div>
                </div>
                <button
                    class=move || {
                        if diff_mode.get() { "btn btn-primary btn-diff-mode active" } else { "btn btn-secondary btn-diff-mode" }
                    }
                    on:click=toggle_diff_mode
                    title="Show only settings that override defaults"
                >
                    {move || if diff_mode.get() { "🔍 Showing Overrides" } else { "🔍 Show Overrides Only" }}
                </button>
            </div>

            <Suspense fallback=move || {
                view! { <p class="loading">"Loading configuration..."</p> }
            }>
                {move || {
                    config_resource
                        .get()
                        .map(|result| match result.as_ref() {
                            Ok(config) => {
                                // Clone callback for each column
                                let cb1 = expand_callback.clone();
                                let cb2 = expand_callback.clone();
                                let cb3 = expand_callback.clone();
                                let cb4 = expand_callback.clone();

                                view! {
                                    <div class="config-grid">
                                        <ConfigColumn
                                            title="Global"
                                            badge_class="badge-global"
                                            file_path="~/.claude/settings.json"
                                            content=config.global.clone()
                                            search_query=search_query
                                            on_expand=cb1
                                        />
                                        <ConfigColumn
                                            title="Project"
                                            badge_class="badge-project"
                                            file_path=".claude/settings.json"
                                            content=config.project.clone()
                                            search_query=search_query
                                            on_expand=cb2
                                        />
                                        <ConfigColumn
                                            title="Local"
                                            badge_class="badge-local"
                                            file_path=".claude/settings.local.json"
                                            content=config.local.clone()
                                            search_query=search_query
                                            on_expand=cb3
                                        />
                                        <ConfigColumn
                                            title="Merged"
                                            badge_class="badge-merged"
                                            file_path="(Active Configuration)"
                                            content=Some(config.merged.clone())
                                            search_query=search_query
                                            on_expand=cb4
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

            // Modal for fullscreen config view
            {move || {
                if modal_open.get() {
                    Some(view! {
                        <div class="config-modal-backdrop" on:click=close_modal>
                            <div class="config-modal-content" on:click=|e: ev::MouseEvent| e.stop_propagation()>
                                <div class="config-modal-header">
                                    <h2 class="config-modal-title">{move || modal_title.get()}</h2>
                                    <button class="btn-icon btn-close" on:click=close_modal title="Close (Esc)">
                                        "✕"
                                    </button>
                                </div>
                                <div class="config-modal-body">
                                    <pre class="config-json-highlighted">{move || modal_content.get()}</pre>
                                </div>
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}
