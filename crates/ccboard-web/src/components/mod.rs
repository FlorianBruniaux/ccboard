//! Leptos UI components

mod budget_status;
mod error_boundary;
mod forecast_chart;
mod header;
mod projects_breakdown;
mod search_bar;
mod session_detail_modal;
mod session_table;
mod sidebar;
mod sparkline;
mod stats_card;
mod toast;

pub use budget_status::BudgetStatus;
pub use error_boundary::{ErrorBoundary, ErrorFallback};
pub use forecast_chart::ForecastChart;
pub use header::Header;
pub use projects_breakdown::ProjectsBreakdown;
pub use search_bar::SearchBar;
pub use session_detail_modal::SessionDetailModal;
pub use session_table::{SessionTable, SortColumn, SortDirection};
pub use sidebar::Sidebar;
pub use sparkline::Sparkline;
pub use stats_card::{CardColor, StatsCard};
pub use toast::{Toast, ToastContext, ToastProvider, ToastType, use_toast};
