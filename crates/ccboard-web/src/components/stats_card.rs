//! Stats card component for dashboard

use leptos::prelude::*;

/// Card color variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardColor {
    Default,
    Green,
    Yellow,
    Red,
}

impl CardColor {
    fn to_class(&self) -> &'static str {
        match self {
            CardColor::Default => "",
            CardColor::Green => "card-green",
            CardColor::Yellow => "card-yellow",
            CardColor::Red => "card-red",
        }
    }
}

/// StatsCard component - displays a single metric with icon and label
#[component]
pub fn StatsCard(
    /// Card label (e.g., "Total Sessions")
    label: String,
    /// Card value (formatted, e.g., "1.2K")
    value: String,
    /// Icon emoji (e.g., "📊")
    icon: String,
    /// Color variant for status indication
    #[prop(default = CardColor::Default)]
    color: CardColor,
    /// Optional click handler for interactive cards
    #[prop(optional)]
    on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    let color_class = color.to_class();
    let is_clickable = on_click.is_some();

    let click_class = if is_clickable {
        "stats-card-clickable"
    } else {
        ""
    };

    let handle_click = move |_| {
        if let Some(ref handler) = on_click {
            handler();
        }
    };

    view! {
        <div
            class=format!("card stats-card {} {}", color_class, click_class)
            on:click=handle_click
        >
            <div class="stats-card-icon">{icon}</div>
            <div class="stats-card-content">
                <div class="stats-card-label">{label}</div>
                <div class="stats-card-value">{value}</div>
            </div>
            {is_clickable.then(|| view! {
                <div class="stats-card-action-hint">"Click to explore →"</div>
            })}
        </div>
    }
}
