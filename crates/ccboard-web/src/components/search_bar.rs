//! Search bar with filters for sessions

use leptos::prelude::*;

/// Search bar with filters
#[component]
pub fn SearchBar(
    search: ReadSignal<String>,
    set_search: WriteSignal<String>,
    project_filter: ReadSignal<Option<String>>,
    set_project_filter: WriteSignal<Option<String>>,
    model_filter: ReadSignal<Option<String>>,
    set_model_filter: WriteSignal<Option<String>>,
    date_filter: ReadSignal<Option<String>>,
    set_date_filter: WriteSignal<Option<String>>,
    available_projects: Signal<Vec<String>>,
) -> impl IntoView {
    let clear_filters = move |_| {
        set_search.set(String::new());
        set_project_filter.set(None);
        set_model_filter.set(None);
        set_date_filter.set(None);
    };

    view! {
        <div class="search-bar">
            <div class="search-input-group">
                <input
                    type="text"
                    class="search-input"
                    placeholder="Search sessions..."
                    prop:value=move || search.get()
                    on:input=move |e| { set_search.set(event_target_value(&e)) }
                />
            </div>

            <div class="filter-group">
                <select
                    class="filter-select"
                    on:change=move |e| {
                        let value = event_target_value(&e);
                        set_project_filter
                            .set(if value.is_empty() { None } else { Some(value) })
                    }
                >

                    <option value="">"All Projects"</option>
                    {move || {
                        available_projects
                            .get()
                            .into_iter()
                            .map(|project| {
                                let value = project.clone();
                                let label = project.clone();
                                view! {
                                    <option value=value>
                                        {label}
                                    </option>
                                }
                            })
                            .collect_view()
                    }}

                </select>

                <select
                    class="filter-select"
                    on:change=move |e| {
                        let value = event_target_value(&e);
                        set_model_filter.set(if value.is_empty() { None } else { Some(value) })
                    }
                >

                    <option value="">"All Models"</option>
                    <option value="sonnet">"Sonnet 4.5"</option>
                    <option value="opus">"Opus 4"</option>
                    <option value="haiku">"Haiku 4"</option>
                </select>

                <select
                    class="filter-select"
                    on:change=move |e| {
                        let value = event_target_value(&e);
                        set_date_filter.set(if value.is_empty() { None } else { Some(value) })
                    }
                >

                    <option value="">"All Time"</option>
                    <option value="7d">"Last 7 Days"</option>
                    <option value="30d">"Last 30 Days"</option>
                    <option value="90d">"Last 90 Days"</option>
                </select>

                <button class="btn btn-secondary" on:click=clear_filters>
                    "Clear Filters"
                </button>
            </div>
        </div>
    }
}
