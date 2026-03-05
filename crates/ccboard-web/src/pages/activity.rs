//! Activity page - security audit of tool calls across sessions

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "";

// ─── API types ───────────────────────────────────────────────────────────────

/// A single security alert from the violations endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViolationInfo {
    pub session_id: String,
    pub timestamp: String,
    pub severity: String, // "Critical" | "Warning" | "Info"
    pub category: String, // "CredentialAccess" | "DestructiveCommand" | ...
    pub detail: String,
    #[serde(default)]
    pub action_hint: String,
}

/// Response from GET /api/activity/violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationsResponse {
    pub violations: Vec<ViolationInfo>,
    pub total: usize,
    pub displayed: usize,
    #[serde(default)]
    pub critical_count: usize,
    #[serde(default)]
    pub warning_count: usize,
    #[serde(default)]
    pub info_count: usize,
}

// ─── Fetch helper ─────────────────────────────────────────────────────────────

async fn fetch_violations(min_severity: Option<String>) -> Result<ViolationsResponse, String> {
    let url = match min_severity.as_deref() {
        Some(sev) => format!(
            "{}/api/activity/violations?limit=100&min_severity={}",
            API_BASE_URL, sev
        ),
        None => format!("{}/api/activity/violations?limit=100", API_BASE_URL),
    };

    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let resp: ViolationsResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(resp)
}

// ─── Sub-components ───────────────────────────────────────────────────────────

/// Severity badge with appropriate colour class
#[component]
fn SeverityBadge(severity: String) -> impl IntoView {
    let class = match severity.as_str() {
        "Critical" => "severity-badge severity-badge--critical",
        "Warning" => "severity-badge severity-badge--warning",
        _ => "severity-badge severity-badge--info",
    };
    view! { <span class=class>{severity}</span> }
}

/// Human-readable category label
fn category_label(cat: &str) -> &'static str {
    match cat {
        "CredentialAccess" => "Credential Access",
        "DestructiveCommand" => "Destructive Command",
        "ExternalExfil" => "External Fetch",
        "ScopeViolation" => "Scope Violation",
        "ForcePush" => "Force Push",
        _ => "Unknown",
    }
}

/// Icon for each category
fn category_icon(cat: &str) -> &'static str {
    match cat {
        "CredentialAccess" => "🔑",
        "DestructiveCommand" => "💣",
        "ExternalExfil" => "🌐",
        "ScopeViolation" => "📁",
        "ForcePush" => "⚡",
        _ => "⚠️",
    }
}

/// Summary stat card
#[component]
fn StatCard(label: &'static str, value: usize, class: &'static str) -> impl IntoView {
    view! {
        <div class=format!("stat-card {}", class)>
            <div class="stat-card__value">{value}</div>
            <div class="stat-card__label">{label}</div>
        </div>
    }
}

/// Single violation row
#[component]
fn ViolationRow(v: ViolationInfo) -> impl IntoView {
    // Truncate session_id to 8 chars for readability
    let short_session = v.session_id.chars().take(8).collect::<String>();

    // Format timestamp: keep only date + time, drop sub-seconds and timezone
    let ts = v.timestamp.get(..19).unwrap_or(&v.timestamp).to_string();

    let category_display = category_label(&v.category);
    let icon = category_icon(&v.category);
    let action_hint = v.action_hint.clone();
    let detail = v.detail.clone();
    let severity = v.severity.clone();

    // Show action hint as a togglable detail
    let (show_hint, set_show_hint) = signal(false);

    view! {
        <div class="violation-row">
            <div class="violation-row__main">
                <SeverityBadge severity=severity />
                <span class="violation-row__category">
                    {icon}" "{category_display}
                </span>
                <span class="violation-row__session" title=v.session_id.clone()>
                    {short_session}"…"
                </span>
                <span class="violation-row__detail">{detail}</span>
                <span class="violation-row__ts">{ts}</span>
                <button
                    class="violation-row__hint-btn"
                    on:click=move |_| set_show_hint.update(|v| *v = !*v)
                    title="Show remediation hint"
                >
                    "?"
                </button>
            </div>
            <Show when=move || show_hint.get()>
                <div class="violation-row__hint">
                    "💡 "{action_hint.clone()}
                </div>
            </Show>
        </div>
    }
}

