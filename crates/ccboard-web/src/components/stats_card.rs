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
) -> impl IntoView {
    let color_class = color.to_class();

    view! {
        <div class=format!("card stats-card {}", color_class)>
            <div class="stats-card-icon">{icon}</div>
            <div class="stats-card-content">
                <div class="stats-card-label">{label}</div>
                <div class="stats-card-value">{value}</div>
            </div>
        </div>
    }
}
