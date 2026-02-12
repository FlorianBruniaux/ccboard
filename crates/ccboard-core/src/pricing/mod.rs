//! Pricing calculations for Claude models
//!
//! This module provides accurate pricing for Claude models with two pricing sources:
//! 1. **Dynamic pricing** from LiteLLM (cached for 7 days in `~/.cache/ccboard/pricing.json`)
//! 2. **Embedded pricing** as fallback when offline or cache expired
//!
//! The system automatically merges cached pricing with embedded pricing, preferring
//! cached values for known models and falling back to embedded pricing for unknown models.

pub mod cache;
pub mod embedded;
pub mod litellm;

use anyhow::Result;
pub use embedded::{ModelPricing, MODEL_PRICING};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Dynamic pricing map (merged from cache + embedded)
///
/// This is loaded at startup and used by `calculate_cost()`. It combines:
/// - Cached pricing from LiteLLM (if available and not expired)
/// - Embedded pricing as fallback
static DYNAMIC_PRICING: Lazy<RwLock<HashMap<String, ModelPricing>>> = Lazy::new(|| {
    let mut pricing = embedded::MODEL_PRICING.clone();

    // Try to load from cache
    if let Ok(Some(cached)) = cache::load_cached_pricing() {
        tracing::info!(
            "Merging {} cached prices with {} embedded prices",
            cached.len(),
            pricing.len()
        );
        pricing.extend(cached);
    } else {
        tracing::debug!("Using embedded pricing only (no cache available)");
    }

    RwLock::new(pricing)
});

/// Get pricing for a model (checks dynamic pricing first, then embedded)
pub fn get_model_pricing(model_id: &str) -> ModelPricing {
    // Try dynamic pricing first
    if let Ok(guard) = DYNAMIC_PRICING.read() {
        if let Some(pricing) = guard.get(model_id) {
            return pricing.clone();
        }
    }

    // Fallback to embedded pricing
    embedded::get_model_pricing(model_id)
}

/// Update pricing from LiteLLM and save to cache
pub async fn update_pricing_from_litellm() -> Result<usize> {
    tracing::info!("Updating pricing from LiteLLM");

    let fetched = litellm::fetch_litellm_pricing().await?;
    let count = fetched.len();

    // Save to cache
    cache::save_pricing_cache(fetched.clone())?;

    // Update in-memory pricing
    if let Ok(mut guard) = DYNAMIC_PRICING.write() {
        // Merge with embedded (keep embedded as fallback)
        let mut merged = embedded::MODEL_PRICING.clone();
        merged.extend(fetched);
        *guard = merged;
    }

    Ok(count)
}

/// Clear pricing cache
pub fn clear_cache() -> Result<()> {
    cache::clear_pricing_cache()
}

/// Calculate cost for token usage with a specific model
///
/// Uses dynamic pricing (LiteLLM cache + embedded fallback) for accurate cost calculation.
///
/// # Arguments
///
/// * `model` - Model ID (e.g., "opus-4", "sonnet-4", "claude-haiku-4-5-20251001")
/// * `input` - Regular input tokens (not cached)
/// * `output` - Generated output tokens
/// * `cache_create` - Tokens written to cache (also called cache_write_tokens)
/// * `cache_read` - Tokens read from cache
///
/// # Returns
///
/// Total cost in USD
///
/// # Examples
///
/// ```
/// use ccboard_core::pricing::calculate_cost;
///
/// // Opus-4.5: 1M input + 1M output
/// let cost = calculate_cost("opus-4", 1_000_000, 1_000_000, 0, 0);
/// assert_eq!(cost, 30.0); // $5 input + $25 output
/// ```
pub fn calculate_cost(
    model: &str,
    input: u64,
    output: u64,
    cache_create: u64,
    cache_read: u64,
) -> f64 {
    let pricing = get_model_pricing(model);

    let input_cost = (input as f64 / 1_000_000.0) * pricing.input_price_per_million;
    let output_cost = (output as f64 / 1_000_000.0) * pricing.output_price_per_million;
    let cache_create_cost = (cache_create as f64 / 1_000_000.0)
        * pricing.input_price_per_million
        * pricing.cache_write_multiplier;
    let cache_read_cost = (cache_read as f64 / 1_000_000.0)
        * pricing.input_price_per_million
        * pricing.cache_read_multiplier;

    input_cost + output_cost + cache_create_cost + cache_read_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_pricing_embedded() {
        let pricing = get_model_pricing("claude-opus-4-5");
        assert_eq!(pricing.input_price_per_million, 5.0);
        assert_eq!(pricing.output_price_per_million, 25.0);
    }

    #[test]
    fn test_get_model_pricing_unknown() {
        let pricing = get_model_pricing("unknown-model");
        // Should return default average
        assert!(pricing.input_price_per_million > 0.0);
    }

    #[test]
    fn test_calculate_cost_opus_basic() {
        // Opus-4.5: 1M input + 1M output = $5 + $25 = $30
        let cost = calculate_cost("opus-4", 1_000_000, 1_000_000, 0, 0);
        assert_eq!(cost, 30.0);
    }

    #[test]
    fn test_calculate_cost_sonnet_basic() {
        // Sonnet-4: 1M input + 1M output = $3 + $15 = $18
        let cost = calculate_cost("sonnet-4", 1_000_000, 1_000_000, 0, 0);
        assert_eq!(cost, 18.0);
    }

    #[test]
    fn test_calculate_cost_with_cache() {
        // Opus-4.5 with cache: 1M input + 1M cache_create + 10M cache_read
        // Input: 1M × $5 = $5
        // Cache create: 1M × $5 × 1.25 = $6.25
        // Cache read: 10M × $5 × 0.1 = $5
        // Total = $5 + $6.25 + $5 = $16.25
        let cost = calculate_cost("opus-4", 1_000_000, 0, 1_000_000, 10_000_000);
        assert_eq!(cost, 16.25);
    }
}
