//! TUI tab implementations

pub mod agents;
pub mod config;
pub mod costs;
pub mod dashboard;
pub mod history;
pub mod hooks;
pub mod sessions;

pub use agents::AgentsTab;
pub use config::ConfigTab;
pub use costs::CostsTab;
pub use dashboard::DashboardTab;
pub use history::HistoryTab;
pub use hooks::HooksTab;
pub use sessions::SessionsTab;
