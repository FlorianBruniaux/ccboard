//! Main Leptos App component with SPA router

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::{Header, Sidebar};
use crate::pages::{Analytics, Dashboard, Sessions};

/// Main App component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="app">
                <Header />
                <div class="layout">
                    <Sidebar />
                    <main class="content">
                        <Routes fallback=|| "Not found">
                            <Route path=path!("/") view=Dashboard />
                            <Route path=path!("/sessions") view=Sessions />
                            <Route path=path!("/analytics") view=Analytics />
                            <Route path=path!("/config") view=|| view! { <div class="page-stub">"Config - Coming Soon"</div> } />
                            <Route path=path!("/history") view=|| view! { <div class="page-stub">"History - Coming Soon"</div> } />
                        </Routes>
                    </main>
                </div>
            </div>
        </Router>
    }
}
