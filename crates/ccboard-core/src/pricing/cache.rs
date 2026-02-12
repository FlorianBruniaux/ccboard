//! Pricing cache management
//!
//! Stores fetched pricing data in `~/.cache/ccboard/pricing.json` with TTL.

use super::litellm::{CachedPricing, LITELLM_PRICING_URL};
use crate::pricing::ModelPricing;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cache expiration duration (7 days)
const CACHE_TTL_DAYS: i64 = 7;

/// Get cache file path
pub fn cache_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .context("Could not determine cache directory")?
        .join("ccboard");

    std::fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir.join("pricing.json"))
}

/// Load pricing from cache
pub fn load_cached_pricing() -> Result<Option<HashMap<String, ModelPricing>>> {
    let path = cache_path()?;

    if !path.exists() {
        tracing::debug!("No pricing cache found at {}", path.display());
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read pricing cache: {}", path.display()))?;

    let cached: CachedPricing = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse pricing cache: {}", path.display()))?;

    // Check if cache is expired
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(cached.last_updated);

    if age.num_days() > CACHE_TTL_DAYS {
        tracing::info!(
            "Pricing cache expired ({} days old, TTL: {} days)",
            age.num_days(),
            CACHE_TTL_DAYS
        );
        return Ok(None);
    }

    tracing::info!(
        "Loaded {} model prices from cache ({} days old)",
        cached.models.len(),
        age.num_days()
    );

    Ok(Some(cached.models))
}

/// Save pricing to cache
pub fn save_pricing_cache(models: HashMap<String, ModelPricing>) -> Result<()> {
    let path = cache_path()?;

    let cached = CachedPricing {
        last_updated: chrono::Utc::now(),
        models,
        source: LITELLM_PRICING_URL.to_string(),
    };

    let json = serde_json::to_string_pretty(&cached)
        .context("Failed to serialize pricing cache")?;

    std::fs::write(&path, json)
        .with_context(|| format!("Failed to write pricing cache: {}", path.display()))?;

    tracing::info!("Saved {} model prices to cache", cached.models.len());

    Ok(())
}

/// Clear pricing cache
pub fn clear_pricing_cache() -> Result<()> {
    let path = cache_path()?;

    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove pricing cache: {}", path.display()))?;
        tracing::info!("Cleared pricing cache");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_path() {
        let path = cache_path().unwrap();
        assert!(path.to_string_lossy().contains("ccboard"));
        assert!(path.to_string_lossy().ends_with("pricing.json"));
    }
}