// ─── Main page component ──────────────────────────────────────────────────────

/// Activity Security Audit page
///
/// Displays aggregated security alerts across all analysed sessions.
/// Uses polling (`/api/activity/violations`) — no SSE needed for this use case.
#[component]
pub fn ActivityPage() -> impl IntoView {
    // Severity filter: None = all, Some("Critical") = Critical only, etc.
    let severity_filter: RwSignal<Option<String>> = RwSignal::new(None);

    // Reactive resource — re-fetches when filter changes
    let violations_resource = LocalResource::new(move || {
        let filter = severity_filter.get();
        async move { fetch_violations(filter).await }
    });

    view! {
        <div class="page activity-page">
            <div class="page-header">
                <h1 class="page-title">"Security Audit"</h1>
                <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                    {move || {
                        violations_resource.get().map(|result| {
                            match *result {
                                Ok(ref r) => view! {
                                    <span class="page-subtitle">
                                        {format!("{} violation(s) found", r.total)}
                                    </span>
                                }.into_any(),
                                Err(_) => view! { <span></span> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Summary cards (only shown when data is loaded)
            <Suspense fallback=|| view! { <div></div> }>
                {move || {
                    violations_resource.get().map(|result| {
                        match result.as_ref() {
                            Ok(r) => view! {
                                <div class="activity-summary">
                                    <StatCard label="Critical" value=r.critical_count class="stat-card--critical" />
                                    <StatCard label="Warning" value=r.warning_count class="stat-card--warning" />
                                    <StatCard label="Info" value=r.info_count class="stat-card--info" />
                                </div>
                            }.into_any(),
                            Err(_) => view! { <div></div> }.into_any(),
                        }
                    })
                }}
            </Suspense>

            // Severity filter controls
            <div class="activity-filters">
                <span class="activity-filters__label">"Filter: "</span>
                {[
                    (None::<String>, "All"),
                    (Some("Critical".to_string()), "Critical only"),
                    (Some("Warning".to_string()), "Warning+"),
                ]
                .into_iter()
                .map(|(val, label)| {
                    let val_clone = val.clone();
                    let is_active = move || severity_filter.get() == val_clone;
                    let val_for_click = val.clone();
                    view! {
                        <button
                            class=move || {
                                if is_active() {
                                    "filter-btn filter-btn--active"
                                } else {
                                    "filter-btn"
                                }
                            }
                            on:click=move |_| severity_filter.set(val_for_click.clone())
                        >
                            {label}
                        </button>
                    }
                })
                .collect::<Vec<_>>()}
            </div>

            // Violations list
            <Suspense fallback=|| view! { <div class="loading">"Loading violations..."</div> }>
                {move || {
                    violations_resource.get().map(|result| {
                        match result.as_ref() {
                            Ok(response) => {
                                if response.violations.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <p>"No violations found."</p>
                                            <p class="empty-state__hint">
                                                "Sessions are analysed on demand — open a session detail in the Sessions tab to trigger analysis."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let violations = response.violations.clone();
                                    view! {
                                        <div class="violations-list">
                                            <div class="violations-list__header">
                                                <span class="violations-col violations-col--severity">"Severity"</span>
                                                <span class="violations-col violations-col--category">"Category"</span>
                                                <span class="violations-col violations-col--session">"Session"</span>
                                                <span class="violations-col violations-col--detail">"Detail"</span>
                                                <span class="violations-col violations-col--ts">"Time"</span>
                                                <span class="violations-col violations-col--hint"></span>
                                            </div>
                                            {violations.into_iter().map(|v| view! {
                                                <ViolationRow v=v />
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! {
                                <div class="error-state">
                                    <p>"Error loading violations: "{e.to_string()}</p>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
