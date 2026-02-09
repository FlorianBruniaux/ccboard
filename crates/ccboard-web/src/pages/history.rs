//! History page - session timeline with filters

use crate::api::SessionData;
use crate::components::{SessionDetailModal, SessionTable};
use leptos::prelude::*;
use serde::Deserialize;

/// API base URL constant
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080";
#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "";

/// Sessions response
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
struct SessionsResponse {
    sessions: Vec<SessionData>,
    total: u64,
}

/// Fetch sessions for history timeline
async fn fetch_history(since: String) -> Result<SessionsResponse, String> {
    let url = format!("{}/api/sessions?page=0&limit=100&since={}&sort=date&order=desc", API_BASE_URL, since);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch history: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let data: SessionsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(data)
}

/// History page component
#[component]
pub fn History() -> impl IntoView {
    // Time filter signal (7d, 30d, 90d, all)
    let (time_filter, set_time_filter) = signal("30d".to_string());

    // Modal state for session details
    let (modal_session, set_modal_session) = signal(None::<SessionData>);

    // Resource that reloads when time_filter changes
    let history_resource = LocalResource::new(move || {
        let filter = time_filter.get();
        async move { fetch_history(filter).await }
    });

    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"History"</h1>
                <p class="page-description">
                    "Session history timeline - track your Claude Code usage patterns over time"
                </p>
            </div>

            // Time filter buttons
            <div class="filter-bar">
                <button
                    class="filter-btn"
                    class:active=move || time_filter.get() == "7d"
                    on:click=move |_| set_time_filter.set("7d".to_string())
                >
                    "Last 7 Days"
                </button>
                <button
                    class="filter-btn"
                    class:active=move || time_filter.get() == "30d"
                    on:click=move |_| set_time_filter.set("30d".to_string())
                >
                    "Last 30 Days"
                </button>
                <button
                    class="filter-btn"
                    class:active=move || time_filter.get() == "90d"
                    on:click=move |_| set_time_filter.set("90d".to_string())
                >
                    "Last 90 Days"
                </button>
                <button
                    class="filter-btn"
                    class:active=move || time_filter.get() == "365d"
                    on:click=move |_| set_time_filter.set("365d".to_string())
                >
                    "Last Year"
                </button>
            </div>

            <Suspense fallback=move || {
                view! { <p class="loading">"Loading history..."</p> }
            }>
                {move || {
                    history_resource
                        .get()
                        .map(|result| match result.as_ref() {
                            Ok(data) => {
                                let sessions = data.sessions.clone();
                                let sessions_signal = Signal::derive(move || Some(sessions.clone()));

                                view! {
                                    <div class="history-container">
                                        <div class="history-stats">
                                            <span class="history-count">
                                                {data.total}
                                                " sessions in this period"
                                            </span>
                                        </div>
                                        <SessionTable
                                            sessions=sessions_signal
                                            on_row_click=set_modal_session
                                        />
                                    </div>
                                }.into_any()
                            }
                            Err(e) => {
                                let error_msg = e.clone();
                                view! {
                                    <div class="error-message">
                                        <p>
                                            <strong>"Error loading history: "</strong>
                                            {error_msg}
                                        </p>
                                    </div>
                                }.into_any()
                            }
                        })
                }}
            </Suspense>

            // Modal rendered separately to avoid type issues
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
