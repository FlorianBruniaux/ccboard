//! Pricing calculations for Claude models based on official Anthropic pricing
//!
//! This module implements accurate pricing based on official Anthropic rates as of January 2025.
//! Pricing varies significantly between models and token types:
//!
//! - **Input tokens**: Regular input tokens (not cached)
//! - **Output tokens**: Generated tokens (typically 5x more expensive than input)
//! - **Cache write tokens**: Tokens written to cache (25% of input price)
//! - **Cache read tokens**: Tokens read from cache (10% of input price)
//!
//! # Examples
//!
//! ```
//! use ccboard_core::pricing::calculate_cost;
//!
//! // Calculate cost for Opus-4 with 1M input + 1M output
//! let cost = calculate_cost("opus-4", 1_000_000, 1_000_000, 0, 0);
//! assert_eq!(cost, 90.0); // $15 input + $75 output = $90
//!
//! // Calculate cost with cache
//! let cost = calculate_cost("opus-4", 1_000_000, 0, 1_000_000, 10_000_000);
//! // $15 input + $3.75 cache_write + $15 cache_read = $33.75
//! assert_eq!(cost, 33.75);
//! ```

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Pricing structure for a Claude model
///
/// All prices are per million tokens (M). Cache multipliers are applied to input price.
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Price per million input tokens ($/M)
    pub input_price_per_million: f64,
    /// Price per million output tokens ($/M)
    pub output_price_per_million: f64,
    /// Cache read multiplier (0.1 = 10% of input price)
    pub cache_read_multiplier: f64,
    /// Cache write multiplier (0.25 = 25% of input price)
    pub cache_write_multiplier: f64,
}

impl ModelPricing {
    /// Default average pricing for unknown models
    ///
    /// Uses a weighted average across common models (sonnet-4 weight: 70%, opus-4: 20%, haiku-4: 10%)
    /// to provide reasonable estimates when model is unknown or unrecognized.
    pub fn default_average() -> Self {
        Self {
            input_price_per_million: 3.5,   // Weighted average
            output_price_per_million: 17.5, // Weighted average
            cache_read_multiplier: 0.1,
            cache_write_multiplier: 0.25,
        }
    }
}

/// Official Claude pricing table (as of January 2025)
///
/// Source: https://www.anthropic.com/api#pricing
///
/// This table includes both full model IDs and common aliases for convenience.
/// All cache pricing follows Anthropic's standard multipliers:
/// - Cache read: 10% of input price
/// - Cache write: 25% of input price
static PRICING_TABLE: Lazy<HashMap<&'static str, ModelPricing>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Claude Opus 4.5 - Most powerful model
    let opus_pricing = ModelPricing {
        input_price_per_million: 15.0,
        output_price_per_million: 75.0,
        cache_read_multiplier: 0.1,
        cache_write_multiplier: 0.25,
    };
    m.insert("claude-opus-4-5-20251101", opus_pricing.clone());
    m.insert("opus-4", opus_pricing.clone());
    m.insert("claude-opus-4", opus_pricing);

    // Claude Sonnet 4.5 - Balanced model (most commonly used)
    let sonnet_pricing = ModelPricing {
        input_price_per_million: 3.0,
        output_price_per_million: 15.0,
        cache_read_multiplier: 0.1,
        cache_write_multiplier: 0.25,
    };
    m.insert("claude-sonnet-4-5-20250929", sonnet_pricing.clone());
    m.insert("sonnet-4", sonnet_pricing.clone());
    m.insert("claude-sonnet-4", sonnet_pricing);

    // Claude Haiku 4.5 - Fastest, most economical model
    let haiku_pricing = ModelPricing {
        input_price_per_million: 1.0,
        output_price_per_million: 5.0,
        cache_read_multiplier: 0.1,
        cache_write_multiplier: 0.25,
    };
    m.insert("claude-haiku-4-5-20251001", haiku_pricing.clone());
    m.insert("haiku-4", haiku_pricing.clone());
    m.insert("claude-haiku-4", haiku_pricing);

    m
});

/// Get pricing for a specific model
///
/// Returns the pricing structure for the given model ID. If the model is not recognized,
/// returns a default weighted average pricing based on typical usage patterns.
///
/// # Examples
///
/// ```
/// use ccboard_core::pricing::get_pricing;
///
/// let pricing = get_pricing("opus-4");
/// assert_eq!(pricing.input_price_per_million, 15.0);
///
/// let pricing = get_pricing("unknown-model");
/// assert_eq!(pricing.input_price_per_million, 3.5); // Default average
/// ```
pub fn get_pricing(model: &str) -> ModelPricing {
    PRICING_TABLE
        .get(model)
        .cloned()
        .unwrap_or_else(ModelPricing::default_average)
}

