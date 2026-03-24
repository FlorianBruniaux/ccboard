//! Parser for `~/.claude.json` — per-project last session costs and model usage
//!
//! Claude Code writes a `~/.claude.json` file with per-project metadata including
//! the token usage and cost of the last session, broken down by model.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Token and cost data for one model in one project's last session
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLastUsage {
    /// Claude Code uses "costUSD" (not "costUsd") — explicit rename required
    #[serde(rename = "costUSD")]
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub web_search_requests: u64,
}

impl ModelLastUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
    }
}

/// Per-project last session data from ~/.claude.json
#[derive(Debug, Clone)]
pub struct ProjectLastUsage {
    /// Absolute path of the project
    pub path: String,
    /// Project directory name (basename)
    pub name: String,
    /// Total cost of last session (sum across models)
    pub last_cost: f64,
    /// Per-model breakdown
    pub model_usage: HashMap<String, ModelLastUsage>,
}

/// Aggregated stats from ~/.claude.json across all projects
#[derive(Debug, Clone, Default)]
pub struct ClaudeGlobalStats {
    /// All projects with last-session data, sorted by cost descending
    pub projects: Vec<ProjectLastUsage>,
    /// Sum of last_cost across all projects (approximate lifetime lower bound)
    pub total_last_cost: f64,
    /// Auto-detected subscription plan from hasAvailableSubscription + hasOpusPlanDefault.
    /// None means the fields were absent (old CC version or unrecognized format).
    pub detected_plan: Option<DetectedPlan>,
}

/// Plan detected from ~/.claude.json account fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedPlan {
    /// Pro subscription ($20/month)
    Pro,
    /// Max subscription (5x or 20x — can't distinguish without API call)
    Max,
    /// API / enterprise (pay-as-you-go or org-level billing)
    Api,
}

