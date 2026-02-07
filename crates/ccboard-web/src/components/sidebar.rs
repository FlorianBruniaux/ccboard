//! Sidebar navigation component

use leptos::prelude::*;
use leptos_router::components::A;

/// Navigation item definition
struct NavItem {
    path: &'static str,
    label: &'static str,
    icon: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        path: "/",
        label: "Dashboard",
        icon: "📊",
    },
    NavItem {
        path: "/sessions",
        label: "Sessions",
        icon: "💬",
    },
    NavItem {
        path: "/analytics",
        label: "Analytics",
        icon: "📈",
    },
    NavItem {
        path: "/config",
        label: "Config",
        icon: "⚙️",
    },
    NavItem {
        path: "/history",
        label: "History",
        icon: "🕒",
    },
];

/// Sidebar with navigation menu
#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="sidebar">
            <nav class="nav">
                <ul class="nav-list">
                    {NAV_ITEMS.iter().map(|item| {
                        view! {
                            <li class="nav-item">
                                <A href=item.path attr:class="nav-link">
                                    <span class="nav-icon">{item.icon}</span>
                                    <span class="nav-label">{item.label}</span>
                                </A>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </nav>
        </aside>
    }
}
