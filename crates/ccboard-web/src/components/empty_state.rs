//! Empty state component for coming soon pages

use leptos::prelude::*;
use leptos_router::components::A;

/// Empty state component for pages under development
#[component]
pub fn EmptyState(
    /// Page title (e.g., "Config", "History")
    title: &'static str,
    /// Description of what this page will do
    description: &'static str,
    /// Optional workaround suggestion
    #[prop(optional)]
    workaround: Option<&'static str>,
    /// Optional estimated timeline
    #[prop(optional)]
    timeline: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state-icon">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="64"
                    height="64"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    // Construction icon (lucide-construction)
                    <rect x="2" y="6" width="20" height="8" rx="1"/>
                    <path d="M17 14v7"/>
                    <path d="M7 14v7"/>
                    <path d="M17 3v3"/>
                    <path d="M7 3v3"/>
                    <path d="M10 14 2.3 6.3"/>
                    <path d="m14 6 7.7 7.7"/>
                    <path d="m8 6 8 8"/>
                </svg>
            </div>
            <h2 class="empty-state-title">{title} " - In Development"</h2>
            <p class="empty-state-description">{description}</p>

            {workaround.map(|w| {
                view! {
                    <div class="empty-state-workaround">
                        <strong>"Current workaround:"</strong>
                        " "
                        {w}
                    </div>
                }
            })}

            {timeline.map(|t| {
                view! {
                    <div class="empty-state-timeline">
                        <strong>"Expected:"</strong>
                        " "
                        {t}
                    </div>
                }
            })}

            <div class="empty-state-actions">
                <A href="/" attr:class="btn btn-primary">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="m12 19-7-7 7-7"/>
                        <path d="M19 12H5"/>
                    </svg>
                    " Back to Dashboard"
                </A>
            </div>
        </div>
    }
}
