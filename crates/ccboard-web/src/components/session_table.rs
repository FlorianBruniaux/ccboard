//! Session table component with sorting

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Session data from API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    pub id: String,
    pub date: Option<String>,
    pub project: String,
    pub model: String,
    pub messages: u64,
    pub tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost: f64,
    pub status: String,
    pub first_timestamp: Option<String>,
    pub duration_seconds: Option<u64>,
    pub preview: Option<String>,
}

/// Sort column
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Date,
    Project,
    Model,
    Messages,
    Tokens,
    Cost,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Session table component
#[component]
pub fn SessionTable(
    sessions: Signal<Option<Vec<SessionData>>>,
    on_row_click: WriteSignal<Option<SessionData>>,
) -> impl IntoView {
    let (sort_column, set_sort_column) = signal(SortColumn::Date);
    let (sort_direction, set_sort_direction) = signal(SortDirection::Desc);
    let (current_page, set_current_page) = signal(0_usize);
    let page_size = 20;

    // Sort sessions
    let sorted_sessions = Memo::new(move |_| {
        sessions.get().map(|mut sessions| {
            match sort_column.get() {
                SortColumn::Date => {
                    sessions.sort_by(|a, b| {
                        let cmp = a.date.cmp(&b.date);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
                SortColumn::Project => {
                    sessions.sort_by(|a, b| {
                        let cmp = a.project.cmp(&b.project);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
                SortColumn::Model => {
                    sessions.sort_by(|a, b| {
                        let cmp = a.model.cmp(&b.model);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
                SortColumn::Messages => {
                    sessions.sort_by(|a, b| {
                        let cmp = a.messages.cmp(&b.messages);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
                SortColumn::Tokens => {
                    sessions.sort_by(|a, b| {
                        let cmp = a.tokens.cmp(&b.tokens);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
                SortColumn::Cost => {
                    sessions.sort_by(|a, b| {
                        let cmp = a
                            .cost
                            .partial_cmp(&b.cost)
                            .unwrap_or(std::cmp::Ordering::Equal);
                        if sort_direction.get() == SortDirection::Asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
            }
            sessions
        })
    });

    // Paginate sessions
    let paginated_sessions = Memo::new(move |_| {
        sorted_sessions.get().map(|sessions| {
            let start = current_page.get() * page_size;
            let end = (start + page_size).min(sessions.len());
            sessions[start..end].to_vec()
        })
    });

    let total_count = Memo::new(move |_| sorted_sessions.get().map(|s| s.len()).unwrap_or(0));
    let total_pages = Memo::new(move |_| (total_count.get() + page_size - 1) / page_size);

    let toggle_sort = move |column: SortColumn| {
        if sort_column.get() == column {
            // Toggle direction
            set_sort_direction.set(if sort_direction.get() == SortDirection::Asc {
                SortDirection::Desc
            } else {
                SortDirection::Asc
            });
        } else {
            // New column, default to descending
            set_sort_column.set(column);
            set_sort_direction.set(SortDirection::Desc);
        }
        // Reset to first page
        set_current_page.set(0);
    };

    let sort_indicator = move |column: SortColumn| {
        if sort_column.get() == column {
            if sort_direction.get() == SortDirection::Asc {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        }
    };

    view! {
        <div class="session-table-container">
            <div class="table-stats">
                {move || {
                    let count = total_count.get();
                    let start = current_page.get() * page_size + 1;
                    let end = ((current_page.get() + 1) * page_size).min(count);
                    format!("Showing {} - {} of {} sessions", start, end, count)
                }}
            </div>

            <table class="session-table">
                <thead>
                    <tr>
                        <th on:click=move |_| toggle_sort(SortColumn::Date)>
                            {"Date"}{move || sort_indicator(SortColumn::Date)}
                        </th>
                        <th on:click=move |_| toggle_sort(SortColumn::Project)>
                            {"Project"}{move || sort_indicator(SortColumn::Project)}
                        </th>
                        <th on:click=move |_| toggle_sort(SortColumn::Model)>
                            {"Model"}{move || sort_indicator(SortColumn::Model)}
                        </th>
                        <th on:click=move |_| toggle_sort(SortColumn::Messages)>
                            {"Messages"}{move || sort_indicator(SortColumn::Messages)}
                        </th>
                        <th on:click=move |_| toggle_sort(SortColumn::Tokens)>
                            {"Tokens"}{move || sort_indicator(SortColumn::Tokens)}
                        </th>
                        <th on:click=move |_| toggle_sort(SortColumn::Cost)>
                            {"Cost"}{move || sort_indicator(SortColumn::Cost)}
                        </th>
                        <th>{"Status"}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        paginated_sessions
                            .get()
                            .map(|sessions| {
                                sessions
                                    .into_iter()
                                    .map(|session| {
                                        let session_clone = session.clone();
                                        view! {
                                            <tr
                                                class="session-row"
                                                on:click=move |_| {
                                                    on_row_click.set(Some(session_clone.clone()))
                                                }
                                            >

                                                <td>{format_date(&session.date)}</td>
                                                <td>{session.project.clone()}</td>
                                                <td>{format_model(&session.model)}</td>
                                                <td>{session.messages.to_string()}</td>
                                                <td>{format_tokens(session.tokens)}</td>
                                                <td>{format!("${:.4}", session.cost)}</td>
                                                <td>
                                                    <span class="badge badge-success">
                                                        {session.status.clone()}
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()
                            })
                    }}
                </tbody>
            </table>

            <div class="pagination">
                <button
                    class="btn btn-secondary"
                    disabled=move || current_page.get() == 0
                    on:click=move |_| set_current_page.update(|p| *p = p.saturating_sub(1))
                >
                    {"← Previous"}
                </button>
                <span class="pagination-info">
                    {move || format!("Page {} of {}", current_page.get() + 1, total_pages.get())}
                </span>
                <button
                    class="btn btn-secondary"
                    disabled=move || current_page.get() >= total_pages.get() - 1
                    on:click=move |_| {
                        set_current_page
                            .update(|p| {
                                if *p < total_pages.get() - 1 {
                                    *p += 1
                                }
                            })
                    }
                >

                    {"Next →"}
                </button>
            </div>
        </div>
    }
}

fn format_date(date: &Option<String>) -> String {
    date.as_ref()
        .and_then(|d| {
            chrono::DateTime::parse_from_rfc3339(d)
                .ok()
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_model(model: &str) -> String {
    if model.contains("sonnet") {
        "Sonnet 4.5".to_string()
    } else if model.contains("opus") {
        "Opus 4".to_string()
    } else if model.contains("haiku") {
        "Haiku 4".to_string()
    } else {
        model.to_string()
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
