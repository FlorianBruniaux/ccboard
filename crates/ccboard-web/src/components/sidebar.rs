//! Sidebar navigation component with inline Lucide-style SVG icons

use leptos::prelude::*;
use leptos_router::components::A;

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
                        <li class="nav-item">
                            <A href="/" attr:class="sidebar-link" on:click=close_sidebar>
                                <span class="sidebar-link-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <rect width="7" height="9" x="3" y="3" rx="1"/>
                                        <rect width="7" height="5" x="14" y="3" rx="1"/>
                                        <rect width="7" height="9" x="14" y="12" rx="1"/>
                                        <rect width="7" height="5" x="3" y="16" rx="1"/>
                                    </svg>
                                </span>
                                <span class="sidebar-link-label">"Dashboard"</span>
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/sessions" attr:class="sidebar-link" on:click=close_sidebar>
                                <span class="sidebar-link-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"/>
                                    </svg>
                                </span>
                                <span class="sidebar-link-label">"Sessions"</span>
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/analytics" attr:class="sidebar-link" on:click=close_sidebar>
                                <span class="sidebar-link-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <polyline points="22 7 13.5 15.5 8.5 10.5 2 17"/>
                                        <polyline points="16 7 22 7 22 13"/>
                                    </svg>
                                </span>
                                <span class="sidebar-link-label">"Analytics"</span>
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/config" attr:class="sidebar-link" on:click=close_sidebar>
                                <span class="sidebar-link-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
                                        <circle cx="12" cy="12" r="3"/>
                                    </svg>
                                </span>
                                <span class="sidebar-link-label">"Config"</span>
                            </A>
                        </li>
                        <li class="nav-item">
                            <A href="/history" attr:class="sidebar-link" on:click=close_sidebar>
                                <span class="sidebar-link-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <circle cx="12" cy="12" r="10"/>
                                        <polyline points="12 6 12 12 16 14"/>
                                    </svg>
                                </span>
                                <span class="sidebar-link-label">"History"</span>
                            </A>
                        </li>
                    </ul>
                </nav>
            </aside>
        </>
    }
}
