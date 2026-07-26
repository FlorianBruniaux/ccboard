//! Skills leaderboard page - skill invocation counts, cost, and reach across projects
//!
//! Table parity only (per the skill-usage-analytics design doc): no sparklines, no
//! graphs, no `--show-context`-style drilldown, no per-event SSE push. A `[retired]`
//! badge marks skills with invocation history but no matching on-disk definition,
//! recomputed fresh on every fetch (never persisted).

use crate::api::{format_cost, format_number};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "";

/// Mirrors the Axum handler's per-skill JSON shape (snake_case, matching backend
/// field names verbatim, same convention as `pages::plugins::PluginUsage`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLeaderboardEntry {
    pub skill: String,
    pub invocations: usize,
    pub projects: usize,
    pub total_tokens: u64,
    pub estimated_cost_usd: f64,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub retired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLeaderboardResponse {
    pub entries: Vec<SkillLeaderboardEntry>,
    pub total: usize,
}

async fn fetch_skill_leaderboard() -> Result<SkillLeaderboardResponse, String> {
    let url = format!("{}/api/skills/leaderboard", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch skill leaderboard: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Truncate an RFC3339 timestamp down to its date component for compact display.
fn format_date(ts: &Option<String>) -> String {
    match ts {
        Some(s) => s.get(..10).unwrap_or(s).to_string(),
        None => "—".to_string(),
    }
}

#[component]
fn SkillRow(entry: SkillLeaderboardEntry) -> impl IntoView {
    view! {
        <tr>
            <td>
                <span class="skills-table__name">
                    {entry.skill.clone()}
                    {entry.retired.then(|| view! {
                        <span
                            class="badge badge-warning"
                            title="Invoked in session history, but no longer defined on disk"
                        >
                            "retired"
                        </span>
                    })}
                </span>
            </td>
            <td class="skills-table__num">{entry.invocations}</td>
            <td class="skills-table__num">{entry.projects}</td>
            <td class="skills-table__num">{format_number(entry.total_tokens)}</td>
            <td class="skills-table__num skills-table__cost">{format_cost(entry.estimated_cost_usd)}</td>
            <td class="skills-table__timestamp">{format_date(&entry.first_seen)}</td>
            <td class="skills-table__timestamp">{format_date(&entry.last_seen)}</td>
        </tr>
    }
}

/// Skills leaderboard page component
#[component]
pub fn Skills() -> impl IntoView {
    let leaderboard = LocalResource::new(move || async move { fetch_skill_leaderboard().await });

    view! {
        <div class="page skills-page">
            <div class="page-header">
                <h1 class="page-title">"Skills Leaderboard"</h1>
                <span class="page-subtitle">"Invocation counts, reach, and estimated cost per skill"</span>
            </div>

            <Suspense fallback=move || view! { <div class="loading">"Loading skill leaderboard..."</div> }>
                {move || Suspend::new(async move {
                    match leaderboard.await {
                        Ok(response) => {
                            if response.entries.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <p>"No skill invocations recorded yet"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="table-container">
                                        <table class="skills-table">
                                            <thead>
                                                <tr>
                                                    <th>"Skill"</th>
                                                    <th class="skills-table__num">"Invocations"</th>
                                                    <th class="skills-table__num">"Projects"</th>
                                                    <th class="skills-table__num">"Tokens"</th>
                                                    <th class="skills-table__num">"Est. Cost"</th>
                                                    <th>"First Seen"</th>
                                                    <th>"Last Seen"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {response.entries.into_iter().map(|entry| {
                                                    view! { <SkillRow entry=entry /> }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(e) => {
                            view! {
                                <div class="error-state">
                                    <div class="error-state__icon">"⚠️"</div>
                                    <h2 class="error-state__title">"Error loading skill leaderboard"</h2>
                                    <p class="error-state__message">{e}</p>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
