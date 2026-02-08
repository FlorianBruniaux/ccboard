//! Analytics page component

use crate::api::{fetch_stats, format_cost, format_number};
use crate::components::{
    BudgetStatus, CardColor, ForecastChart, ProjectsBreakdown, StatsCard, use_toast,
};
use crate::sse_hook::{SseEvent, use_sse};
use crate::utils::export_as_csv;
use leptos::prelude::*;

/// Analytics page
#[component]
pub fn Analytics() -> impl IntoView {
    // Version signal to trigger refetch
    let (stats_version, set_stats_version) = signal(0u32);

    // Fetch stats with analytics data
    let stats = LocalResource::new(move || {
        let _ = stats_version.get(); // Track version to trigger refetch
        async move { fetch_stats().await }
    });

    // Toast notifications
    let toast = use_toast();

    // SSE setup for live updates
    let sse_event = use_sse();

    Effect::new(move |_| {
        if let Some(event) = sse_event.get() {
            match event {
                SseEvent::StatsUpdated | SseEvent::AnalyticsUpdated => {
                    set_stats_version.update(|v| *v += 1);
                    toast.info("Analytics refreshed".to_string());
                }
                _ => {}
            }
        }
    });

    view! {
        <div class="page analytics-page">
            <div class="page-header">
                <h2>"Analytics"</h2>
                <div class="page-actions">
                    <button
                        class="export-button"
                        on:click=move |_| {
                            if let Some(Ok(data)) = stats.get().as_ref().map(|r| r.as_ref()) {
                                let headers = vec![
                                    "Day".to_string(),
                                    "Historical Tokens".to_string(),
                                    "Forecast Tokens".to_string(),
                                ];
                                let rows: Vec<Vec<String>> = data
                                    .daily_tokens_30d
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &tokens)| {
                                        let forecast = data.forecast_tokens_30d.get(i).copied().unwrap_or(0);
                                        vec![
                                            format!("Day {}", i + 1),
                                            tokens.to_string(),
                                            forecast.to_string(),
                                        ]
                                    })
                                    .collect();
                                export_as_csv(headers, rows, "ccboard-analytics-forecast");
                            }
                        }
                    >
                        "📥 Export CSV"
                    </button>
                </div>
            </div>
            <div class="page-content">
                <Suspense fallback=move || {
                    view! {
                        <div class="loading-state">
                            <div class="spinner"></div>
                            <p>"Loading analytics..."</p>
                        </div>
                    }
                }>
                    {move || Suspend::new(async move {
                        match stats.await {
                            Ok(data) => {
                                // Extract metrics for cards
                                let total_cost = data.this_month_cost;
                                let avg_session_cost = data.avg_session_cost;
                                let most_used = data.most_used_model.as_ref().map(|m| {
                                    format!("{} ({})", m.name, format_number(m.count))
                                }).unwrap_or_else(|| "N/A".to_string());
                                let forecast_cost = data.forecast_cost_30d;

                                view! {
                                    <div class="analytics-content">
                                        // Key metrics summary (top section)
                                        <div class="metrics-summary">
                                            <StatsCard
                                                label="Total Cost (Month)".to_string()
                                                value={format_cost(total_cost)}
                                                icon="💰".to_string()
                                                color=CardColor::Default
                                            />
                                            <StatsCard
                                                label="Avg Cost Per Session".to_string()
                                                value={format_cost(avg_session_cost)}
                                                icon="📊".to_string()
                                                color=CardColor::Default
                                            />
                                            <StatsCard
                                                label="Most Used Model".to_string()
                                                value={most_used}
                                                icon="🤖".to_string()
                                                color=CardColor::Default
                                            />
                                            <StatsCard
                                                label="Forecast Next Month".to_string()
                                                value={format_cost(forecast_cost)}
                                                icon="📈".to_string()
                                                color=CardColor::Yellow
                                            />
                                        </div>

                                        // Forecast chart section
                                        <div class="forecast-section">
                                            <ForecastChart
                                                historical={data.daily_tokens_30d.clone()}
                                                forecast={data.forecast_tokens_30d.clone()}
                                                budget={None::<u64>}
                                                confidence={data.forecast_confidence}
                                            />
                                        </div>

                                        // Breakdown section (side by side on desktop, stacked on mobile)
                                        <div class="breakdown-section">
                                            <div class="breakdown-left">
                                                <BudgetStatus
                                                    used={data.total_tokens()}
                                                    budget={None::<u64>}
                                                />
                                            </div>
                                            <div class="breakdown-right">
                                                <ProjectsBreakdown
                                                    projects={data.projects_by_cost.clone()}
                                                />
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Err(err) => {
                                view! {
                                    <div class="error-state">
                                        <p class="error-message">"Failed to load analytics"</p>
                                        <p class="error-details">{err}</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
