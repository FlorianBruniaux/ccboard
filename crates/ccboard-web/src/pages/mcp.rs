//! MCP page - displays MCP servers with split view (list + detail)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API base URL constant
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080";
#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "";

/// MCP server info structure matching backend API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub server_type: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub has_env: bool,
}

/// MCP servers list response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersResponse {
    pub servers: Vec<McpServerInfo>,
    pub total: usize,
}

/// Fetch MCP servers from API
async fn fetch_mcp_servers() -> Result<McpServersResponse, String> {
    let url = format!("{}/api/mcp", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch MCP servers: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let servers_response: McpServersResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(servers_response)
}

/// MCP server list item component
#[component]
fn McpServerListItem(
    server: McpServerInfo,
    selected: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = if selected {
        "mcp-list-item mcp-list-item--selected"
    } else {
        "mcp-list-item"
    };

    let status_class = "mcp-status-badge mcp-status-badge--up";

    view! {
        <div class=class on:click=move |_| on_click()>
            <div class="mcp-list-item__header">
                <div class="mcp-list-item__name">{server.name.clone()}</div>
                <span class=status_class>"●"</span>
            </div>
            <div class="mcp-list-item__type">{server.server_type.clone()}</div>
        </div>
    }
}

/// MCP server detail component
#[component]
fn McpServerDetail(server: McpServerInfo) -> impl IntoView {
    view! {
        <div class="mcp-detail">
            <div class="mcp-detail__header">
                <h2 class="mcp-detail__name">{server.name.clone()}</h2>
                <span class="mcp-detail__badge mcp-detail__badge--up">"Active"</span>
            </div>

            <div class="mcp-detail__section">
                <h3 class="mcp-detail__section-title">"Type"</h3>
                <code class="mcp-detail__type">{server.server_type.clone()}</code>
            </div>

            <div class="mcp-detail__section">
                <h3 class="mcp-detail__section-title">"Command"</h3>
                <code class="mcp-detail__command">{server.command.clone()}</code>
            </div>

            {(!server.args.is_empty()).then(|| view! {
                <div class="mcp-detail__section">
                    <h3 class="mcp-detail__section-title">"Arguments"</h3>
                    <div class="mcp-detail__args-list">
                        {server.args.iter().map(|arg| {
                            view! {
                                <code class="mcp-detail__arg">{arg.clone()}</code>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            })}

            {server.url.as_ref().map(|url| view! {
                <div class="mcp-detail__section">
                    <h3 class="mcp-detail__section-title">"URL"</h3>
                    <code class="mcp-detail__url">{url.clone()}</code>
                </div>
            })}

            {server.has_env.then(|| view! {
                <div class="mcp-detail__section">
                    <h3 class="mcp-detail__section-title">"Environment Variables"</h3>
                    <div class="mcp-detail__env-list">
                        {server.env.iter().map(|(key, value)| {
                            view! {
                                <div class="mcp-detail__env-item">
                                    <span class="mcp-detail__env-key">{key.clone()}</span>
                                    <span class="mcp-detail__env-value">{value.clone()}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            })}
        </div>
    }
}

/// MCP page component
#[component]
pub fn Mcp() -> impl IntoView {
    let mcp_resource = LocalResource::new(move || async move { fetch_mcp_servers().await });
    let selected_server_index = RwSignal::new(0usize);

    view! {
        <div class="page mcp-page">
            <div class="page-header">
                <h1 class="page-title">"MCP Servers"</h1>
                <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                    {move || {
                        mcp_resource
                            .get()
                            .map(|result| {
                                match *result {
                                    Ok(ref response) => {
                                        view! {
                                            <span class="page-subtitle">
                                                {format!("{} server(s) configured", response.total)}
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

            <Suspense fallback=|| view! { <div class="loading">"Loading MCP servers..."</div> }>
                {move || {
                    mcp_resource
                        .get()
                        .map(|result| {
                            match result.as_ref() {
                                Ok(response) => {
                                    if response.servers.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p>"No MCP servers configured"</p>
                                                <p class="empty-state-hint">"Add servers to ~/.claude/claude_desktop_config.json"</p>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        let servers = RwSignal::new(response.servers.clone());
                                        view! {
                                            <div class="mcp-content">
                                                <div class="mcp-servers-list">
                                                    {move || {
                                                        let selected_idx = selected_server_index.get();
                                                        servers
                                                            .get()
                                                            .iter()
                                                            .enumerate()
                                                            .map(|(idx, server): (usize, &McpServerInfo)| {
                                                                let server_clone = server.clone();
                                                                let is_selected = selected_idx == idx;
                                                                view! {
                                                                    <McpServerListItem
                                                                        server=server_clone
                                                                        selected=is_selected
                                                                        on_click=move || selected_server_index.set(idx)
                                                                    />
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()
                                                    }}
                                                </div>
                                                <div class="mcp-server-detail">
                                                    {move || {
                                                        let idx = selected_server_index.get();
                                                        servers
                                                            .with(|servers_vec: &Vec<McpServerInfo>| {
                                                                if let Some(server) = servers_vec.get(idx) {
                                                                    let s = server.clone();
                                                                    view! { <McpServerDetail server=s /> }.into_any()
                                                                } else {
                                                                    view! { <div>"No server selected"</div> }.into_any()
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
                                            <p>"Error loading MCP servers: " {e.to_string()}</p>
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
