//! Toast notification system

use leptos::prelude::*;
use std::time::Duration;

/// Toast notification type (determines styling)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastType {
    fn icon(&self) -> &'static str {
        match self {
            ToastType::Info => "ℹ️",
            ToastType::Success => "✅",
            ToastType::Warning => "⚠️",
            ToastType::Error => "❌",
        }
    }

    fn class(&self) -> &'static str {
        match self {
            ToastType::Info => "toast-info",
            ToastType::Success => "toast-success",
            ToastType::Warning => "toast-warning",
            ToastType::Error => "toast-error",
        }
    }
}

/// Single toast notification
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
    pub auto_dismiss_ms: Option<u64>,
}

impl Toast {
    pub fn new(id: usize, message: String, toast_type: ToastType) -> Self {
        Self {
            id,
            message,
            toast_type,
            auto_dismiss_ms: Some(3000), // 3 seconds default
        }
    }

    pub fn info(id: usize, message: String) -> Self {
        Self::new(id, message, ToastType::Info)
    }

    pub fn success(id: usize, message: String) -> Self {
        Self::new(id, message, ToastType::Success)
    }

    pub fn warning(id: usize, message: String) -> Self {
        Self::new(id, message, ToastType::Warning)
    }

    pub fn error(id: usize, message: String) -> Self {
        let mut toast = Self::new(id, message, ToastType::Error);
        toast.auto_dismiss_ms = Some(5000); // Errors stay longer
        toast
    }
}

/// Toast context for managing global toast state
#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<Toast>>,
    next_id: RwSignal<usize>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(0),
        }
    }

    /// Add a toast notification
    pub fn show(&self, message: String, toast_type: ToastType) {
        let id = self.next_id.get();
        self.next_id.update(|n| *n += 1);

        let toast = Toast::new(id, message, toast_type);
        let auto_dismiss_ms = toast.auto_dismiss_ms;

        self.toasts.update(|toasts| {
            toasts.push(toast.clone());
        });

        // Auto-dismiss after timeout
        if let Some(ms) = auto_dismiss_ms {
            let toasts = self.toasts;
            set_timeout(
                move || {
                    toasts.update(|toasts| {
                        toasts.retain(|t| t.id != id);
                    });
                },
                Duration::from_millis(ms),
            );
        }
    }

    /// Show info toast
    pub fn info(&self, message: String) {
        self.show(message, ToastType::Info);
    }

    /// Show success toast
    pub fn success(&self, message: String) {
        self.show(message, ToastType::Success);
    }

    /// Show warning toast
    pub fn warning(&self, message: String) {
        self.show(message, ToastType::Warning);
    }

    /// Show error toast
    pub fn error(&self, message: String) {
        self.show(message, ToastType::Error);
    }

    /// Manually dismiss a toast
    pub fn dismiss(&self, id: usize) {
        self.toasts.update(|toasts| {
            toasts.retain(|t| t.id != id);
        });
    }

    /// Get current toasts
    pub fn toasts(&self) -> Vec<Toast> {
        self.toasts.get()
    }
}

impl Default for ToastContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Toast provider component (wraps app root)
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toast_context = ToastContext::new();

    provide_context(toast_context);

    view! {
        {children()}
        <ToastContainer />
    }
}

/// Toast container component (renders all active toasts)
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();

    view! {
        <div class="toast-container">
            <For
                each=move || toast_context.toasts()
                key=|toast| toast.id
                children=move |toast| {
                    view! { <ToastItem toast=toast /> }
                }
            />
        </div>
    }
}

/// Individual toast item component
#[component]
fn ToastItem(toast: Toast) -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();
    let id = toast.id;
    let message = toast.message.clone();
    let icon = toast.toast_type.icon();
    let class = toast.toast_type.class();

    view! {
        <div class=format!("toast {}", class)>
            <div class="toast-content">
                <span class="toast-icon">{icon}</span>
                <span class="toast-message">{message}</span>
            </div>
            <button
                class="toast-close"
                on:click=move |_| toast_context.dismiss(id)
                aria-label="Dismiss"
            >
                "×"
            </button>
        </div>
    }
}

/// Hook to access toast context
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}
