//! Sessions Explorer page component

use crate::components::{SearchBar, SessionData, SessionDetailModal, SessionTable, use_toast};
use crate::sse_hook::{SseEvent, use_sse};
use crate::utils::export::{export_as_csv, export_as_json};
use chrono::{DateTime, Duration, Utc};
use leptos::prelude::*;
use serde::Deserialize;

/// API response for sessions
#[derive(Debug, Clone, Deserialize)]
struct SessionsResponse {
    sessions: Vec<SessionData>,
}

/// Fetch sessions from API
async fn fetch_sessions() -> Result<Vec<SessionData>, String> {
    let response = gloo_net::http::Request::get("/api/sessions")
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

    Ok(data.sessions)
}

/// Sessions Explorer page
#[component]
pub fn Sessions() -> impl IntoView {
    // Version signal to trigger refetch
    let (sessions_version, set_sessions_version) = signal(0u32);

    // Fetch sessions data (use LocalResource for CSR with non-Send futures)
    let sessions_resource = LocalResource::new(move || {
        let _ = sessions_version.get(); // Track version to trigger refetch
        fetch_sessions()
    });

    // Filter state
    let (search, set_search) = signal(String::new());
    let (project_filter, set_project_filter) = signal(None::<String>);
    let (model_filter, set_model_filter) = signal(None::<String>);
    let (date_filter, set_date_filter) = signal(None::<String>);

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
                    set_sessions_version.update(|v| *v += 1);
                    toast.success(format!("New session: {}", id));
                }
                SseEvent::SessionUpdated { id } => {
                    set_sessions_version.update(|v| *v += 1);
                    toast.info(format!("Session updated: {}", id));
                }
                SseEvent::StatsUpdated => {
                    set_sessions_version.update(|v| *v += 1);
                }
                _ => {}
            }
        }
    });

    // Extract unique projects from sessions
    let available_projects = Memo::new(move |_| {
        sessions_resource
            .get()
            .and_then(|result| result.as_ref().ok().cloned())
            .map(|sessions| {
                let mut projects: Vec<String> = sessions
                    .iter()
                    .map(|s| s.project.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                projects.sort();
                projects
            })
            .unwrap_or_default()
    });

    // Filter sessions based on search and filters
    let filtered_sessions = Memo::new(move |_| {
        sessions_resource
            .get()
            .and_then(|result| result.as_ref().ok().cloned())
            .map(|sessions| {
                let search_term = search.get().to_lowercase();
                let project_f = project_filter.get();
                let model_f = model_filter.get();
                let date_f = date_filter.get();

                sessions
                    .into_iter()
                    .filter(|session| {
                        // Search filter
                        if !search_term.is_empty() {
                            let matches_search = session.id.to_lowercase().contains(&search_term)
                                || session.project.to_lowercase().contains(&search_term)
                                || session
                                    .preview
                                    .as_ref()
                                    .map(|p| p.to_lowercase().contains(&search_term))
                                    .unwrap_or(false);

                            if !matches_search {
                                return false;
                            }
                        }

                        // Project filter
                        if let Some(ref project) = project_f {
                            if &session.project != project {
                                return false;
                            }
                        }

                        // Model filter
                        if let Some(ref model) = model_f {
                            if !session.model.contains(model) {
                                return false;
                            }
                        }

                        // Date filter
                        if let Some(ref date_range) = date_f {
                            if let Some(ref date_str) = session.date {
                                if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
                                    let now = Utc::now();
                                    let cutoff = match date_range.as_str() {
                                        "7d" => now - Duration::days(7),
                                        "30d" => now - Duration::days(30),
                                        "90d" => now - Duration::days(90),
                                        _ => return true,
                                    };

                                    if date.with_timezone(&Utc) < cutoff {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }

                        true
                    })
                    .collect::<Vec<_>>()
            })
    });

    // Handle Escape key to close modal
    leptos::leptos_dom::helpers::window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            set_modal_session.set(None);
        }
    });

    view! {
        <div class="page sessions-page">
            <div class="page-header">
                <h2>"Sessions Explorer"</h2>
                <div class="page-actions">
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(sessions) = filtered_sessions.get() {
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
                                            format!("{:.4}", s.cost),
                                        ]
                                    })
                                    .collect();
                                export_as_csv(headers, rows, "ccboard-sessions");
                            }
                        }
                    >
                        "📥 Export CSV"
                    </button>
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(sessions) = filtered_sessions.get() {
                                export_as_json(&sessions, "ccboard-sessions");
                            }
                        }
                    >
                        "📥 Export JSON"
                    </button>
                </div>
            </div>

            <Suspense fallback=move || {
                view! { <div class="loading">"Loading sessions..."</div> }
            }>
                {move || {
                    sessions_resource
                        .get()
                        .map(|result| {
                            match result.as_ref() {
                                Ok(_) => {
                                    view! {
                                        <div class="page-content">
                                            <SearchBar
                                                search=search
                                                set_search=set_search
                                                project_filter=project_filter
                                                set_project_filter=set_project_filter
                                                model_filter=model_filter
                                                set_model_filter=set_model_filter
                                                date_filter=date_filter
                                                set_date_filter=set_date_filter
                                                available_projects=available_projects.into()
                                            />
                                            {move || {
                                                let sessions = filtered_sessions.get();
                                                if let Some(sessions) = sessions {
                                                    if sessions.is_empty() {
                                                        view! {
                                                            <div class="empty-state">
                                                                <p>"No sessions match your filters."</p>
                                                                <p class="hint">
                                                                    "Try adjusting your search or filters."
                                                                </p>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {
                                                            <SessionTable
                                                                sessions=Signal::derive(move || {
                                                                    filtered_sessions.get()
                                                                })
                                                                on_row_click=set_modal_session
                                                            />
                                                        }
                                                            .into_any()
                                                    }
                                                } else {
                                                    view! {
                                                        <div class="empty-state">
                                                            <p>"No sessions found."</p>
                                                            <p class="hint">
                                                                "Start using Claude Code to see sessions here!"
                                                            </p>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}

                                        </div>
                                    }
                                        .into_any()
                                }
                                Err(e) => {
                                    let error_msg = e.clone();
                                    view! {
                                        <div class="error-state">
                                            <p>"Failed to load sessions"</p>
                                            <p class="hint">{error_msg}</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                        })
                }}

            </Suspense>

            <Show when=move || modal_session.get().is_some()>
                <SessionDetailModal session=modal_session on_close=move || {
                    set_modal_session.set(None)
                } />
            </Show>
        </div>
    }
}
