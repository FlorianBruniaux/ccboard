//! Analytics page component

use crate::api::{fetch_stats, format_cost, format_number};
use crate::components::{BudgetStatus, CardColor, ForecastChart, ProjectsBreakdown, StatsCard};
use leptos::prelude::*;

/// Analytics page
#[component]
pub fn Analytics() -> impl IntoView {
    // Fetch stats with analytics data
    let stats = LocalResource::new(|| async move { fetch_stats().await });

    view! {
        <div class="page analytics-page">
            <h2>"Analytics"</h2>
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
