//! Header component

use leptos::prelude::*;

/// Header with logo, subtitle, and mobile hamburger menu
#[component]
pub fn Header(
    sidebar_open: ReadSignal<bool>,
    set_sidebar_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="header">
            <button
                class="hamburger"
                on:click=move |_| set_sidebar_open.update(|v| *v = !*v)
                aria-label="Toggle sidebar"
                aria-expanded=move || sidebar_open.get().to_string()
            >
                <span class="hamburger-icon">"☰"</span>
            </button>

            <div class="header-content">
                <h1 class="logo">"ccboard"</h1>
                <p class="subtitle">"Claude Code Dashboard"</p>
            </div>
        </header>
    }
}