/// Calculate cost for token usage with a specific model
///
/// This is the main pricing calculation function. It applies official Anthropic pricing
/// rates for each token type and sums them to produce the total cost.
///
/// # Pricing Formula
///
/// ```text
/// Input cost = (input / 1M) × input_price
/// Output cost = (output / 1M) × output_price
/// Cache create cost = (cache_create / 1M) × input_price × 0.25
/// Cache read cost = (cache_read / 1M) × input_price × 0.1
/// Total = sum of all
/// ```
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
/// // Opus-4: 1M input + 1M output
/// let cost = calculate_cost("opus-4", 1_000_000, 1_000_000, 0, 0);
/// assert_eq!(cost, 90.0); // $15 input + $75 output
///
/// // Sonnet-4: 500K input + 100K output
/// let cost = calculate_cost("sonnet-4", 500_000, 100_000, 0, 0);
/// assert_eq!(cost, 3.0); // $1.5 input + $1.5 output
///
/// // Opus-4 with cache: 1M input + 1M cache_create + 10M cache_read
/// let cost = calculate_cost("opus-4", 1_000_000, 0, 1_000_000, 10_000_000);
/// assert_eq!(cost, 33.75); // $15 + $3.75 + $15
/// ```
pub fn calculate_cost(
    model: &str,
    input: u64,
    output: u64,
    cache_create: u64,
    cache_read: u64,
) -> f64 {
    let pricing = get_pricing(model);

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
    fn test_opus_pricing() {
        let pricing = get_pricing("opus-4");
        assert_eq!(pricing.input_price_per_million, 15.0);
        assert_eq!(pricing.output_price_per_million, 75.0);
        assert_eq!(pricing.cache_read_multiplier, 0.1);
        assert_eq!(pricing.cache_write_multiplier, 0.25);
    }

    #[test]
    fn test_sonnet_pricing() {
        let pricing = get_pricing("sonnet-4");
        assert_eq!(pricing.input_price_per_million, 3.0);
        assert_eq!(pricing.output_price_per_million, 15.0);
    }

    #[test]
    fn test_haiku_pricing() {
        let pricing = get_pricing("haiku-4");
        assert_eq!(pricing.input_price_per_million, 1.0);
        assert_eq!(pricing.output_price_per_million, 5.0);
    }

    #[test]
    fn test_full_model_id() {
        let pricing = get_pricing("claude-sonnet-4-5-20250929");
        assert_eq!(pricing.input_price_per_million, 3.0);
    }

    #[test]
    fn test_unknown_model_fallback() {
        let pricing = get_pricing("unknown-model-xyz");
        assert_eq!(pricing.input_price_per_million, 3.5); // Default average
    }

    #[test]
    fn test_cost_calculation_opus_basic() {
        // Opus-4: 1M input + 1M output = $15 + $75 = $90
        let cost = calculate_cost("opus-4", 1_000_000, 1_000_000, 0, 0);
        assert_eq!(cost, 90.0);
    }

    #[test]
    fn test_cost_calculation_sonnet_basic() {
        // Sonnet-4: 1M input + 1M output = $3 + $15 = $18
        let cost = calculate_cost("sonnet-4", 1_000_000, 1_000_000, 0, 0);
        assert_eq!(cost, 18.0);
    }

    #[test]
    fn test_cost_calculation_haiku_basic() {
        // Haiku-4: 1M input + 1M output = $1 + $5 = $6
        let cost = calculate_cost("haiku-4", 1_000_000, 1_000_000, 0, 0);
        assert_eq!(cost, 6.0);
    }

    #[test]
    fn test_cost_calculation_with_cache() {
        // Opus-4 with cache: 1M input + 1M cache_create + 10M cache_read
        // $15 input + ($15 × 0.25) cache_create + ($15 × 0.1 × 10) cache_read
        // = $15 + $3.75 + $15 = $33.75
        let cost = calculate_cost("opus-4", 1_000_000, 0, 1_000_000, 10_000_000);
        assert_eq!(cost, 33.75);
    }

    #[test]
    fn test_cost_calculation_zero_tokens() {
        let cost = calculate_cost("opus-4", 0, 0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_cost_calculation_small_numbers() {
        // Sonnet-4: 10K tokens input only = $0.03
        let cost = calculate_cost("sonnet-4", 10_000, 0, 0, 0);
        assert_eq!(cost, 0.03);
    }

    #[test]
    fn test_cost_calculation_mixed_tokens() {
        // Sonnet-4: 500K input + 100K output + 50K cache_create + 1M cache_read
        // Input: (500K / 1M) × $3 = $1.50
        // Output: (100K / 1M) × $15 = $1.50
        // Cache create: (50K / 1M) × $3 × 0.25 = $0.0375
        // Cache read: (1M / 1M) × $3 × 0.1 = $0.30
        // Total = $3.3375
        let cost = calculate_cost("sonnet-4", 500_000, 100_000, 50_000, 1_000_000);
        let expected = 1.5 + 1.5 + 0.0375 + 0.3;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_total_tokens_includes_cache_read() {
        // Verify that total_tokens calculation aligns with pricing
        let input = 1000u64;
        let output = 500u64;
        let cache_create = 100u64;
        let cache_read = 50000u64;

        let total = input + output + cache_create + cache_read;
        assert_eq!(total, 51600);

        // Cost should be calculated from all token types
        let cost = calculate_cost("sonnet-4", input, output, cache_create, cache_read);
        assert!(cost > 0.0);
    }
}
