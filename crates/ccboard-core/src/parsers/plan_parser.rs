//! Parser for PLAN.md files
//!
//! Parses PLAN.md files with YAML frontmatter and markdown sections to extract
//! phases, tasks, and metadata for workflow tracking.

use crate::models::plan::{Phase, PhaseStatus, PlanFile, PlanMetadata, Task};
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Task metadata tuple (issue, duration, difficulty, crate_name, description)
type TaskMetadata = (
    Option<u32>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// Parser for PLAN.md files
pub struct PlanParser;

impl PlanParser {
    /// Parse a PLAN.md file from a path
    pub fn parse_file(path: &Path) -> Result<Option<PlanFile>> {
        if !path.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(path).context(format!("Failed to read PLAN.md at {:?}", path))?;

        Self::parse(&content)
    }

    /// Parse PLAN.md content string
    pub fn parse(content: &str) -> Result<Option<PlanFile>> {
        // Split frontmatter and body
        let (frontmatter, body) = Self::split_frontmatter(content)?;

        if frontmatter.is_none() {
            // No frontmatter, graceful degradation
            return Ok(None);
        }

        // Parse YAML frontmatter
        let metadata = Self::parse_metadata(frontmatter.unwrap())
            .context("Failed to parse PLAN.md frontmatter")?;

        // Parse phases from markdown sections
        let phases = Self::parse_phases(&body).context("Failed to parse phases")?;

        Ok(Some(PlanFile { metadata, phases }))
    }

    /// Split content into frontmatter and body
    fn split_frontmatter(content: &str) -> Result<(Option<&str>, String)> {
        // Check if content starts with ---
        if !content.trim_start().starts_with("---") {
            return Ok((None, content.to_string()));
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            // Malformed frontmatter
            return Ok((None, content.to_string()));
        }

        // parts[0] is empty (before first ---)
        // parts[1] is frontmatter
        // parts[2] is body
        Ok((Some(parts[1].trim()), parts[2].to_string()))
    }

    /// Parse YAML frontmatter into metadata
    fn parse_metadata(yaml: &str) -> Result<PlanMetadata> {
        serde_yaml::from_str(yaml).context("Failed to parse YAML frontmatter")
    }

    /// Parse markdown body into phases
    fn parse_phases(body: &str) -> Result<Vec<Phase>> {
        let mut phases = Vec::new();

        // Regex to match phase headers: ## [emoji] Phase F: Title (rest of line)
        // Emoji prefix (✅🚧⏸️❌) is optional, phase ID can contain dots (Phase 2.1)
        // Captures: 1=phase_id, 2=rest of line (contains title + optional priority)
        let phase_re = Regex::new(r"(?m)^##\s+(?:[✅🚧⏸️❌]\s+)?Phase\s+([A-Za-z0-9\.]+):\s+(.+)$")
            .context("Failed to compile phase regex")?;

        // Regex to extract priority from line: (Priority: 🔴 HIGH)
        let priority_re = Regex::new(r"\(Priority:.+?\s+([A-Z]+)\)")
            .context("Failed to compile priority regex")?;

        // Find all phase headers
        let matches: Vec<_> = phase_re.captures_iter(body).collect();

        for (i, cap) in matches.iter().enumerate() {
            let phase_id = cap[1].trim().to_string();
            let rest_of_line = cap[2].trim();

            // Extract priority from rest of line
            let priority = priority_re
                .captures(rest_of_line)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            // Extract title by removing " (Priority...)" if present
            let mut phase_title = rest_of_line.to_string();
            if let Some(pos) = phase_title.find(" (Priority:") {
                phase_title = phase_title[..pos].trim().to_string();
            }

            // Extract section content (from current ## Phase to next ## Phase or end)
            let start = cap.get(0).unwrap().end();
            let end = if i + 1 < matches.len() {
                matches[i + 1].get(0).unwrap().start()
            } else {
                body.len()
            };

            let section = &body[start..end];

            // Parse phase metadata and tasks from section
            // Pass full header line to extract status from emoji
            let header = cap.get(0).unwrap().as_str();
            let (status, estimated_duration, version_target) =
                Self::parse_phase_metadata(section, header);
            let tasks = Self::parse_tasks(&phase_id, section)?;

            phases.push(Phase {
                id: phase_id,
                title: phase_title,
                status,
                tasks,
                estimated_duration,
                priority,
                version_target,
            });
        }

        Ok(phases)
    }

    /// Parse phase metadata from section content
    fn parse_phase_metadata(
        section: &str,
        header: &str,
    ) -> (PhaseStatus, Option<String>, Option<String>) {
        // Extract status from emoji prefix in header (✅=Complete, 🚧=InProgress, default=Future)
        let mut status = if header.contains("✅") {
            PhaseStatus::Complete
        } else if header.contains("🚧") {
            PhaseStatus::InProgress
        } else {
            PhaseStatus::Future
        };
        let mut estimated_duration = None;
        let mut version_target = None;

        // Look for metadata lines
        for line in section.lines().take(20) {
            // Check first 20 lines
            let line = line.trim();

            if line.starts_with("**Durée estimée**") {
                if let Some(duration) = Self::extract_metadata_value(line) {
                    estimated_duration = Some(duration);
                }
            }

            if line.starts_with("**Version cible**") {
                if let Some(version) = Self::extract_metadata_value(line) {
                    version_target = Some(version);
                }
            }

            if line.contains("in-progress") || line.contains("IN PROGRESS") {
                status = PhaseStatus::InProgress;
            }

            if line.contains("complete") || line.contains("COMPLETE") {
                status = PhaseStatus::Complete;
            }
        }

        (status, estimated_duration, version_target)
    }

    /// Extract value from metadata line (e.g., "**Key**: value")
    fn extract_metadata_value(line: &str) -> Option<String> {
        if let Some(pos) = line.find(':') {
            let value = line[pos + 1..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }

    /// Parse tasks from phase section
    fn parse_tasks(_phase_id: &str, section: &str) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Regex to match task headers: #### Task F.1: Title [emoji] (P0)
        // Emoji status (✅🚧⏸️❌) is optional after title, excluded from title capture
        let task_re =
            Regex::new(r"(?m)^####\s+Task\s+([A-Za-z0-9\.]+):\s+([^\(✅🚧⏸️❌]+?)(?:\s+[✅🚧⏸️❌])?(?:\s+\(([^\)]+)\))?\s*$")
                .context("Failed to compile task regex")?;

        // Find all task headers
        let matches: Vec<_> = task_re.captures_iter(section).collect();

        for (i, cap) in matches.iter().enumerate() {
            let task_id = cap[1].trim().to_string();
            let task_title = cap[2].trim().to_string();
            let priority = cap.get(3).map(|m| m.as_str().to_string());

            // Extract task content (from current #### Task to next #### Task or next ### section)
            let start = cap.get(0).unwrap().end();
            let end = if i + 1 < matches.len() {
                matches[i + 1].get(0).unwrap().start()
            } else {
                // Find next ### section or end
                section[start..]
                    .find("\n###")
                    .map(|pos| start + pos)
                    .unwrap_or(section.len())
            };

            let task_content = &section[start..end];

            // Parse task metadata
            let (issue, duration, difficulty, crate_name, description) =
                Self::parse_task_metadata(task_content);

            tasks.push(Task {
                id: task_id,
                title: task_title,
                description,
                priority,
                duration,
                difficulty,
                crate_name,
                issue,
            });
        }

        Ok(tasks)
    }

    /// Parse task metadata from task section
    fn parse_task_metadata(content: &str) -> TaskMetadata {
        let mut issue = None;
        let mut duration = None;
        let mut difficulty = None;
        let mut crate_name = None;
        let mut description = None;

        for line in content.lines().take(30) {
            let line = line.trim();

            if line.starts_with("**Issue**") {
                if let Some(val) = Self::extract_metadata_value(line) {
                    // Extract number from "#123" format
                    if let Some(num_str) = val.strip_prefix('#') {
                        issue = num_str.parse::<u32>().ok();
                    }
                }
            }

            if line.starts_with("**Durée**") {
                duration = Self::extract_metadata_value(line);
            }

            if line.starts_with("**Difficulté**") {
                difficulty = Self::extract_metadata_value(line);
            }

            if line.starts_with("**Crate**") {
                crate_name = Self::extract_metadata_value(line);
            }

            // Capture first non-metadata paragraph as description
            if description.is_none()
                && !line.is_empty()
                && !line.starts_with('*')
                && !line.starts_with('#')
                && !line.starts_with('-')
            {
                description = Some(line.to_string());
            }
        }

        (issue, duration, difficulty, crate_name, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = r#"---
date: 2026-02-12
title: Test Plan
---

# Body content
"#;

        let (fm, body) = PlanParser::split_frontmatter(content).unwrap();
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("date: 2026-02-12"));
        assert!(body.contains("Body content"));
    }

    #[test]
    fn test_split_frontmatter_missing() {
        let content = "# No frontmatter\n\nJust body";
        let (fm, body) = PlanParser::split_frontmatter(content).unwrap();
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_metadata() {
        let yaml = r#"
date: 2026-02-12
title: Test Plan
status: in-progress
version: 0.7.0
"#;

        let meta = PlanParser::parse_metadata(yaml).unwrap();
        assert_eq!(meta.title, "Test Plan");
        assert_eq!(meta.status, Some("in-progress".to_string()));
        assert_eq!(meta.version, Some("0.7.0".to_string()));
    }

    #[test]
    fn test_extract_metadata_value() {
        assert_eq!(
            PlanParser::extract_metadata_value("**Durée**: 3-4h"),
            Some("3-4h".to_string())
        );
        assert_eq!(
            PlanParser::extract_metadata_value("**Issue**: #42"),
            Some("#42".to_string())
        );
    }
}
