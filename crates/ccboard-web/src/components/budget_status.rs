//! Budget status card with progress visualization

use leptos::prelude::*;

/// Budget status card
#[component]
pub fn BudgetStatus(
    /// Tokens used this month
    used: u64,
    /// Optional budget limit
    budget: Option<u64>,
) -> impl IntoView {
    let (percentage, color_class) = if let Some(budget_limit) = budget {
        let pct = if budget_limit > 0 {
            (used as f64 / budget_limit as f64 * 100.0).min(100.0)
        } else {
            0.0
        };

        let color = if pct < 70.0 {
            "budget-ok" // Green
        } else if pct < 90.0 {
            "budget-warning" // Yellow
        } else {
            "budget-danger" // Red
        };

        (pct, color)
    } else {
        (0.0, "budget-none")
    };

    view! {
        <div class="card budget-card">
            <div class="card-header">
                <h3 class="card-title">"Budget Status"</h3>
            </div>
            <div class="card-body">
                {if let Some(budget_limit) = budget {
                    view! {
                        <div class="budget-status">
                            <div class="budget-text">
                                <span class="budget-label">"Used "</span>
                                <span class="budget-value">{crate::api::format_number(used)}</span>
                                <span class="budget-label">" of "</span>
                                <span class="budget-value">{crate::api::format_number(budget_limit)}</span>
                                <span class="budget-label">" tokens this month"</span>
                                <span class={format!("budget-percentage {}", color_class)}>
                                    " ("{format!("{:.1}%", percentage)}")"
                                </span>
                            </div>
                            <div class="budget-bar">
                                <div
                                    class={format!("budget-fill {}", color_class)}
                                    style={format!("width: {}%", percentage)}
                                ></div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="budget-status">
                            <p class="hint">"No budget configured"</p>
                            <p class="hint-sub">"Set a budget in settings to track usage limits"</p>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
