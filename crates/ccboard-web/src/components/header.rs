//! Header component

use leptos::prelude::*;

/// Header with logo and subtitle
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="header-content">
                <h1 class="logo">"ccboard"</h1>
                <p class="subtitle">"Claude Code Dashboard"</p>
            </div>
        </header>
    }
}
