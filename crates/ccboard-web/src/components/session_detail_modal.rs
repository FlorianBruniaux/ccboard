//! Session detail modal component

use crate::api::SessionData;
use leptos::prelude::*;
use leptos::web_sys::window;

/// Session detail modal
#[component]
pub fn SessionDetailModal(
    session: SessionData,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let session_id = session.id.clone();
    let copy_id = move || {
        if let Some(window) = window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&session_id);
        }
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close()>
            <div
                class="modal-content session-detail-modal"
                on:click=move |e| e.stop_propagation()
            >
                <div class="modal-header">
                    <h2>"Session Details"</h2>
                    <button class="modal-close" on:click=move |_| on_close()>
                        "×"
                    </button>
                </div>

                <div class="modal-body">
                    <div class="detail-section">
                        <h3>"Metadata"</h3>
                        <div class="detail-grid">
                            <div class="detail-item">
                                <span class="detail-label">"ID:"</span>
                                <span class="detail-value">
                                    {session.id.clone()}
                                    <button
                                        class="btn-icon"
                                        on:click=move |_| copy_id()
                                        title="Copy to clipboard"
                                    >
                                        "📋"
                                    </button>
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Project:"</span>
                                <span class="detail-value">{session.project.clone()}</span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Model:"</span>
                                <span class="detail-value">{format_model(&session.model)}</span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Date:"</span>
                                <span class="detail-value">{format_date(&session.date)}</span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Duration:"</span>
                                <span class="detail-value">
                                    {format_duration(session.duration_seconds)}
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Status:"</span>
                                <span class="detail-value">
                                    <span class="badge badge-success">
                                        {session.status.clone()}
                                    </span>
                                </span>
                            </div>
                        </div>
                    </div>

                    <div class="detail-section">
                        <h3>"Token Breakdown"</h3>
                        <div class="detail-grid">
                            <div class="detail-item">
                                <span class="detail-label">"Total:"</span>
                                <span class="detail-value">
                                    {session.tokens.to_string()}
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Input:"</span>
                                <span class="detail-value">
                                    {session.input_tokens.to_string()}
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Output:"</span>
                                <span class="detail-value">
                                    {session.output_tokens.to_string()}
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Cache Creation:"</span>
                                <span class="detail-value">
                                    {session.cache_creation_tokens.to_string()}
                                </span>
                            </div>
                            <div class="detail-item">
                                <span class="detail-label">"Cache Read:"</span>
                                <span class="detail-value">
                                    {session.cache_read_tokens.to_string()}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div class="detail-section">
                        <h3>"Cost Calculation"</h3>
                        <div class="detail-item">
                            <span class="detail-label">"Total Cost:"</span>
                                            <span class="detail-value cost-highlight">
                                                {format!("${:.4}", session.cost)}
                                            </span>
                                        </div>
                                    </div>

                                    <div class="detail-section">
                                        <h3>"Message Summary"</h3>
                                        <div class="detail-item">
                                            <span class="detail-label">"Message Count:"</span>
                                            <span class="detail-value">{session.messages.to_string()}</span>
                                        </div>
                                        {session
                                            .preview
                                            .clone()
                                            .map(|preview| {
                                                view! {
                                                    <div class="detail-item preview-item">
                                                        <span class="detail-label">"Preview:"</span>
                                                        <span class="detail-value preview-text">
                                                            {preview}
                                                        </span>
                                                    </div>
                                                }
                                            })}

                                    </div>

                                    <div class="detail-section">
                                        <h3>"TUI Integration"</h3>
                                        <p class="hint">
                                            "To view full session details with timeline visualization:"
                                        </p>
                                        <code class="command-example">
                                            {format!("ccboard info {}", session.id)}
                                        </code>
                                    </div>
                                </div>

                                <div class="modal-footer">
                                    <button class="btn btn-secondary" on:click=move |_| on_close()>
                                        "Close (Esc)"
                                    </button>
                </div>
            </div>
        </div>
    }
}

fn format_date(date: &Option<String>) -> String {
    date.as_ref()
        .and_then(|d| {
            chrono::DateTime::parse_from_rfc3339(d)
                .ok()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_model(model: &str) -> String {
    if model.contains("sonnet") {
        "Claude Sonnet 4.5".to_string()
    } else if model.contains("opus") {
        "Claude Opus 4".to_string()
    } else if model.contains("haiku") {
        "Claude Haiku 4".to_string()
    } else {
        model.to_string()
    }
}

fn format_duration(duration_seconds: Option<u64>) -> String {
    duration_seconds
        .map(|secs| {
            let hours = secs / 3600;
            let minutes = (secs % 3600) / 60;
            let seconds = secs % 60;

            if hours > 0 {
                format!("{}h {}m {}s", hours, minutes, seconds)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds)
            } else {
                format!("{}s", seconds)
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}
