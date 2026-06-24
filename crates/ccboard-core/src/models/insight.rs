//! Insight model for Brain feature — cross-session knowledge storage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of insight stored in Brain DB
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsightType {
    Progress,
    Decision,
    Blocked,
    Pattern,
    Fix,
    Context,
}

impl InsightType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsightType::Progress => "progress",
            InsightType::Decision => "decision",
            InsightType::Blocked => "blocked",
            InsightType::Pattern => "pattern",
            InsightType::Fix => "fix",
            InsightType::Context => "context",
        }
    }
}

impl std::fmt::Display for InsightType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for InsightType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "progress" => Ok(InsightType::Progress),
            "decision" => Ok(InsightType::Decision),
            "blocked" => Ok(InsightType::Blocked),
            "pattern" => Ok(InsightType::Pattern),
            "fix" => Ok(InsightType::Fix),
            "context" => Ok(InsightType::Context),
            other => Err(format!("Unknown insight type: {other}")),
        }
    }
}

/// A single Brain insight record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: i64,
    /// Session that produced this insight (None for manual /ccboard-remember)
    pub session_id: Option<String>,
    /// Absolute path of the project (cwd at session time)
    pub project: String,
    pub insight_type: InsightType,
    pub content: String,
    /// Original user input — filled for manual entries
    pub reasoning: Option<String>,
    /// Soft-deleted insights are excluded from queries by default
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}
