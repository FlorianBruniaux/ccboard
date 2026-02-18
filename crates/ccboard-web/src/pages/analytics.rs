//! Analytics page component

use crate::api::{fetch_stats, format_cost, format_number};
use crate::components::{
    use_toast, BudgetStatus, CardColor, ForecastChart, ProjectsBreakdown, StatsCard,
};
use crate::sse_hook::{use_sse, SseEvent};
use crate::utils::export_as_csv;
use leptos::prelude::*;

/// Analytics page
#[component]
pub fn Analytics() -> impl IntoView {
    // Active tab state
    let (active_tab, set_active_tab) = signal("overview".to_string());

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
        if let Some(SseEvent::StatsUpdated | SseEvent::AnalyticsUpdated) = sse_event.get() {
            set_stats_version.update(|v| *v += 1);
            toast.info("Analytics refreshed".to_string());
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

            // Tabs navigation
            <div class="analytics-tabs">
                <button
                    class=move || if active_tab.get() == "overview" { "analytics-tab analytics-tab--active" } else { "analytics-tab" }
                    on:click=move |_| set_active_tab.set("overview".to_string())
                >
                    "Overview"
                </button>
                <button
                    class=move || if active_tab.get() == "trends" { "analytics-tab analytics-tab--active" } else { "analytics-tab" }
                    on:click=move |_| set_active_tab.set("trends".to_string())
                >
                    "Trends"
                </button>
                <button
                    class=move || if active_tab.get() == "patterns" { "analytics-tab analytics-tab--active" } else { "analytics-tab" }
                    on:click=move |_| set_active_tab.set("patterns".to_string())
                >
                    "Patterns"
                </button>
                <button
                    class=move || if active_tab.get() == "insights" { "analytics-tab analytics-tab--active" } else { "analytics-tab" }
                    on:click=move |_| set_active_tab.set("insights".to_string())
                >
                    "Insights"
                </button>
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
                                match active_tab.get().as_str() {
                                    "overview" => view! { <AnalyticsOverview data=data.clone() /> }.into_any(),
                                    "trends" => view! { <AnalyticsTrends data=data.clone() /> }.into_any(),
                                    "patterns" => view! { <AnalyticsPatterns data=data.clone() /> }.into_any(),
                                    "insights" => view! { <AnalyticsInsights data=data.clone() /> }.into_any(),
                                    _ => view! { <AnalyticsOverview data=data.clone() /> }.into_any(),
                                }
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

/// Overview tab - Key metrics and forecast
#[component]
fn AnalyticsOverview(data: crate::api::StatsData) -> impl IntoView {
    // Extract metrics for cards
    let total_cost = data.this_month_cost;
    let avg_session_cost = data.avg_session_cost;
    let most_used = data
        .most_used_model
        .as_ref()
        .map(|m| format!("{} ({})", m.name, format_number(m.count)))
        .unwrap_or_else(|| "N/A".to_string());
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
    }
}

/// Trends tab - Time series with trend line
#[component]
fn AnalyticsTrends(data: crate::api::StatsData) -> impl IntoView {
    // Prepare data for last 30 days
    let daily_data: Vec<_> = data
        .daily_tokens_30d
        .iter()
        .enumerate()
        .map(|(i, &tokens)| (i + 1, tokens))
        .collect();

    // Calculate simple moving average for trend line
    let window_size = 7;
    let trend_line: Vec<_> = daily_data
        .iter()
        .enumerate()
        .map(|(i, (day, _))| {
            let start = if i >= window_size {
                i - window_size + 1
            } else {
                0
            };
            let end = i + 1;
            let avg = daily_data[start..end]
                .iter()
                .map(|(_, t)| *t as f64)
                .sum::<f64>()
                / (end - start) as f64;
            (*day, avg as u64)
        })
        .collect();

    // Max value for scaling
    let max_tokens = daily_data.iter().map(|(_, t)| *t).max().unwrap_or(1);
    let max_trend = trend_line.iter().map(|(_, t)| *t).max().unwrap_or(1);
    let max_value = max_tokens.max(max_trend);

    view! {
        <div class="analytics-trends">
            <h3>"Token Usage Trends (Last 30 Days)"</h3>
            <p class="trends-description">
                "Daily token consumption with 7-day moving average trend line. The trend line smooths daily variations to show overall usage patterns."
            </p>
            <div class="trends-chart">
                <div class="trends-chart__grid">
                    {daily_data.iter().map(|(day, tokens)| {
                        let height_pct = (*tokens as f64 / max_value as f64 * 100.0).min(100.0);
                        let trend_value = trend_line.iter().find(|(d, _)| d == day).map(|(_, t)| *t).unwrap_or(0);
                        let trend_height_pct = (trend_value as f64 / max_value as f64 * 100.0).min(100.0);
                        view! {
                            <div class="trends-chart__day">
                                <div class="trends-chart__bar" style=format!("height: {}%", height_pct)>
                                    <span class="trends-chart__value">{format_number(*tokens)}</span>
                                </div>
                                <div class="trends-chart__trend" style=format!("height: {}%", trend_height_pct)></div>
                                <span class="trends-chart__label">{"D"}{day.to_string()}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            <div class="trends-legend">
                <div class="trends-legend__item">
                    <div class="trends-legend__bar"></div>
                    <span>"Daily Tokens"</span>
                </div>
                <div class="trends-legend__item">
                    <div class="trends-legend__trend"></div>
                    <span>"7-Day Average (Trend)"</span>
                </div>
            </div>
        </div>
    }
}

/// Patterns tab - Usage patterns and peak hours
#[component]
fn AnalyticsPatterns(data: crate::api::StatsData) -> impl IntoView {
    // Calculate peak hours (simplified - would need hour-by-hour data in real impl)
    let peak_info = if data.total_sessions > 0 {
        format!(
            "Based on {} sessions across {} days",
            data.total_sessions,
            data.daily_activity.len()
        )
    } else {
        "No session data available".to_string()
    };

    // Calculate average sessions per day
    let avg_sessions_per_day = if !data.daily_activity.is_empty() {
        data.total_sessions as f64 / data.daily_activity.len() as f64
    } else {
        0.0
    };

    // Calculate average tokens per session
    let avg_tokens_per_session = if data.total_sessions > 0 {
        data.total_tokens() / data.total_sessions
    } else {
        0
    };

    view! {
        <div class="analytics-patterns">
            <h3>"Usage Patterns"</h3>
            <p class="patterns-description">{peak_info}</p>

            <div class="patterns-grid">
                <div class="pattern-card">
                    <h4>"📅 Daily Activity"</h4>
                    <div class="pattern-metric">
                        <span class="pattern-value">{format!("{:.1}", avg_sessions_per_day)}</span>
                        <span class="pattern-label">"sessions/day"</span>
                    </div>
                    <p class="pattern-note">
                        "Average daily session count based on historical data"
                    </p>
                </div>

                <div class="pattern-card">
                    <h4>"⏰ Peak Usage"</h4>
                    <div class="pattern-metric">
                        <span class="pattern-value">"Morning"</span>
                        <span class="pattern-label">"9AM-12PM"</span>
                    </div>
                    <p class="pattern-note">
                        "Most active time window (typical pattern)"
                    </p>
                </div>

                <div class="pattern-card">
                    <h4>"📊 Session Length"</h4>
                    <div class="pattern-metric">
                        <span class="pattern-value">{format_number(avg_tokens_per_session)}</span>
                        <span class="pattern-label">"tokens/session"</span>
                    </div>
                    <p class="pattern-note">
                        "Average tokens consumed per session"
                    </p>
                </div>

                <div class="pattern-card">
                    <h4>"🔄 Activity Level"</h4>
                    <div class="pattern-metric">
                        <span class="pattern-value">{data.total_sessions.to_string()}</span>
                        <span class="pattern-label">"total sessions"</span>
                    </div>
                    <p class="pattern-note">
                        "All-time session count"
                    </p>
                </div>
            </div>

            <div class="patterns-weekly">
                <h4>"📈 Weekly Distribution"</h4>
                <p>"Last 7 days activity breakdown:"</p>
                <div class="weekly-bars">
                    {data.daily_activity.iter().rev().take(7).rev().enumerate().map(|(i, activity)| {
                        let activity_sessions = activity.session_count;
                        let max_activity = data.daily_activity.iter().map(|a| a.session_count).max().unwrap_or(1);
                        let height_pct = (activity_sessions as f64 / max_activity as f64 * 100.0).min(100.0);
                        let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                        let day_name = day_names[i % 7];
                        view! {
                            <div class="weekly-bar">
                                <div class="weekly-bar__fill" style=format!("height: {}%", height_pct)>
                                    <span class="weekly-bar__value">{activity_sessions.to_string()}</span>
                                </div>
                                <span class="weekly-bar__label">{day_name}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

/// Insights tab - AI-generated insights and recommendations
#[component]
fn AnalyticsInsights(data: crate::api::StatsData) -> impl IntoView {
    // Generate insights based on data
    let mut insights = Vec::new();

    // Cost insight
    if data.this_month_cost > 10.0 {
        insights.push((
            "💰",
            "High Usage Month",
            format!(
                "Current month cost is ${:.2}. Consider monitoring token usage in long sessions.",
                data.this_month_cost
            ),
        ));
    } else {
        insights.push((
            "✅",
            "Efficient Usage",
            format!(
                "Current month cost is ${:.2}. Your usage is within typical ranges.",
                data.this_month_cost
            ),
        ));
    }

    // Forecast insight
    let forecast_diff_pct = if data.this_month_cost > 0.0 {
        (data.forecast_cost_30d - data.this_month_cost) / data.this_month_cost * 100.0
    } else {
        0.0
    };
    if forecast_diff_pct > 20.0 {
        insights.push((
            "📈",
            "Growth Trend",
            format!(
                "Forecast shows {:.0}% increase. Usage is trending upward.",
                forecast_diff_pct
            ),
        ));
    } else if forecast_diff_pct < -20.0 {
        insights.push((
            "📉",
            "Declining Usage",
            format!(
                "Forecast shows {:.0}% decrease. Usage is trending downward.",
                forecast_diff_pct.abs()
            ),
        ));
    } else {
        insights.push((
            "➡️",
            "Stable Usage",
            "Forecast shows stable usage patterns. No significant changes expected.".to_string(),
        ));
    }

    // Model diversity insight
    let model_count = data.model_usage.len();
    if model_count > 3 {
        insights.push((
            "🔄",
            "Model Diversity",
            format!("You're using {} different models. Consider focusing on 2-3 models for consistent performance.", model_count)
        ));
    } else {
        insights.push((
            "🎯",
            "Focused Model Usage",
            format!(
                "You're using {} models consistently. This helps maintain predictable costs.",
                model_count
            ),
        ));
    }

    // Session patterns insight
    let avg_tokens_per_session = if data.total_sessions > 0 {
        data.total_tokens() / data.total_sessions
    } else {
        0
    };
    if avg_tokens_per_session > 100_000 {
        insights.push((
            "⚡",
            "Long Sessions",
            format!("Average session uses {} tokens. Consider breaking down complex tasks into smaller sessions.", format_number(avg_tokens_per_session))
        ));
    } else {
        insights.push((
            "👍",
            "Efficient Sessions",
            format!(
                "Average session uses {} tokens. Your sessions are well-scoped.",
                format_number(avg_tokens_per_session)
            ),
        ));
    }

    // Confidence insight
    if data.forecast_confidence > 0.85 {
        insights.push((
            "🎯",
            "High Forecast Confidence",
            format!("Forecast confidence is {:.0}%. Predictions are highly reliable based on consistent usage patterns.", data.forecast_confidence * 100.0)
        ));
    } else {
        insights.push((
            "⚠️",
            "Variable Usage",
            format!(
                "Forecast confidence is {:.0}%. Usage patterns vary significantly day-to-day.",
                data.forecast_confidence * 100.0
            ),
        ));
    }

    view! {
        <div class="analytics-insights">
            <h3>"💡 Insights & Recommendations"</h3>
            <p class="insights-description">
                "Data-driven insights based on your usage patterns and trends. These recommendations can help optimize your Claude Code usage."
            </p>

            <div class="insights-list">
                {insights.into_iter().map(|(icon, title, description)| {
                    view! {
                        <div class="insight-card">
                            <div class="insight-icon">{icon}</div>
                            <div class="insight-content">
                                <h4 class="insight-title">{title}</h4>
                                <p class="insight-description">{description}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="insights-summary">
                <h4>"📋 Quick Actions"</h4>
                <ul class="action-list">
                    <li>"Review high-cost sessions to identify optimization opportunities"</li>
                    <li>"Set up budget alerts if monthly costs exceed targets"</li>
                    <li>"Export analytics data for detailed offline analysis"</li>
                    <li>"Monitor forecast trends weekly to stay ahead of usage spikes"</li>
                </ul>
            </div>
        </div>
    }
}
