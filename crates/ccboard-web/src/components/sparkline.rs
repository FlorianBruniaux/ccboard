//! Sparkline component for token usage visualization

use leptos::prelude::*;

/// Sparkline component - simple SVG line chart
#[component]
pub fn Sparkline(
    /// Data points for the sparkline (30 values for 30 days)
    data: Vec<u64>,
    /// Chart width in pixels
    #[prop(default = 800)]
    width: u32,
    /// Chart height in pixels
    #[prop(default = 100)]
    height: u32,
    /// Chart label
    #[prop(default = "Activity".to_string())]
    label: String,
) -> impl IntoView {
    if data.is_empty() {
        return view! {
            <div class="sparkline-container">
                <div class="sparkline-label">{label.clone()}</div>
                <div class="sparkline-empty">"No data available"</div>
            </div>
        }
        .into_any();
    }

    let max_value = *data.iter().max().unwrap_or(&1);
    let min_value = *data.iter().min().unwrap_or(&0);
    let range = if max_value > min_value {
        max_value - min_value
    } else {
        1
    };

    // Calculate SVG path points
    let points_len = data.len();
    let x_step = width as f64 / (points_len - 1).max(1) as f64;
    let y_scale = height as f64 / range as f64;

    let mut path_data = String::from("M ");
    for (i, value) in data.iter().enumerate() {
        let x = i as f64 * x_step;
        let y = height as f64 - ((*value - min_value) as f64 * y_scale);
        if i == 0 {
            path_data.push_str(&format!("{:.2},{:.2}", x, y));
        } else {
            path_data.push_str(&format!(" L {:.2},{:.2}", x, y));
        }
    }

    view! {
        <div class="sparkline-container">
            <div class="sparkline-label">{label}</div>
            <svg
                class="sparkline"
                width=width.to_string()
                height=height.to_string()
                viewBox=format!("0 0 {} {}", width, height)
            >
                <path
                    d=path_data
                    fill="none"
                    stroke="var(--accent-primary)"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
        </div>
    }
    .into_any()
}
