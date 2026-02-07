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
pub fn Sidebar(
    sidebar_open: ReadSignal<bool>,
    set_sidebar_open: WriteSignal<bool>,
) -> impl IntoView {
    // Close sidebar when clicking a link (mobile)
    let close_sidebar = move |_| {
        set_sidebar_open.set(false);
    };

    view! {
        <>
            // Backdrop overlay for mobile
            <Show when=move || sidebar_open.get()>
                <div
                    class="sidebar-backdrop"
                    on:click=move |_| set_sidebar_open.set(false)
                ></div>
            </Show>

            <aside class="sidebar" class:sidebar-open=move || sidebar_open.get()>
                <button
                    class="sidebar-close"
                    on:click=move |_| set_sidebar_open.set(false)
                    aria-label="Close sidebar"
                >
                    "✕"
                </button>

                <nav class="nav">
                    <ul class="nav-list">
                        {NAV_ITEMS.iter().map(|item| {
                            view! {
                                <li class="nav-item">
                                    <A href=item.path attr:class="nav-link" on:click=close_sidebar>
                                        <span class="nav-icon">{item.icon}</span>
                                        <span class="nav-label">{item.label}</span>
                                    </A>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </nav>
            </aside>
        </>
    }
}
