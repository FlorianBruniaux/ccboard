//! Caching layer for ccboard-core
//!
//! Provides SQLite-based metadata caching for 90% startup speedup.

pub mod metadata_cache;

pub use metadata_cache::{CacheStats, MetadataCache};
