//! Task graph page - dependency visualization

use crate::components::TaskDependencyGraph;
use leptos::prelude::*;

/// Task graph page component
#[component]
pub fn TaskGraphPage() -> impl IntoView {
    view! {
        <div class="page task-graph-page">
            <TaskDependencyGraph />
        </div>
    }
}
