//! Page components

mod agents;
mod analytics;
mod config;
mod costs;
mod dashboard;
mod history;
mod hooks;
mod mcp;
mod plugins;
mod sessions;
mod task_graph;

pub use agents::Agents;
pub use analytics::Analytics;
pub use config::Config;
pub use costs::Costs;
pub use dashboard::Dashboard;
pub use history::History;
pub use hooks::Hooks;
pub use mcp::Mcp;
pub use plugins::PluginsPage;
pub use sessions::Sessions;
pub use task_graph::TaskGraphPage;
