//! Agents/Commands/Skills page - displays custom agents, commands, and skills with 3 tabs

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// API base URL constant (empty = relative URL, same origin)
const API_BASE_URL: &str = "";

/// Item info structure (Agent/Command/Skill)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    pub name: String,
    pub frontmatter: Value,
    pub body: String,
    pub path: String,
}

/// Items list response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsResponse {
    pub items: Vec<ItemInfo>,
    pub total: usize,
}

/// Fetch items from API
async fn fetch_items(endpoint: &str) -> Result<ItemsResponse, String> {
    let url = format!("{}/api/{}", API_BASE_URL, endpoint);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {}: {}", endpoint, e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let items_response: ItemsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(items_response)
}

/// Item list item component
#[component]
fn ItemListItem(item: ItemInfo, selected: bool, on_click: impl Fn() + 'static) -> impl IntoView {
    let class = if selected {
        "agent-list-item agent-list-item--selected"
    } else {
        "agent-list-item"
    };

    // Extract description from frontmatter
    let description = item
        .frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    view! {
        <div class=class on:click=move |_| on_click()>
            <div class="agent-list-item__name">{item.name.clone()}</div>
            {(!description.is_empty()).then(|| view! {
                <div class="agent-list-item__desc">{description}</div>
            })}
        </div>
    }
}

/// Item detail component
#[component]
fn ItemDetail(item: ItemInfo) -> impl IntoView {
    let description = item
        .frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let category = item
        .frontmatter
        .get("category")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tools = item
        .frontmatter
        .get("tools")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    view! {
        <div class="agent-detail">
            <div class="agent-detail__header">
                <h2 class="agent-detail__name">{item.name.clone()}</h2>
                {category.map(|cat| view! {
                    <span class="agent-detail__badge">{cat}</span>
                })}
            </div>

            {description.map(|desc| view! {
                <div class="agent-detail__section">
                    <p class="agent-detail__description">{desc}</p>
                </div>
            })}

            {tools.map(|t| view! {
                <div class="agent-detail__section">
                    <h3 class="agent-detail__section-title">"Tools"</h3>
                    <code class="agent-detail__tools">{t}</code>
                </div>
            })}

            <div class="agent-detail__section">
                <h3 class="agent-detail__section-title">"Content"</h3>
                <div class="agent-detail__body">
                    {item.body}
                </div>
            </div>

            <div class="agent-detail__section">
                <h3 class="agent-detail__section-title">"Path"</h3>
                <code class="agent-detail__path">{item.path}</code>
            </div>
        </div>
    }
}

/// Items list component (generic for agents/commands/skills)
#[component]
fn ItemsList(items: RwSignal<Vec<ItemInfo>>, selected_index: RwSignal<usize>) -> impl IntoView {
    view! {
        <div class="agents-content">
            <div class="agents-list">
                {move || {
                    let selected_idx = selected_index.get();
                    items
                        .get()
                        .iter()
                        .enumerate()
                        .map(|(idx, item): (usize, &ItemInfo)| {
                            let item_clone = item.clone();
                            let is_selected = selected_idx == idx;
                            view! {
                                <ItemListItem
                                    item=item_clone
                                    selected=is_selected
                                    on_click=move || selected_index.set(idx)
                                />
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
            <div class="agents-detail">
                {move || {
                    let idx = selected_index.get();
                    items
                        .with(|items_vec: &Vec<ItemInfo>| {
                            if let Some(item) = items_vec.get(idx) {
                                let i = item.clone();
                                view! { <ItemDetail item=i /> }.into_any()
                            } else {
                                view! { <div class="empty-state">"No item selected"</div> }.into_any()
                            }
                        })
                }}
            </div>
        </div>
    }
}

/// Agents page component with 3 tabs
#[component]
pub fn Agents() -> impl IntoView {
    let active_tab = RwSignal::new("agents".to_string());
    let selected_index = RwSignal::new(0usize);

    // Resources for each tab
    let agents_resource = LocalResource::new(move || async move { fetch_items("agents").await });
    let commands_resource =
        LocalResource::new(move || async move { fetch_items("commands").await });
    let skills_resource = LocalResource::new(move || async move { fetch_items("skills").await });

    view! {
        <div class="page agents-page">
            <div class="page-header">
                <h1 class="page-title">"Agents & Capabilities"</h1>
            </div>

            <div class="agents-tabs">
                <button
                    class=move || if active_tab.get() == "agents" { "agents-tab agents-tab--active" } else { "agents-tab" }
                    on:click=move |_| { active_tab.set("agents".to_string()); selected_index.set(0); }
                >
                    "Agents"
                    <Suspense fallback=|| "">
                        {move || {
                            agents_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            view! { <span class="agents-tab__count">{response.total}</span> }.into_any()
                                        }
                                        Err(_) => view! { <span></span> }.into_any(),
                                    }
                                })
                        }}
                    </Suspense>
                </button>
                <button
                    class=move || if active_tab.get() == "commands" { "agents-tab agents-tab--active" } else { "agents-tab" }
                    on:click=move |_| { active_tab.set("commands".to_string()); selected_index.set(0); }
                >
                    "Commands"
                    <Suspense fallback=|| "">
                        {move || {
                            commands_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            view! { <span class="agents-tab__count">{response.total}</span> }.into_any()
                                        }
                                        Err(_) => view! { <span></span> }.into_any(),
                                    }
                                })
                        }}
                    </Suspense>
                </button>
                <button
                    class=move || if active_tab.get() == "skills" { "agents-tab agents-tab--active" } else { "agents-tab" }
                    on:click=move |_| { active_tab.set("skills".to_string()); selected_index.set(0); }
                >
                    "Skills"
                    <Suspense fallback=|| "">
                        {move || {
                            skills_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            view! { <span class="agents-tab__count">{response.total}</span> }.into_any()
                                        }
                                        Err(_) => view! { <span></span> }.into_any(),
                                    }
                                })
                        }}
                    </Suspense>
                </button>
            </div>

            <Suspense fallback=|| view! { <div class="loading">"Loading..."</div> }>
                {move || {
                    let tab = active_tab.get();
                    match tab.as_str() {
                        "agents" => {
                            agents_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            if response.items.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No agents found"</p>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                let items = RwSignal::new(response.items.clone());
                                                view! { <ItemsList items selected_index /> }.into_any()
                                            }
                                        }
                                        Err(e) => {
                                            view! {
                                                <div class="error-state">
                                                    <p>"Error: " {e.to_string()}</p>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                })
                        }
                        "commands" => {
                            commands_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            if response.items.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No commands found"</p>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                let items = RwSignal::new(response.items.clone());
                                                view! { <ItemsList items selected_index /> }.into_any()
                                            }
                                        }
                                        Err(e) => {
                                            view! {
                                                <div class="error-state">
                                                    <p>"Error: " {e.to_string()}</p>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                })
                        }
                        "skills" => {
                            skills_resource
                                .get()
                                .map(|result| {
                                    match result.as_ref() {
                                        Ok(response) => {
                                            if response.items.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No skills found"</p>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                let items = RwSignal::new(response.items.clone());
                                                view! { <ItemsList items selected_index /> }.into_any()
                                            }
                                        }
                                        Err(e) => {
                                            view! {
                                                <div class="error-state">
                                                    <p>"Error: " {e.to_string()}</p>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                })
                        }
                        _ => None,
                    }
                }}
            </Suspense>
        </div>
    }
}
