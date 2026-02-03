//! ccboard-core - Core library for ccboard
//!
//! Provides parsers, models, store, and file watcher for Claude Code data.

pub mod cache;
pub mod error;
pub mod event;
pub mod export;
pub mod models;
pub mod parsers;
pub mod store;
pub mod watcher;

pub use error::{CoreError, DegradedState, LoadReport};
pub use event::{DataEvent, EventBus};
pub use export::{export_billing_blocks_to_csv, export_sessions_to_csv, export_sessions_to_json};
pub use store::DataStore;
pub use watcher::FileWatcher;
