//! Page components

mod activity;
mod agents;
mod analytics;
mod brain;
mod config;
mod costs;
mod dashboard;
mod history;
mod hooks;
mod mcp;
mod plugins;
mod search;
mod sessions;
mod task_graph;

pub use activity::ActivityPage;
pub use agents::Agents;
pub use analytics::Analytics;
pub use brain::Brain;
pub use config::Config;
pub use costs::Costs;
pub use dashboard::Dashboard;
pub use history::History;
pub use hooks::Hooks;
pub use mcp::Mcp;
pub use plugins::PluginsPage;
pub use search::SearchPage;
pub use sessions::Sessions;
pub use task_graph::TaskGraphPage;
