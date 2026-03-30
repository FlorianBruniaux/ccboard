//! Caching layer for ccboard-core
//!
//! Provides SQLite-based metadata caching for 90% startup speedup.

pub mod claude_mem_db;
pub mod insights_db;
pub mod metadata_cache;

pub use claude_mem_db::ClaudeMemDb;
pub use insights_db::InsightsDb;
pub use metadata_cache::{
    ActivityCacheStats, AggregateStats, CacheStats, MetadataCache, SearchResult, StoredAlert,
};
