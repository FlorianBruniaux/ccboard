//! Sessions Explorer page component with server-side pagination

use crate::api::SessionData;
use crate::components::{SessionDetailModal, SessionTable, use_toast};
use crate::sse_hook::{SseEvent, use_sse};
use crate::utils::{export_as_csv, export_as_json};
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen::JsCast;

/// API response for paginated sessions
#[derive(Debug, Clone, Deserialize)]
struct SessionsResponse {
    sessions: Vec<SessionData>,
    total: u64,
    page: usize,
    page_size: usize,
}

/// Fetch sessions from API with pagination and filters
async fn fetch_sessions(
    page: usize,
    search: String,
    project: Option<String>,
    model: Option<String>,
    date_filter: Option<String>,
) -> Result<SessionsResponse, String> {
    let mut url = format!("/api/sessions?page={}&limit=50", page);

    if !search.is_empty() {
        // Simple URL encoding: replace spaces with %20
        let encoded = search.replace(' ', "%20");
        url.push_str(&format!("&search={}", encoded));
    }
    if let Some(p) = project {
        let encoded = p.replace(' ', "%20");
        url.push_str(&format!("&project={}", encoded));
    }
    if let Some(m) = model {
        let encoded = m.replace(' ', "%20");
        url.push_str(&format!("&model={}", encoded));
    }
    if let Some(d) = date_filter {
        url.push_str(&format!("&since={}", d));
    }

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch sessions: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let data: SessionsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(data)
}

