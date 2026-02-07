//! Dashboard page component

use crate::api::{fetch_sessions, fetch_stats, format_cost, format_number};
use crate::components::{CardColor, Sparkline, StatsCard};
use leptos::prelude::*;
use leptos::web_sys::{EventSource, MessageEvent};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Dashboard page - main overview with live stats
#[component]
pub fn Dashboard() -> impl IntoView {
    // Stats resource for initial load and manual refresh
    let (stats_version, set_stats_version) = signal(0u32);
    let stats = LocalResource::new(move || {
        let _ = stats_version.get(); // Track this to trigger refetch
        async move { fetch_stats().await }
    });

    // Sessions resource for recent sessions list
    let sessions = LocalResource::new(move || {
        let _ = stats_version.get(); // Track this to trigger refetch
        async move { fetch_sessions().await }
    });

    // SSE setup for live updates
    Effect::new(move |_| {
        let event_source = match EventSource::new("/api/events") {
            Ok(es) => es,
            Err(_) => return,
        };

        // Handle stats_updated events
        let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().as_string() {
                if data.contains("stats_updated") {
                    // Increment version to trigger resource refetch
                    set_stats_version.update(|v| *v += 1);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        event_source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
    });

    view! {
        <div class="page dashboard-page">
            <h2>"Dashboard"</h2>
            <div class="page-content">
                <Suspense fallback=move || view! { <div class="loading">"Loading stats..."</div> }>
                    {move || match stats.get().as_ref().map(|r| r.as_ref()) {
                        Some(Ok(data)) => {
                            let total_tokens = data.total_tokens();
                            let total_cost = data.total_cost();
                            let avg_cost = data.avg_session_cost();
                            let this_month = data.this_month_sessions();
                            let this_week = data.this_week_tokens();
                            let daily_tokens = data.daily_tokens_30d();

                            // Determine color based on cost (example thresholds)
                            let cost_color = if total_cost > 100.0 {
                                CardColor::Red
                            } else if total_cost > 50.0 {
                                CardColor::Yellow
                            } else {
                                CardColor::Green
                            };

                            view! {
                                <div>
                                    <div class="stats-grid">
                                        <StatsCard
                                            label="Total Sessions".to_string()
                                            value=data.total_sessions.to_string()
                                            icon="📊".to_string()
                                            color=CardColor::Default
                                        />
                                        <StatsCard
                                            label="Total Tokens".to_string()
                                            value=format_number(total_tokens)
                                            icon="🔢".to_string()
                                            color=CardColor::Default
                                        />
                                        <StatsCard
                                            label="Total Cost".to_string()
                                            value=format_cost(total_cost)
                                            icon="💰".to_string()
                                            color=cost_color
                                        />
                                        <StatsCard
                                            label="Avg Session Cost".to_string()
                                            value=format_cost(avg_cost)
                                            icon="📈".to_string()
                                            color=CardColor::Default
                                        />
                                        <StatsCard
                                            label="This Month Sessions".to_string()
                                            value=this_month.to_string()
                                            icon="📅".to_string()
                                            color=CardColor::Default
                                        />
                                        <StatsCard
                                            label="This Week Tokens".to_string()
                                            value=format_number(this_week)
                                            icon="🔥".to_string()
                                            color=CardColor::Default
                                        />
                                    </div>

                                    <div class="sparkline-section">
                                        <Sparkline
                                            data=daily_tokens
                                            width=800
                                            height=100
                                            label="Token Usage - Last 30 Days".to_string()
                                        />
                                    </div>

                                    <div class="recent-sessions-section">
                                        <h3>"Recent Sessions"</h3>
                                        <Suspense fallback=move || view! { <div class="loading">"Loading sessions..."</div> }>
                                            {move || match sessions.get().as_ref().map(|r| r.as_ref()) {
                                                Some(Ok(session_data)) => {
                                                    view! {
                                                        <div class="table-container">
                                                            <table class="sessions-table">
                                                                <thead>
                                                                    <tr>
                                                                        <th>"ID"</th>
                                                                        <th>"Project"</th>
                                                                        <th>"Tokens"</th>
                                                                        <th>"Messages"</th>
                                                                        <th>"Preview"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    {session_data.recent.iter().take(5).map(|s| {
                                                                        let id = s.id.clone();
                                                                        let project = s.project.clone();
                                                                        let tokens = s.tokens;
                                                                        let messages = s.messages;
                                                                        let preview = s.preview.clone().unwrap_or_default();
                                                                        view! {
                                                                            <tr>
                                                                                <td class="session-id">{id}</td>
                                                                                <td>{project}</td>
                                                                                <td class="tokens">{format_number(tokens)}</td>
                                                                                <td>{messages}</td>
                                                                                <td class="preview">{preview}</td>
                                                                            </tr>
                                                                        }
                                                                    }).collect::<Vec<_>>()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    }.into_any()
                                                },
                                                Some(Err(e)) => {
                                                    let err = e.clone();
                                                    view! { <div class="error">"Failed to load sessions: " {err}</div> }.into_any()
                                                },
                                                None => {
                                                    view! { <div class="loading">"Loading sessions..."</div> }.into_any()
                                                }
                                            }}
                                        </Suspense>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        Some(Err(e)) => {
                            let err = e.clone();
                            view! { <div class="error">"Failed to load stats: " {err}</div> }.into_any()
                        },
                        None => {
                            view! { <div class="loading">"Loading stats..."</div> }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
