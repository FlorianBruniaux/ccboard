//! Dashboard page component

use leptos::prelude::*;

/// Dashboard page - main overview
#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="page dashboard-page">
            <h2>"Dashboard"</h2>
            <div class="page-content">
                <p>"Dashboard - Coming Soon"</p>
                <p class="hint">"This will display stats overview, recent sessions, and quick actions."</p>
            </div>
        </div>
    }
}
