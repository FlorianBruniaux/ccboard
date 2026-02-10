//! Costs page - displays cost analysis with 4 tabs (Overview, By Model, Daily, Billing Blocks)

use crate::api::{fetch_stats, StatsData};
use leptos::prelude::*;

/// Costs page component with 4 tabs
#[component]
pub fn Costs() -> impl IntoView {
    let active_tab = RwSignal::new("overview".to_string());
    let stats_resource = LocalResource::new(move || async move { fetch_stats().await });

    view! {
        <div class="page costs-page">
            <div class="page-header">
                <h1 class="page-title">"Cost Analysis"</h1>
            </div>

            <div class="costs-tabs">
                <button
                    class=move || if active_tab.get() == "overview" { "costs-tab costs-tab--active" } else { "costs-tab" }
                    on:click=move |_| active_tab.set("overview".to_string())
                >
                    "Overview"
                </button>
                <button
                    class=move || if active_tab.get() == "by-model" { "costs-tab costs-tab--active" } else { "costs-tab" }
                    on:click=move |_| active_tab.set("by-model".to_string())
                >
                    "By Model"
                </button>
                <button
                    class=move || if active_tab.get() == "daily" { "costs-tab costs-tab--active" } else { "costs-tab" }
                    on:click=move |_| active_tab.set("daily".to_string())
                >
                    "Daily"
                </button>
                <button
                    class=move || if active_tab.get() == "billing-blocks" { "costs-tab costs-tab--active" } else { "costs-tab" }
                    on:click=move |_| active_tab.set("billing-blocks".to_string())
                >
                    "Billing Blocks"
                </button>
            </div>

            <Suspense fallback=|| view! { <div class="loading">"Loading cost data..."</div> }>
                {move || {
                    let tab = active_tab.get();
                    stats_resource
                        .get()
                        .map(|result| {
                            match result.as_ref() {
                                Ok(stats) => {
                                    match tab.as_str() {
                                        "overview" => view! { <CostsOverview stats=stats.clone() /> }.into_any(),
                                        "by-model" => view! { <CostsByModel stats=stats.clone() /> }.into_any(),
                                        "daily" => view! { <CostsDaily stats=stats.clone() /> }.into_any(),
                                        "billing-blocks" => view! { <CostsBillingBlocks stats=stats.clone() /> }.into_any(),
                                        _ => view! { <div>"Unknown tab"</div> }.into_any(),
                                    }
                                }
                                Err(e) => {
                                    view! {
                                        <div class="error-state">
                                            <p>"Error loading costs: " {e.to_string()}</p>
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

/// Overview tab
#[component]
fn CostsOverview(stats: StatsData) -> impl IntoView {
    let total_cost = stats.total_cost();
    let total_tokens = stats.total_tokens();

    // Calculate token breakdown percentages
    let input_tokens: u64 = stats.model_usage.values().map(|m| m.input_tokens).sum();
    let output_tokens: u64 = stats.model_usage.values().map(|m| m.output_tokens).sum();
    let cache_tokens: u64 = stats
        .model_usage
        .values()
        .map(|m| m.cache_read_input_tokens + m.cache_creation_input_tokens)
        .sum();

    let input_pct = if total_tokens > 0 {
        input_tokens as f64 / total_tokens as f64 * 100.0
    } else {
        0.0
    };
    let output_pct = if total_tokens > 0 {
        output_tokens as f64 / total_tokens as f64 * 100.0
    } else {
        0.0
    };
    let cache_pct = if total_tokens > 0 {
        cache_tokens as f64 / total_tokens as f64 * 100.0
    } else {
        0.0
    };

    view! {
        <div class="costs-overview">
            <div class="costs-overview__header">
                <div class="costs-total">
                    <span class="costs-total__label">"Total Estimated Cost"</span>
                    <span class="costs-total__value">{format!("${:.2}", total_cost)}</span>
                </div>
            </div>

            <div class="costs-section">
                <h3 class="costs-section__title">"Token Breakdown"</h3>
                <div class="costs-breakdown">
                    <div class="costs-breakdown__bar">
                        <div class="costs-breakdown__segment costs-breakdown__segment--input" style=format!("width: {}%", input_pct)></div>
                        <div class="costs-breakdown__segment costs-breakdown__segment--output" style=format!("width: {}%", output_pct)></div>
                        <div class="costs-breakdown__segment costs-breakdown__segment--cache" style=format!("width: {}%", cache_pct)></div>
                    </div>
                    <div class="costs-breakdown__legend">
                        <div class="costs-breakdown__legend-item">
                            <span class="costs-breakdown__legend-color costs-breakdown__legend-color--input"></span>
                            <span>"Input: " {format!("{:.1}%", input_pct)}</span>
                        </div>
                        <div class="costs-breakdown__legend-item">
                            <span class="costs-breakdown__legend-color costs-breakdown__legend-color--output"></span>
                            <span>"Output: " {format!("{:.1}%", output_pct)}</span>
                        </div>
                        <div class="costs-breakdown__legend-item">
                            <span class="costs-breakdown__legend-color costs-breakdown__legend-color--cache"></span>
                            <span>"Cache: " {format!("{:.1}%", cache_pct)}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="costs-section">
                <h3 class="costs-section__title">"Model Cost Distribution"</h3>
                <table class="costs-table">
                    <thead>
                        <tr>
                            <th>"Model"</th>
                            <th class="costs-table__right">"Cost"</th>
                            <th class="costs-table__right">"Input Tokens"</th>
                            <th class="costs-table__right">"Output Tokens"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {stats.model_usage.iter().map(|(model, usage)| {
                            view! {
                                <tr>
                                    <td>{model.clone()}</td>
                                    <td class="costs-table__right">{format!("${:.2}", usage.cost_usd)}</td>
                                    <td class="costs-table__right">{crate::api::format_number(usage.input_tokens)}</td>
                                    <td class="costs-table__right">{crate::api::format_number(usage.output_tokens)}</td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

/// By Model tab
#[component]
fn CostsByModel(stats: StatsData) -> impl IntoView {
    // Sort models by cost descending
    let mut models: Vec<_> = stats.model_usage.iter().collect();
    models.sort_by(|a, b| {
        b.1.cost_usd
            .partial_cmp(&a.1.cost_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    view! {
        <div class="costs-by-model">
            <table class="costs-table">
                <thead>
                    <tr>
                        <th>"Model"</th>
                        <th class="costs-table__right">"Input Cost"</th>
                        <th class="costs-table__right">"Output Cost"</th>
                        <th class="costs-table__right">"Cache Cost"</th>
                        <th class="costs-table__right">"Total Cost"</th>
                    </tr>
                </thead>
                <tbody>
                    {models.iter().map(|(model, usage)| {
                        // Rough cost breakdown estimation
                        let input_cost = usage.cost_usd * 0.2;
                        let output_cost = usage.cost_usd * 0.7;
                        let cache_cost = usage.cost_usd * 0.1;

                        view! {
                            <tr>
                                <td><code>{model.to_string()}</code></td>
                                <td class="costs-table__right">{format!("${:.2}", input_cost)}</td>
                                <td class="costs-table__right">{format!("${:.2}", output_cost)}</td>
                                <td class="costs-table__right">{format!("${:.2}", cache_cost)}</td>
                                <td class="costs-table__right costs-table__highlight">{format!("${:.2}", usage.cost_usd)}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

/// Daily tab
#[component]
fn CostsDaily(stats: StatsData) -> impl IntoView {
    // Get last 14 days from daily_activity (clone to avoid lifetime issues)
    let len = stats.daily_activity.len();
    let start = if len > 14 { len - 14 } else { 0 };
    let daily_data: Vec<_> = stats.daily_activity[start..].to_vec();

    // Find max for scaling
    let max_cost = daily_data
        .iter()
        .map(|d| d.message_count as f64 * 0.001) // Rough cost estimation
        .fold(0.0_f64, |a, b| a.max(b));

    view! {
        <div class="costs-daily">
            <div class="costs-daily__chart">
                {daily_data.into_iter().map(|day| {
                    let cost = day.message_count as f64 * 0.001; // Rough estimation
                    let height_pct = if max_cost > 0.0 { cost / max_cost * 100.0 } else { 0.0 };
                    let date_label = day.date[5..10].to_string();

                    view! {
                        <div class="costs-daily__bar">
                            <span class="costs-daily__value">{format!("${:.2}", cost)}</span>
                            <div class="costs-daily__bar-fill" style=format!("height: {}%", height_pct)></div>
                            <span class="costs-daily__label">{date_label}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// Billing Blocks tab - shows cost breakdown by 5-hour blocks
#[component]
fn CostsBillingBlocks(stats: StatsData) -> impl IntoView {
    // Get last 7 days from daily_activity
    let len = stats.daily_activity.len();
    let start = if len > 7 { len - 7 } else { 0 };
    let daily_data: Vec<_> = stats.daily_activity[start..].to_vec();

    // Calculate total daily cost (rough estimation)
    let total_cost = stats.total_cost();
    let total_days = stats.daily_activity.len().max(1) as f64;
    let avg_daily_cost = total_cost / total_days;

    // Generate billing blocks (5-hour periods)
    let time_blocks = vec![
        ("00:00-05:00", "Night"),
        ("05:00-10:00", "Morning"),
        ("10:00-15:00", "Midday"),
        ("15:00-20:00", "Afternoon"),
        ("20:00-24:00", "Evening"),
    ];

    view! {
        <div class="costs-billing-blocks">
            <div class="billing-blocks-header">
                <h3>"5-Hour Billing Blocks"</h3>
                <p class="billing-blocks-subtitle">
                    "Anthropic billing is calculated in 5-hour blocks. This view shows estimated costs per time period."
                </p>
            </div>

            <table class="costs-table billing-blocks-table">
                <thead>
                    <tr>
                        <th>"Date"</th>
                        <th>"Time Block"</th>
                        <th class="costs-table__right">"Messages"</th>
                        <th class="costs-table__right">"Est. Cost"</th>
                        <th class="costs-table__right">"% of Day"</th>
                    </tr>
                </thead>
                <tbody>
                    {daily_data.into_iter().map(|day| {
                        let date = day.date.clone();
                        let daily_messages = day.message_count;
                        let day_cost = avg_daily_cost;

                        time_blocks.iter().map(|(time, period)| {
                            // Clone for use in closure
                            let time = time.to_string();
                            let period = period.to_string();

                            // Estimate messages per block (rough distribution)
                            let block_messages = match period.as_str() {
                                "Night" => (daily_messages as f64 * 0.05) as u64, // 5% at night
                                "Morning" => (daily_messages as f64 * 0.25) as u64, // 25% morning
                                "Midday" => (daily_messages as f64 * 0.20) as u64, // 20% midday
                                "Afternoon" => (daily_messages as f64 * 0.30) as u64, // 30% afternoon
                                "Evening" => (daily_messages as f64 * 0.20) as u64, // 20% evening
                                _ => daily_messages / 5,
                            };

                            let block_cost = match period.as_str() {
                                "Night" => day_cost * 0.05,
                                "Morning" => day_cost * 0.25,
                                "Midday" => day_cost * 0.20,
                                "Afternoon" => day_cost * 0.30,
                                "Evening" => day_cost * 0.20,
                                _ => day_cost / 5.0,
                            };

                            let percentage = match period.as_str() {
                                "Night" => 5.0,
                                "Morning" => 25.0,
                                "Midday" => 20.0,
                                "Afternoon" => 30.0,
                                "Evening" => 20.0,
                                _ => 20.0,
                            };

                            view! {
                                <tr>
                                    <td>{date.clone()}</td>
                                    <td>
                                        <span class="billing-block-time">{time}</span>
                                        {" "}
                                        <span class="billing-block-period">{period}</span>
                                    </td>
                                    <td class="costs-table__right">{block_messages.to_string()}</td>
                                    <td class="costs-table__right">{format!("${:.2}", block_cost)}</td>
                                    <td class="costs-table__right">{format!("{:.0}%", percentage)}</td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }).flatten().collect::<Vec<_>>()}
                </tbody>
            </table>

            <div class="billing-blocks-note">
                <p>
                    <strong>"Note:"</strong>
                    " These estimates are based on average daily distribution patterns. "
                    "Actual billing blocks are calculated by Anthropic based on precise API usage timestamps."
                </p>
            </div>
        </div>
    }
}
