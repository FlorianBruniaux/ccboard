//! TUI tab implementations

pub mod agents;
pub mod analytics;
pub mod config;
pub mod conversation;
pub mod costs;
pub mod dashboard;
pub mod history;
pub mod hooks;
pub mod mcp;
pub mod plugins;
pub mod sessions;

pub use agents::AgentsTab;
pub use analytics::AnalyticsTab;
pub use config::ConfigTab;
pub use conversation::ConversationTab;
pub use costs::CostsTab;
pub use dashboard::DashboardTab;
pub use history::HistoryTab;
pub use hooks::HooksTab;
pub use mcp::McpTab;
pub use plugins::PluginsTab;
pub use sessions::SessionsTab;
