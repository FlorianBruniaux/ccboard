//! Task dependency graph visualization component

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Task graph data structures matching backend API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGraphData {
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNode {
    pub id: String,
    pub label: String,
    pub phase: String,
    pub status: String,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEdge {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

/// External JavaScript function from d3-graph.js
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = renderTaskGraph)]
    fn render_task_graph(nodes: JsValue, edges: JsValue);
}

/// Fetch task graph data from backend API
async fn fetch_task_graph() -> Result<TaskGraphData, String> {
    let response = Request::get("/api/task-graph")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let data = response
        .json::<TaskGraphData>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(data)
}

/// Task dependency graph visualization component
#[component]
pub fn TaskDependencyGraph() -> impl IntoView {
    // Resource for loading graph data
    let graph_data = LocalResource::new(|| async move { fetch_task_graph().await });

    // Effect to render D3 graph when data is loaded
    Effect::new(move |_| {
        if let Some(Ok(data)) = graph_data.get().as_ref().map(|r| r.as_ref()) {
            // Convert to JsValue for JS interop
            if let Ok(nodes_js) = serde_wasm_bindgen::to_value(&data.nodes) {
                if let Ok(edges_js) = serde_wasm_bindgen::to_value(&data.edges) {
                    render_task_graph(nodes_js, edges_js);
                }
            }
        }
    });

    view! {
        <div class="task-graph-container">
            <div class="page-header">
                <h2>"Task Dependency Graph"</h2>
                <p class="subtitle">"Visualize task dependencies and execution order"</p>
            </div>

            <Suspense fallback=move || view! { <div class="loading">"Loading task graph..."</div> }>
                {move || match graph_data.get().as_ref().map(|r| r.as_ref()) {
                    Some(Ok(data)) => {
                        let node_count = data.nodes.len();
                        let edge_count = data.edges.len();

                        view! {
                            <div class="graph-content">
                                <div class="graph-stats">
                                    <div class="stat-item">
                                        <span class="stat-label">"Tasks: "</span>
                                        <span class="stat-value">{node_count}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">"Dependencies: "</span>
                                        <span class="stat-value">{edge_count}</span>
                                    </div>
                                </div>

                                <div class="graph-legend">
                                    <h3>"Legend"</h3>
                                    <div class="legend-items">
                                        <div class="legend-item">
                                            <div class="legend-color" style="background: #4CAF50;"></div>
                                            <span>"Complete"</span>
                                        </div>
                                        <div class="legend-item">
                                            <div class="legend-color" style="background: #FFC107;"></div>
                                            <span>"In Progress"</span>
                                        </div>
                                        <div class="legend-item">
                                            <div class="legend-color" style="background: #9E9E9E;"></div>
                                            <span>"Future"</span>
                                        </div>
                                    </div>
                                </div>

                                <div id="d3-graph" style="width: 100%; height: 600px; border: 1px solid #333; background: #1a1a1a; border-radius: 8px;"></div>

                                <div class="graph-instructions">
                                    <p>"💡 Tip: Drag nodes to rearrange. Zoom with mouse wheel. Pan by dragging background."</p>
                                </div>
                            </div>
                        }.into_any()
                    },
                    Some(Err(e)) => {
                        let err = e.clone();
                        view! {
                            <div class="error">
                                <h3>"Failed to load task graph"</h3>
                                <p>{err}</p>
                            </div>
                        }.into_any()
                    },
                    None => {
                        view! { <div class="loading">"Loading task graph..."</div> }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}
