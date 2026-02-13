//! Data models for PLAN.md files

use serde::{Deserialize, Serialize};

/// Full PLAN.md file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFile {
    /// Metadata from YAML frontmatter
    pub metadata: PlanMetadata,

    /// Phases extracted from markdown sections
    pub phases: Vec<Phase>,
}

/// YAML frontmatter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// Plan creation date
    pub date: Option<String>,

    /// Last update timestamp
    pub last_updated: Option<String>,

    /// Version string (e.g., "0.7.0-dev")
    pub version: Option<String>,

    /// Plan title
    pub title: String,

    /// Current status (in-progress, complete, future)
    pub status: String,

    /// Estimated total duration string (e.g., "82-120h")
    pub estimated_total_duration: Option<String>,
}

/// Individual phase in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    /// Phase identifier (e.g., "F", "H", "11", "12")
    pub id: String,

    /// Phase title
    pub title: String,

    /// Phase status
    pub status: PhaseStatus,

    /// Tasks within this phase
    pub tasks: Vec<Task>,

    /// Estimated duration for this phase
    pub estimated_duration: Option<String>,

    /// Priority level (HIGH, MEDIUM, LOW)
    pub priority: Option<String>,

    /// Version target (e.g., "v0.7.0")
    pub version_target: Option<String>,
}

/// Phase status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    /// Phase completed
    Complete,

    /// Phase in progress
    InProgress,

    /// Phase not started yet
    Future,
}

/// Individual task within a phase
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Task {
    /// Task identifier (e.g., "F.1", "H.2")
    pub id: String,

    /// Task title
    pub title: String,

    /// Task description
    pub description: Option<String>,

    /// Priority level (P0, P1, P2)
    pub priority: Option<String>,

    /// Estimated duration
    pub duration: Option<String>,

    /// Difficulty level (Good First Issue, Intermediate, Advanced)
    pub difficulty: Option<String>,

    /// Crate affected
    pub crate_name: Option<String>,

    /// GitHub issue number
    pub issue: Option<u32>,
}

impl Default for PlanMetadata {
    fn default() -> Self {
        Self {
            date: None,
            last_updated: None,
            version: None,
            title: String::new(),
            status: "future".to_string(),
            estimated_total_duration: None,
        }
    }
}

impl PhaseStatus {
    /// Parse status from string (not implementing FromStr trait to keep simple)
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "complete" | "completed" | "done" => Self::Complete,
            "in-progress" | "in_progress" | "active" => Self::InProgress,
            _ => Self::Future,
        }
    }
}
