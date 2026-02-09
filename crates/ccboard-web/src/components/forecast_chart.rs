//! Interactive forecast chart with SVG

use crate::api::format_number;
use leptos::prelude::*;

const CHART_WIDTH: f64 = 800.0;
const CHART_HEIGHT: f64 = 250.0;
const MARGIN_TOP: f64 = 50.0;
const MARGIN_BOTTOM: f64 = 50.0;
const MARGIN_LEFT: f64 = 70.0;
const MARGIN_RIGHT: f64 = 40.0;

/// Forecast chart props
#[component]
pub fn ForecastChart(
    /// Historical data (last 30 days)
    historical: Vec<u64>,
    /// Forecast data (next 30 days)
    forecast: Vec<u64>,
    /// Optional budget limit
    budget: Option<u64>,
    /// Confidence level (R²)
    #[prop(default = 0.0)]
    confidence: f64,
) -> impl IntoView {
    // Calculate chart dimensions
    let chart_inner_width = CHART_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    let chart_inner_height = CHART_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;

    // Combine historical + forecast for scaling
    let all_values: Vec<u64> = historical.iter().chain(forecast.iter()).copied().collect();
    let max_value = all_values
        .iter()
        .max()
        .copied()
        .unwrap_or(1000)
        .max(budget.unwrap_or(0));
    let min_value = 0u64; // Always start from 0

    // Scale functions
    let x_scale = |index: usize| -> f64 {
        MARGIN_LEFT
            + (index as f64 / (historical.len() + forecast.len()) as f64) * chart_inner_width
    };

    let y_scale = |value: u64| -> f64 {
        MARGIN_TOP + chart_inner_height
            - ((value as f64 - min_value as f64) / (max_value as f64 - min_value as f64))
                * chart_inner_height
    };

    // Generate path for historical line (blue, solid)
    let historical_path = if !historical.is_empty() {
        let mut path = format!("M {} {}", x_scale(0), y_scale(historical[0]));
        for (i, &value) in historical.iter().enumerate().skip(1) {
            path.push_str(&format!(" L {} {}", x_scale(i), y_scale(value)));
        }
        path
    } else {
        String::new()
    };

    // Generate path for forecast line (orange, dashed)
    let forecast_path = if !forecast.is_empty() {
        let start_index = historical.len();
        let mut path = format!(
            "M {} {}",
            x_scale(start_index),
            y_scale(*forecast.first().unwrap_or(&0))
        );
        for (i, &value) in forecast.iter().enumerate().skip(1) {
            path.push_str(&format!(
                " L {} {}",
                x_scale(start_index + i),
                y_scale(value)
            ));
        }
        path
    } else {
        String::new()
    };

    // Budget line (horizontal red dashed)
    let budget_y = budget.map(|b| y_scale(b));

    // Y-axis labels (4 ticks)
    let y_ticks: Vec<_> = (0..=3)
        .map(|i| {
            let value = min_value + (max_value - min_value) * i / 3;
            let y = y_scale(value);
            (y, format_number(value))
        })
        .collect();

    // X-axis labels (every 10 days)
    let total_days = historical.len() + forecast.len();
    let x_ticks: Vec<_> = (0..=total_days)
        .step_by(10)
        .map(|day| {
            let x = x_scale(day);
            let label = if day < historical.len() {
                format!("-{}", historical.len() - day)
            } else {
                format!("+{}", day - historical.len())
            };
            (x, label)
        })
        .collect();

    view! {
        <div class="card forecast-card">
            <div class="card-header">
                <h3 class="card-title">"Token Usage Forecast"</h3>
                <span class="forecast-confidence">
                    {format!("Confidence: {:.1}%", confidence * 100.0)}
                </span>
            </div>
            <div class="card-body">
                <svg
                    viewBox={format!("0 0 {} {}", CHART_WIDTH, CHART_HEIGHT)}
                    class="forecast-chart"
                    style="width: 100%; height: auto;"
                >
                    // Y-axis
                    <line
                        x1={MARGIN_LEFT.to_string()}
                        y1={MARGIN_TOP.to_string()}
                        x2={MARGIN_LEFT.to_string()}
                        y2={(CHART_HEIGHT - MARGIN_BOTTOM).to_string()}
                        stroke="var(--border-color)"
                        stroke-width="2"
                    />

                    // X-axis
                    <line
                        x1={MARGIN_LEFT.to_string()}
                        y1={(CHART_HEIGHT - MARGIN_BOTTOM).to_string()}
                        x2={(CHART_WIDTH - MARGIN_RIGHT).to_string()}
                        y2={(CHART_HEIGHT - MARGIN_BOTTOM).to_string()}
                        stroke="var(--border-color)"
                        stroke-width="2"
                    />

                    // Y-axis ticks and labels
                    {y_ticks.into_iter().map(|(y, label)| {
                        view! {
                            <>
                                <line
                                    x1={(MARGIN_LEFT - 5.0).to_string()}
                                    y1={y.to_string()}
                                    x2={MARGIN_LEFT.to_string()}
                                    y2={y.to_string()}
                                    stroke="var(--border-color)"
                                    stroke-width="1"
                                />
                                <text
                                    x={(MARGIN_LEFT - 10.0).to_string()}
                                    y={y.to_string()}
                                    text-anchor="end"
                                    alignment-baseline="middle"
                                    fill="var(--text-secondary)"
                                    font-size="12"
                                >
                                    {label}
                                </text>
                            </>
                        }
                    }).collect::<Vec<_>>()}

                    // X-axis ticks and labels
                    {x_ticks.into_iter().map(|(x, label)| {
                        view! {
                            <>
                                <line
                                    x1={x.to_string()}
                                    y1={(CHART_HEIGHT - MARGIN_BOTTOM).to_string()}
                                    x2={x.to_string()}
                                    y2={(CHART_HEIGHT - MARGIN_BOTTOM + 5.0).to_string()}
                                    stroke="var(--border-color)"
                                    stroke-width="1"
                                />
                                <text
                                    x={x.to_string()}
                                    y={(CHART_HEIGHT - MARGIN_BOTTOM + 20.0).to_string()}
                                    text-anchor="middle"
                                    fill="var(--text-secondary)"
                                    font-size="12"
                                >
                                    {label}
                                </text>
                            </>
                        }
                    }).collect::<Vec<_>>()}

                    // Budget line (if configured)
                    {if let Some(by) = budget_y {
                        view! {
                            <line
                                x1={MARGIN_LEFT.to_string()}
                                y1={by.to_string()}
                                x2={(CHART_WIDTH - MARGIN_RIGHT).to_string()}
                                y2={by.to_string()}
                                stroke="var(--color-danger)"
                                stroke-width="2"
                                stroke-dasharray="8,4"
                            />
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}

                    // Historical path (blue solid)
                    {if !historical_path.is_empty() {
                        view! {
                            <path
                                d={historical_path}
                                fill="none"
                                stroke="var(--accent-primary)"
                                stroke-width="3"
                            />
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}

                    // Forecast path (orange dashed)
                    {if !forecast_path.is_empty() {
                        view! {
                            <path
                                d={forecast_path}
                                fill="none"
                                stroke="var(--accent-secondary)"
                                stroke-width="3"
                                stroke-dasharray="8,4"
                            />
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}

                    // Legend
                    <g transform="translate(100, 25)">
                        <rect x="0" y="0" width="20" height="3" fill="var(--accent-primary)" />
                        <text x="25" y="5" fill="var(--text-secondary)" font-size="12">"Historical"</text>

                        <rect x="100" y="0" width="20" height="3" fill="var(--accent-secondary)" />
                        <rect x="100" y="0" width="20" height="3" fill="none" stroke="var(--accent-secondary)" stroke-width="1" stroke-dasharray="4,2" />
                        <text x="125" y="5" fill="var(--text-secondary)" font-size="12">"Forecast"</text>

                        {if budget.is_some() {
                            view! {
                                <>
                                    <rect x="200" y="0" width="20" height="3" fill="var(--color-danger)" />
                                    <rect x="200" y="0" width="20" height="3" fill="none" stroke="var(--color-danger)" stroke-width="1" stroke-dasharray="4,2" />
                                    <text x="225" y="5" fill="var(--text-secondary)" font-size="12">"Budget Limit"</text>
                                </>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                    </g>
                </svg>
            </div>
        </div>
    }
}