// Intermediate deserialization types ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawProject {
    #[serde(default, rename = "lastModelUsage")]
    last_model_usage: HashMap<String, ModelLastUsage>,
    #[serde(default, rename = "lastCost")]
    last_cost: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOauthAccount {
    /// Non-null when user has (or had) a personal subscription
    subscription_created_at: Option<String>,
    #[allow(dead_code)]
    billing_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawClaudeJson {
    #[serde(default)]
    projects: HashMap<String, RawProject>,
    /// True when subscription quota is still available this period
    #[serde(default)]
    has_available_subscription: bool,
    /// True when the user's default model is Opus (strong Max indicator)
    #[serde(default)]
    has_opus_plan_default: bool,
    /// OAuth account metadata — present for authenticated users
    oauth_account: Option<RawOauthAccount>,
}

// ─────────────────────────────────────────────────────────────────────────────

/// Parse `~/.claude.json` and return aggregated project usage stats.
///
/// Returns `None` if the file does not exist or cannot be parsed.
/// Individual malformed project entries are silently skipped (graceful degradation).
pub fn parse_claude_global(home: &Path) -> Option<ClaudeGlobalStats> {
    let path = home.join(".claude.json");
    if !path.exists() {
        return None;
    }

    let data = std::fs::read(&path).ok()?;
    let raw: RawClaudeJson = serde_json::from_slice(&data).ok()?;

    // Detect plan — priority order matters:
    //  1. hasOpusPlanDefault: true → Max (strongest signal, independent of quota availability)
    //  2. hasAvailableSubscription: true → at minimum Pro
    //  3. oauthAccount.subscriptionCreatedAt present → has/had a subscription (Pro or Max
    //     with exhausted quota); show as Pro rather than Api to avoid false negatives
    //  4. Otherwise → Api (API key / no subscription)
    let has_subscription_record = raw
        .oauth_account
        .as_ref()
        .map(|a| a.subscription_created_at.is_some())
        .unwrap_or(false);

    let detected_plan = Some(if raw.has_opus_plan_default {
        DetectedPlan::Max
    } else if raw.has_available_subscription || has_subscription_record {
        DetectedPlan::Pro
    } else {
        DetectedPlan::Api
    });

    let mut projects: Vec<ProjectLastUsage> = raw
        .projects
        .into_iter()
        .filter_map(|(raw_path, project)| {
            // Skip projects with no usage data
            if project.last_model_usage.is_empty() && project.last_cost == 0.0 {
                return None;
            }

            let name = Path::new(&raw_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&raw_path)
                .to_string();

            Some(ProjectLastUsage {
                path: raw_path,
                name,
                last_cost: project.last_cost,
                model_usage: project.last_model_usage,
            })
        })
        .collect();

    // Sort by cost descending
    projects.sort_by(|a, b| {
        b.last_cost
            .partial_cmp(&a.last_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_last_cost = projects.iter().map(|p| p.last_cost).sum();

    Some(ClaudeGlobalStats {
        projects,
        total_last_cost,
        detected_plan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_claude_json(dir: &TempDir, content: &str) {
        let path = dir.path().join(".claude.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_parse_missing_file() {
        let dir = TempDir::new().unwrap();
        assert!(parse_claude_global(dir.path()).is_none());
    }

    #[test]
    fn test_parse_valid() {
        let dir = TempDir::new().unwrap();
        write_claude_json(
            &dir,
            r#"{
                "projects": {
                    "/Users/alice/myproject": {
                        "lastCost": 1.23,
                        "lastModelUsage": {
                            "claude-sonnet-4-5": {
                                "costUSD": 1.0,
                                "inputTokens": 1000,
                                "outputTokens": 500,
                                "cacheCreationInputTokens": 0,
                                "cacheReadInputTokens": 0,
                                "webSearchRequests": 0
                            },
                            "claude-haiku-4-5-20251001": {
                                "costUSD": 0.23,
                                "inputTokens": 2000,
                                "outputTokens": 100,
                                "cacheCreationInputTokens": 0,
                                "cacheReadInputTokens": 0,
                                "webSearchRequests": 0
                            }
                        }
                    },
                    "/Users/alice/other": {
                        "lastCost": 0.5,
                        "lastModelUsage": {
                            "claude-opus-4": {
                                "costUSD": 0.5,
                                "inputTokens": 500,
                                "outputTokens": 200,
                                "cacheCreationInputTokens": 0,
                                "cacheReadInputTokens": 0,
                                "webSearchRequests": 0
                            }
                        }
                    }
                }
            }"#,
        );

        let stats = parse_claude_global(dir.path()).expect("should parse");
        assert_eq!(stats.projects.len(), 2);
        // Sorted by cost desc: myproject (1.23) then other (0.5)
        assert_eq!(stats.projects[0].name, "myproject");
        assert_eq!(stats.projects[0].model_usage.len(), 2);
        assert!((stats.total_last_cost - 1.73).abs() < 0.01);
    }

    #[test]
    fn test_skips_empty_projects() {
        let dir = TempDir::new().unwrap();
        write_claude_json(
            &dir,
            r#"{
                "projects": {
                    "/tmp/empty": {
                        "lastCost": 0.0,
                        "lastModelUsage": {}
                    },
                    "/tmp/real": {
                        "lastCost": 0.42,
                        "lastModelUsage": {
                            "claude-sonnet-4-5": {
                                "costUSD": 0.42,
                                "inputTokens": 100,
                                "outputTokens": 50,
                                "cacheCreationInputTokens": 0,
                                "cacheReadInputTokens": 0,
                                "webSearchRequests": 0
                            }
                        }
                    }
                }
            }"#,
        );

        let stats = parse_claude_global(dir.path()).expect("should parse");
        assert_eq!(stats.projects.len(), 1);
        assert_eq!(stats.projects[0].name, "real");
    }

    #[test]
    fn test_total_tokens() {
        let usage = ModelLastUsage {
            cost_usd: 1.0,
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: 200,
            cache_read_input_tokens: 100,
            web_search_requests: 0,
        };
        assert_eq!(usage.total_tokens(), 1800);
    }
}
