//! ccboard-core - Core library for ccboard
//!
//! Provides parsers, models, store, and file watcher for Claude Code data.

pub mod error;
pub mod event;
pub mod models;
pub mod parsers;
pub mod store;
pub mod watcher;

pub use error::{CoreError, DegradedState, LoadReport};
pub use event::{DataEvent, EventBus};
pub use store::DataStore;
