//! Caching layer for ccboard-core
//!
//! Provides SQLite-based metadata caching for 90% startup speedup.

pub mod insights_db;
pub mod metadata_cache;

pub use insights_db::InsightsDb;
pub use metadata_cache::{
    ActivityCacheStats, AggregateStats, CacheStats, MetadataCache, SearchResult, StoredAlert,
};
