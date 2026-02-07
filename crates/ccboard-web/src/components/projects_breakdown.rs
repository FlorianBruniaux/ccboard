//! Projects breakdown horizontal bar chart

use crate::api::{ProjectCost, format_cost};
use leptos::prelude::*;

/// Projects breakdown component
#[component]
pub fn ProjectsBreakdown(
    /// Top 5 projects by cost
    projects: Vec<ProjectCost>,
) -> impl IntoView {
    // Color palette for bars (cycling through accent colors)
    let colors = vec![
        "--accent-primary",
        "--accent-secondary",
        "--accent-tertiary",
        "--color-info",
        "--color-warning",
    ];

    view! {
        <div class="card projects-card">
            <div class="card-header">
                <h3 class="card-title">"Top Projects by Cost"</h3>
            </div>
            <div class="card-body">
                {if projects.is_empty() {
                    view! {
                        <p class="hint">"No project data available"</p>
                    }.into_any()
                } else {
                    view! {
                        <div class="projects-breakdown">
                            {projects.into_iter().enumerate().map(|(i, project)| {
                                let color = colors.get(i % colors.len()).unwrap_or(&"--accent-primary");
                                let project_name = if project.project.is_empty() {
                                    "Unknown Project".to_string()
                                } else {
                                    project.project.split('/').last().unwrap_or(&project.project).to_string()
                                };

                                view! {
                                    <div class="project-item">
                                        <div class="project-header">
                                            <span class="project-name">{project_name}</span>
                                            <span class="project-cost">
                                                {format_cost(project.cost)}
                                                " ("{format!("{:.1}%", project.percentage)}")"
                                            </span>
                                        </div>
                                        <div class="project-bar">
                                            <div
                                                class="project-fill"
                                                style={format!(
                                                    "width: {}%; background-color: var({});",
                                                    project.percentage,
                                                    color
                                                )}
                                            ></div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
