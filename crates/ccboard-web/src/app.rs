//! Main Leptos App component with SPA router

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::{EmptyState, Header, Sidebar, ToastProvider};
use crate::pages::{Analytics, Dashboard, Sessions};

/// Main App component
#[component]
pub fn App() -> impl IntoView {
    // Mobile sidebar state
    let (sidebar_open, set_sidebar_open) = signal(false);

    view! {
        <ToastProvider>
            <Router>
                <div class="app">
                    <Header sidebar_open set_sidebar_open />
                    <div class="layout">
                        <Sidebar sidebar_open set_sidebar_open />
                        <main class="content">
                            <Routes fallback=|| "Not found">
                                <Route path=path!("/") view=Dashboard />
                                <Route path=path!("/sessions") view=Sessions />
                                <Route path=path!("/analytics") view=Analytics />
                                <Route
                                    path=path!("/config")
                                    view=|| view! {
                                        <EmptyState
                                            title="Config"
                                            description="Configuration management interface is under development. View and edit Claude Code settings across global, project, and local levels with merge visualization."
                                            workaround="Edit ~/.claude/settings.json directly"
                                            timeline="Q2 2026"
                                        />
                                    }
                                />
                                <Route
                                    path=path!("/history")
                                    view=|| view! {
                                        <EmptyState
                                            title="History"
                                            description="Session history timeline with filters and search. Track your Claude Code usage patterns over time with detailed analytics."
                                            workaround="Use Sessions Explorer with date filters"
                                            timeline="Q2 2026"
                                        />
                                    }
                                />
                            </Routes>
                        </main>
                    </div>
                </div>
            </Router>
        </ToastProvider>
    }
}
