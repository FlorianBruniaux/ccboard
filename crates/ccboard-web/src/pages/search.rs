//! Search page — FTS5 full-text search across sessions

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "";

// ─── API types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResultItem {
    session_id: String,
    project: Option<String>,
    first_user_message: Option<String>,
    snippet: Option<String>,
    rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResultItem>,
    total: usize,
    query: String,
}

// ─── Fetch helper ─────────────────────────────────────────────────────────────

async fn fetch_search(query: String, limit: usize) -> Result<SearchResponse, String> {
    let encoded = js_sys::encode_uri_component(&query);
    let url = format!("{}/api/search?q={}&limit={}", API_BASE_URL, encoded, limit);

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let resp: SearchResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(resp)
}

// ─── Main page component ──────────────────────────────────────────────────────

/// Search page with FTS5 full-text search
#[component]
pub fn SearchPage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let submitted_query: RwSignal<Option<String>> = RwSignal::new(None);

    // Reactive resource — only fetches when submitted_query is Some (user pressed Enter/button)
    let search_resource = LocalResource::new(move || {
        let q = submitted_query.get();
        async move {
            match q {
                Some(q) if q.trim().len() >= 2 => fetch_search(q, 50).await,
                Some(_) => Err("Query must be at least 2 characters".to_string()),
                None => Ok(SearchResponse {
                    results: Vec::new(),
                    total: 0,
                    query: String::new(),
                }),
            }
        }
    });

    let do_search = move || {
        let q = query.get();
        if q.trim().len() >= 2 {
            submitted_query.set(Some(q));
        }
    };

    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"Search Sessions"</h1>
                <p class="page-subtitle">"Full-text search across all session messages"</p>
            </div>

            // Search input
            <div class="card" style="margin-bottom: 1.5rem;">
                <div style="display: flex; gap: 1rem; align-items: center;">
                    <input
                        type="text"
                        placeholder="Search sessions... (min 2 chars)"
                        style="flex: 1; padding: 0.75rem 1rem; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: 6px; color: var(--text-primary); font-size: 1rem;"
                        prop:value=query
                        on:input=move |ev| {
                            query.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                do_search();
                            }
                        }
                    />
                    <button
                        style="padding: 0.75rem 1.5rem; background: var(--accent); color: white; border: none; border-radius: 6px; cursor: pointer;"
                        on:click=move |_| {
                            do_search();
                        }
                    >
                        "Search"
                    </button>
                </div>
            </div>

            // Results
            <Suspense fallback=|| view! { <div class="card" style="text-align: center; color: var(--text-muted);">"Searching..."</div> }>
                {move || {
                    search_resource.get().map(|result| {
                        match result.as_ref() {
                            Ok(response) => {
                                if submitted_query.get().is_none() {
                                    return view! {
                                        <div class="card" style="text-align: center; color: var(--text-muted); padding: 3rem;">
                                            "Enter a query and press Enter or click Search."
                                        </div>
                                    }.into_any();
                                }

                                if response.results.is_empty() {
                                    return view! {
                                        <div class="card" style="text-align: center; color: var(--text-muted); padding: 3rem;">
                                            "No results found. Try searching for session content, project names, or model names."
                                        </div>
                                    }.into_any();
                                }

                                let results = response.results.clone();
                                let total = response.total;

                                view! {
                                    <div>
                                        <div style="margin-bottom: 1rem; color: var(--text-muted); font-size: 0.9rem;">
                                            {format!("{} result(s)", total)}
                                        </div>
                                        <div>
                                            {results.into_iter().map(|item| {
                                                let snippet = item.snippet
                                                    .clone()
                                                    .or_else(|| item.first_user_message.clone())
                                                    .unwrap_or_else(|| "(no preview)".to_string());
                                                let project = item.project.clone().unwrap_or_else(|| "unknown".to_string());
                                                let session_id = item.session_id.clone();
                                                let id_len = session_id.len().min(8);
                                                let session_id_short = session_id[..id_len].to_string();
                                                let href = format!("/sessions?id={}", session_id);

                                                view! {
                                                    <a
                                                        href=href
                                                        style="display: block; text-decoration: none; padding: 1rem; margin-bottom: 0.5rem; background: var(--bg-secondary); border-radius: 8px; border: 1px solid var(--border);"
                                                    >
                                                        <div style="display: flex; justify-content: space-between; margin-bottom: 0.5rem;">
                                                            <span style="color: var(--accent); font-weight: 600;">{project}</span>
                                                            <span style="color: var(--text-muted); font-size: 0.85rem;">{session_id_short}</span>
                                                        </div>
                                                        <div style="color: var(--text-secondary); font-size: 0.9rem;">{snippet}</div>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div style="color: var(--error, #f87171); padding: 1rem;">
                                    {format!("Error: {}", e)}
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
