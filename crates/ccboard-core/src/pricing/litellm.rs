//! LiteLLM pricing parser
//!
//! Fetches and parses model pricing from LiteLLM's canonical pricing database.
//! Source: https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json

use crate::pricing::ModelPricing;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LiteLLM pricing source URL
pub const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

/// LiteLLM model entry (simplified, only what we need)
#[derive(Debug, Deserialize)]
struct LiteLLMModelEntry {
    /// Input cost per token (e.g., 5e-06 = $0.000005 = $5/M)
    #[serde(default)]
    input_cost_per_token: Option<f64>,

    /// Output cost per token
    #[serde(default)]
    output_cost_per_token: Option<f64>,

    /// Cache creation (write) cost per token
    #[serde(default)]
    cache_creation_input_token_cost: Option<f64>,

    /// Cache read cost per token
    #[serde(default)]
    cache_read_input_token_cost: Option<f64>,

    /// Provider (we only care about "anthropic" and "bedrock")
    #[serde(default)]
    litellm_provider: Option<String>,

    /// Mode (we only care about "chat")
    #[serde(default)]
    mode: Option<String>,
}

/// Cached pricing data
#[derive(Debug, Serialize, Deserialize)]
pub struct CachedPricing {
    /// When the pricing was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,

    /// Model pricing map
    pub models: HashMap<String, ModelPricing>,

    /// Source URL
    pub source: String,
}

/// Fetch pricing from LiteLLM API
pub async fn fetch_litellm_pricing() -> Result<HashMap<String, ModelPricing>> {
    tracing::info!("Fetching pricing from LiteLLM: {}", LITELLM_PRICING_URL);

    let response = reqwest::get(LITELLM_PRICING_URL)
        .await
        .context("Failed to fetch LiteLLM pricing")?;

    let json_text = response
        .text()
        .await
        .context("Failed to read LiteLLM response")?;

    parse_litellm_json(&json_text)
}

/// Parse LiteLLM JSON into ModelPricing map
fn parse_litellm_json(json: &str) -> Result<HashMap<String, ModelPricing>> {
    let entries: HashMap<String, LiteLLMModelEntry> =
        serde_json::from_str(json).context("Failed to parse LiteLLM JSON")?;

    let mut pricing_map = HashMap::new();

    for (model_id, entry) in entries {
        // Only process Anthropic Claude models in chat mode
        if let Some(provider) = &entry.litellm_provider {
            if !matches!(provider.as_str(), "anthropic" | "bedrock") {
                continue;
            }
        }

        if let Some(mode) = &entry.mode {
            if mode != "chat" {
                continue;
            }
        }

        // Only include Claude models
        if !model_id.starts_with("claude-") {
            continue;
        }

        // Extract pricing (skip if missing critical fields)
        let Some(input_cost) = entry.input_cost_per_token else {
            continue;
        };
        let Some(output_cost) = entry.output_cost_per_token else {
            continue;
        };

        // Convert per-token to per-million (multiply by 1,000,000)
        let input_price_per_million = input_cost * 1_000_000.0;
        let output_price_per_million = output_cost * 1_000_000.0;

        // Calculate multipliers from cache costs
        let (cache_read_mult, cache_write_mult) = if let (Some(cache_read), Some(cache_write)) = (
            entry.cache_read_input_token_cost,
            entry.cache_creation_input_token_cost,
        ) {
            (
                cache_read / input_cost,  // Should be ~0.1
                cache_write / input_cost, // Should be ~1.25
            )
        } else {
            (0.1, 1.25) // Defaults
        };

        pricing_map.insert(
            model_id,
            ModelPricing {
                input_price_per_million,
                output_price_per_million,
                cache_read_multiplier: cache_read_mult,
                cache_write_multiplier: cache_write_mult,
            },
        );
    }

    tracing::info!("Parsed {} Claude model prices from LiteLLM", pricing_map.len());

    Ok(pricing_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_litellm_json() {
        let json = r#"{
            "claude-opus-4-5": {
                "input_cost_per_token": 5e-06,
                "output_cost_per_token": 2.5e-05,
                "cache_creation_input_token_cost": 6.25e-06,
                "cache_read_input_token_cost": 5e-07,
                "litellm_provider": "anthropic",
                "mode": "chat"
            },
            "claude-sonnet-4-5-20250929": {
                "input_cost_per_token": 3e-06,
                "output_cost_per_token": 1.5e-05,
                "cache_creation_input_token_cost": 3.75e-06,
                "cache_read_input_token_cost": 3e-07,
                "litellm_provider": "anthropic",
                "mode": "chat"
            },
            "gpt-4": {
                "input_cost_per_token": 1e-05,
                "output_cost_per_token": 3e-05,
                "litellm_provider": "openai",
                "mode": "chat"
            }
        }"#;

        let pricing = parse_litellm_json(json).unwrap();

        // Should only have Claude models
        assert_eq!(pricing.len(), 2);

        // Opus 4.5
        let opus = pricing.get("claude-opus-4-5").unwrap();
        assert_eq!(opus.input_price_per_million, 5.0);
        assert_eq!(opus.output_price_per_million, 25.0);
        assert!((opus.cache_read_multiplier - 0.1).abs() < 0.01);
        assert!((opus.cache_write_multiplier - 1.25).abs() < 0.01);

        // Sonnet 4.5
        let sonnet = pricing.get("claude-sonnet-4-5-20250929").unwrap();
        assert_eq!(sonnet.input_price_per_million, 3.0);
        assert_eq!(sonnet.output_price_per_million, 15.0);
    }
}