/// Sessions Explorer page
#[component]
pub fn Sessions() -> impl IntoView {
    // Filter state
    let (search, set_search) = signal(String::new());
    let (project_filter, set_project_filter) = signal(None::<String>);
    let (model_filter, set_model_filter) = signal(None::<String>);
    let (date_filter, set_date_filter) = signal(None::<String>);
    let (current_page, set_current_page) = signal(0usize);

    // Quick filters state (client-side filtering)
    let (cost_filter, set_cost_filter) = signal(None::<f64>); // Min cost threshold
    let (tokens_filter, set_tokens_filter) = signal(None::<u64>); // Min tokens threshold

    // Search triggers immediate refetch (debouncing could be added later with gloo_timers)
    let search_debounced = Signal::derive(move || search.get());

    // Fetch sessions data
    let sessions_resource = LocalResource::new(move || {
        let page = current_page.get();
        let search = search_debounced.get();
        let project = project_filter.get();
        let model = model_filter.get();
        let date = date_filter.get();

        async move { fetch_sessions(page, search, project, model, date).await }
    });

    // Modal state
    let (modal_session, set_modal_session) = signal(None::<SessionData>);

    // Toast notifications
    let toast = use_toast();

    // SSE setup for live updates
    let sse_event = use_sse();

    Effect::new(move |_| {
        if let Some(event) = sse_event.get() {
            match event {
                SseEvent::SessionCreated { id } => {
                    sessions_resource.refetch();
                    toast.success(format!("New session: {}", id));
                }
                SseEvent::SessionUpdated { id } => {
                    sessions_resource.refetch();
                    toast.info(format!("Session updated: {}", id));
                }
                SseEvent::StatsUpdated => {
                    sessions_resource.refetch();
                }
                _ => {}
            }
        }
    });

    // Handle Escape key to close modal
    leptos::leptos_dom::helpers::window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            set_modal_session.set(None);
        }
    });

    // Filter change handlers (reset to page 0)
    let on_project_change = move |p: Option<String>| {
        set_project_filter.set(p);
        set_current_page.set(0);
    };

    let on_model_change = move |m: Option<String>| {
        set_model_filter.set(m);
        set_current_page.set(0);
    };

    let on_date_change = move |d: Option<String>| {
        set_date_filter.set(d);
        set_current_page.set(0);
    };

    view! {
        <div class="page sessions-page">
            <div class="page-header">
                <h2>"Sessions Explorer"</h2>
                <div class="page-actions">
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(response) = sessions_resource.get() {
                                if let Ok(response) = response.as_ref() {
                                let sessions = response.sessions.clone();
                                let headers = vec![
                                    "Date".to_string(),
                                    "Project".to_string(),
                                    "Model".to_string(),
                                    "Messages".to_string(),
                                    "Tokens".to_string(),
                                    "Cost".to_string(),
                                ];
                                let rows: Vec<Vec<String>> = sessions
                                    .iter()
                                    .map(|s| {
                                        vec![
                                            s.date.clone().unwrap_or_default(),
                                            s.project.clone(),
                                            s.model.clone(),
                                            s.messages.to_string(),
                                            s.tokens.to_string(),
                                            format!("${:.4}", s.cost),
                                        ]
                                    })
                                    .collect();
                                export_as_csv(headers, rows, "ccboard-sessions");
                                }
                            }
                        }
                    >
                        "📥 Export CSV"
                    </button>
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(response) = sessions_resource.get() {
                                if let Ok(response) = response.as_ref() {
                                    export_as_json(&response.sessions, "ccboard-sessions");
                                }
                            }
                        }
                    >
                        "📥 Export JSON"
                    </button>
                </div>
            </div>

            <div class="page-content">
                // Quick filters (above search)
                <div class="quick-filters">
                    <span class="quick-filters-label">"Quick filters:"</span>
                    <button
                        class="quick-filter-btn"
                        class:active=move || cost_filter.get() == Some(5.0)
                        on:click=move |_| {
                            if cost_filter.get() == Some(5.0) {
                                set_cost_filter.set(None);
                            } else {
                                set_cost_filter.set(Some(5.0));
                                set_tokens_filter.set(None); // Clear other filter
                            }
                            set_current_page.set(0);
                        }
                    >
                        "💰 High Cost >$5"
                    </button>
                    <button
                        class="quick-filter-btn"
                        class:active=move || tokens_filter.get() == Some(10_000_000)
                        on:click=move |_| {
                            if tokens_filter.get() == Some(10_000_000) {
                                set_tokens_filter.set(None);
                            } else {
                                set_tokens_filter.set(Some(10_000_000));
                                set_cost_filter.set(None); // Clear other filter
                            }
                            set_current_page.set(0);
                        }
                    >
                        "🔥 High Tokens >10M"
                    </button>
                    <button
                        class="quick-filter-btn"
                        class:active=move || date_filter.get() == Some("7d".to_string())
                        on:click=move |_| {
                            if date_filter.get() == Some("7d".to_string()) {
                                set_date_filter.set(None);
                            } else {
                                set_date_filter.set(Some("7d".to_string()));
                            }
                            set_current_page.set(0);
                        }
                    >
                        "📅 Last 7 Days"
                    </button>
                    {move || {
                        // Show clear filters if any quick filter active
                        if cost_filter.get().is_some() || tokens_filter.get().is_some() || date_filter.get().is_some() {
                            view! {
                                <button
                                    class="quick-filter-btn clear-btn"
                                    on:click=move |_| {
                                        set_cost_filter.set(None);
                                        set_tokens_filter.set(None);
                                        set_date_filter.set(None);
                                        set_current_page.set(0);
                                    }
                                >
                                    "✕ Clear Filters"
                                </button>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>

                // Search bar (simple input)
                <div class="search-bar">
                    <input
                        type="text"
                        placeholder="Search sessions by ID, project, or preview..."
                        prop:value=move || search.get()
                        on:input=move |e| {
                            if let Some(target) = e.target() {
                                if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
                                    set_search.set(input.value());
                                }
                            }
                        }
                    />
                </div>

                // Filters
                <div class="filters-bar">
                    <div class="filter-group">
                        <label>"Project:"</label>
                        <select
                            on:change=move |e| {
                                if let Some(target) = e.target() {
                                    if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
                                        let value = select.value();
                                        on_project_change(if value.is_empty() { None } else { Some(value) });
                                    }
                                }
                            }
                        >
                            <option value="">"All Projects"</option>
                            // Note: We could fetch available projects from a separate endpoint
                            // For now, users can type in search
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Model:"</label>
                        <select
                            on:change=move |e| {
                                if let Some(target) = e.target() {
                                    if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
                                        let value = select.value();
                                        on_model_change(if value.is_empty() { None } else { Some(value) });
                                    }
                                }
                            }
                        >
                            <option value="">"All Models"</option>
                            <option value="sonnet">"Sonnet"</option>
                            <option value="opus">"Opus"</option>
                            <option value="haiku">"Haiku"</option>
                        </select>
                    </div>

                    <div class="filter-group">
                        <label>"Date Range:"</label>
                        <select
                            on:change=move |e| {
                                if let Some(target) = e.target() {
                                    if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
                                        let value = select.value();
                                        on_date_change(if value.is_empty() { None } else { Some(value) });
                                    }
                                }
                            }
                        >
                            <option value="">"All Time"</option>
                            <option value="7d">"Last 7 days"</option>
                            <option value="30d">"Last 30 days"</option>
                            <option value="90d">"Last 90 days"</option>
                        </select>
                    </div>
                </div>

                // Sessions table with loading state
                <Suspense fallback=move || view! { <div class="loading">"Loading sessions..."</div> }>
                    {move || {
                        sessions_resource.get().map(|result| match result.as_ref() {
                            Ok(response) => {
                                // Apply client-side filters (cost, tokens)
                                let mut sessions = response.sessions.clone();

                                // Filter by cost if set
                                if let Some(min_cost) = cost_filter.get() {
                                    sessions.retain(|s| s.cost >= min_cost);
                                }

                                // Filter by tokens if set
                                if let Some(min_tokens) = tokens_filter.get() {
                                    sessions.retain(|s| s.tokens >= min_tokens);
                                }

                                let total = response.total;
                                let page = response.page;
                                let page_size = response.page_size;
                                let total_pages = (total as f64 / page_size as f64).ceil() as usize;

                                // Store count AFTER filtering
                                let sessions_count = sessions.len();

                                // Create signals for SessionTable
                                let sessions_signal = Signal::derive(move || Some(sessions.clone()));

                                view! {
                                    <div class="sessions-container">
                                        <div class="sessions-stats">
                                            <span>{format!("Showing {} sessions (page {} of {})", sessions_count, page + 1, total_pages.max(1))}</span>
                                            <span>{format!("Total: {}", total)}</span>
                                        </div>

                                        <SessionTable
                                            sessions=sessions_signal
                                            on_row_click=set_modal_session
                                        />

                                        // Pagination controls
                                        <div class="pagination">
                                            <button
                                                class="pagination-button"
                                                disabled=move || current_page.get() == 0
                                                on:click=move |_| set_current_page.update(|p| *p = p.saturating_sub(1))
                                            >
                                                "← Previous"
                                            </button>
                                            <span class="pagination-info">
                                                {format!("Page {} of {}", page + 1, total_pages.max(1))}
                                            </span>
                                            <button
                                                class="pagination-button"
                                                disabled=move || {
                                                    let page = current_page.get();
                                                    let total_pages = sessions_resource.get()
                                                        .and_then(|r| r.as_ref().ok().cloned())
                                                        .map(|resp| (resp.total as f64 / resp.page_size as f64).ceil() as usize)
                                                        .unwrap_or(1);
                                                    page >= total_pages.saturating_sub(1)
                                                }
                                                on:click=move |_| set_current_page.update(|p| *p += 1)
                                            >
                                                "Next →"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="error-message">
                                    <p>"Failed to load sessions: " {e.clone()}</p>
                                </div>
                            }.into_any()
                        })
                    }}
                </Suspense>
            </div>

            // Session detail modal
            {move || {
                modal_session.get().map(|session| {
                    view! {
                        <SessionDetailModal
                            session=session
                            on_close=move || set_modal_session.set(None)
                        />
                    }
                })
            }}
        </div>
    }
}
