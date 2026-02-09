//! Error boundary component for graceful error handling

use leptos::prelude::*;

/// Error fallback component
///
/// Displays user-friendly error messages with retry functionality.
/// Use within Suspense error handling or as standalone error display.
///
/// # Example
/// ```ignore
/// view! {
///     <ErrorFallback
///         error="Failed to load data".to_string()
///         on_retry=Callback::new(move |_| window().location().reload().unwrap())
///     />
/// }
/// ```
#[component]
pub fn ErrorFallback(
    /// Error message to display
    #[prop(into)]
    error: String,
    /// Callback to retry/reset
    #[prop(into)]
    on_retry: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="error-boundary">
            <div class="error-boundary-content">
                <div class="error-boundary-icon">"⚠️"</div>
                <h2 class="error-boundary-title">"Error"</h2>
                <p class="error-boundary-message">{error}</p>
                <button
                    class="error-boundary-retry"
                    on:click=move |_| on_retry.run(())
                >
                    "Retry"
                </button>
            </div>
        </div>
    }
}

/// Simple error boundary wrapper using Show
///
/// Use Show component directly in your views for conditional error display.
/// This is a convenience re-export to make the pattern more discoverable.
///
/// # Example
/// ```ignore
/// let (error, set_error) = signal(None::<String>);
///
/// view! {
///     <Show
///         when=move || error.get().is_none()
///         fallback=move || {
///             view! {
///                 <ErrorFallback
///                     error=error.get().unwrap_or_default()
///                     on_retry=Callback::new(move |_| set_error.set(None))
///                 />
///             }
///         }
///     >
///         <MyComponent />
///     </Show>
/// }
/// ```
pub use leptos::prelude::Show as ErrorBoundary;
