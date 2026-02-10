//! Hooks page - displays Claude Code hooks with split view (list + detail)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// API base URL constant (empty = relative URL, same origin)
const API_BASE_URL: &str = "";

/// Hook info structure matching backend API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookInfo {
    pub name: String,
    pub event: String,
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub r#async: bool,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub matcher: Option<String>,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default)]
    pub script_content: Option<String>,
}

/// Hooks list response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksResponse {
    pub hooks: Vec<HookInfo>,
    pub total: usize,
}

/// Fetch hooks from API
async fn fetch_hooks() -> Result<HooksResponse, String> {
    let url = format!("{}/api/hooks", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch hooks: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let hooks_response: HooksResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(hooks_response)
}

/// Hook list item component
#[component]
fn HookListItem(hook: HookInfo, selected: bool, on_click: impl Fn() + 'static) -> impl IntoView {
    let class = if selected {
        "hook-list-item hook-list-item--selected"
    } else {
        "hook-list-item"
    };

    view! {
        <div class=class on:click=move |_| on_click()>
            <div class="hook-list-item__name">{hook.name.clone()}</div>
            <div class="hook-list-item__event">{hook.event.clone()}</div>
        </div>
    }
}

/// Hook detail component
#[component]
fn HookDetail(hook: HookInfo) -> impl IntoView {
    view! {
        <div class="hook-detail">
            <div class="hook-detail__header">
                <h2 class="hook-detail__name">{hook.name.clone()}</h2>
                <span class="hook-detail__badge">{hook.event.clone()}</span>
            </div>

            <div class="hook-detail__section">
                <h3 class="hook-detail__section-title">"Command"</h3>
                <code class="hook-detail__command">{hook.command.clone()}</code>
            </div>

            {hook.description.as_ref().map(|desc| view! {
                <div class="hook-detail__section">
                    <h3 class="hook-detail__section-title">"Description"</h3>
                    <p class="hook-detail__description">{desc.clone()}</p>
                </div>
            })}

            <div class="hook-detail__metadata">
                <div class="hook-detail__meta-item">
                    <span class="hook-detail__meta-label">"Async:"</span>
                    <span class="hook-detail__meta-value">{if hook.r#async { "Yes" } else { "No" }}</span>
                </div>
                {hook.timeout.map(|t| view! {
                    <div class="hook-detail__meta-item">
                        <span class="hook-detail__meta-label">"Timeout:"</span>
                        <span class="hook-detail__meta-value">{format!("{}s", t)}</span>
                    </div>
                })}
                {hook.cwd.as_ref().map(|cwd| view! {
                    <div class="hook-detail__meta-item">
                        <span class="hook-detail__meta-label">"Working Dir:"</span>
                        <span class="hook-detail__meta-value">{cwd.clone()}</span>
                    </div>
                })}
                {hook.matcher.as_ref().map(|matcher| view! {
                    <div class="hook-detail__meta-item">
                        <span class="hook-detail__meta-label">"Matcher:"</span>
                        <span class="hook-detail__meta-value">{matcher.clone()}</span>
                    </div>
                })}
            </div>

            {hook.script_content.as_ref().map(|script| view! {
                <div class="hook-detail__section">
                    <h3 class="hook-detail__section-title">"Script Content"</h3>
                    {hook.script_path.as_ref().map(|path| view! {
                        <code class="hook-detail__script-path">{path.clone()}</code>
                    })}
                    <pre class="hook-detail__script">{script.clone()}</pre>
                </div>
            })}
        </div>
    }
}

/// Hooks page component
#[component]
pub fn Hooks() -> impl IntoView {
    let hooks_resource = LocalResource::new(move || async move { fetch_hooks().await });
    let selected_hook_index = RwSignal::new(0usize);

    view! {
        <div class="page hooks-page">
            <div class="page-header">
                <h1 class="page-title">"Hooks"</h1>
                <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                    {move || {
                        hooks_resource
                            .get()
                            .map(|result| {
                                match *result {
                                    Ok(ref response) => {
                                        view! {
                                            <span class="page-subtitle">
                                                {format!("{} hook(s) configured", response.total)}
                                            </span>
                                        }
                                            .into_any()
                                    }
                                    Err(_) => view! { <span></span> }.into_any(),
                                }
                            })
                    }}
                </Suspense>
            </div>

            <Suspense fallback=|| view! { <div class="loading">"Loading hooks..."</div> }>
                {move || {
                    hooks_resource
                        .get()
                        .map(|result| {
                            match result.as_ref() {
                                Ok(response) => {
                                    if response.hooks.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p>"No hooks configured"</p>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        let hooks = RwSignal::new(response.hooks.clone());
                                        view! {
                                            <div class="hooks-content">
                                                <div class="hooks-list">
                                                    {move || {
                                                        let selected_idx = selected_hook_index.get();
                                                        hooks
                                                            .get()
                                                            .iter()
                                                            .enumerate()
                                                            .map(|(idx, hook): (usize, &HookInfo)| {
                                                                let hook_clone = hook.clone();
                                                                let is_selected = selected_idx == idx;
                                                                view! {
                                                                    <HookListItem
                                                                        hook=hook_clone
                                                                        selected=is_selected
                                                                        on_click=move || selected_hook_index.set(idx)
                                                                    />
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()
                                                    }}
                                                </div>
                                                <div class="hooks-detail">
                                                    {move || {
                                                        let idx = selected_hook_index.get();
                                                        hooks
                                                            .with(|hooks_vec: &Vec<HookInfo>| {
                                                                if let Some(hook) = hooks_vec.get(idx) {
                                                                    let h = hook.clone();
                                                                    view! { <HookDetail hook=h /> }.into_any()
                                                                } else {
                                                                    view! { <div>"No hook selected"</div> }.into_any()
                                                                }
                                                            })
                                                    }}
                                                </div>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                }
                                Err(e) => {
                                    view! {
                                        <div class="error-state">
                                            <p>"Error loading hooks: " {e.to_string()}</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
