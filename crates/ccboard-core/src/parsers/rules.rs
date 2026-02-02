//! Parser for CLAUDE.md rules files
//!
//! Parses global (~/.claude/CLAUDE.md) and project (.claude/CLAUDE.md) rules files.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Combined rules from global and project CLAUDE.md files
#[derive(Debug, Clone, Default)]
pub struct Rules {
    /// Global rules from ~/.claude/CLAUDE.md
    pub global: Option<RulesFile>,

    /// Project rules from .claude/CLAUDE.md
    pub project: Option<RulesFile>,
}

/// Individual rules file
#[derive(Debug, Clone)]
pub struct RulesFile {
    /// Path to the file
    pub path: PathBuf,

    /// Full content
    pub content: String,

    /// File size in bytes
    pub size: u64,
}

impl Rules {
    /// Load rules from global and optional project paths
    pub fn load(claude_home: &Path, project: Option<&Path>) -> Result<Self> {
        let global = Self::load_file(&claude_home.join("CLAUDE.md"));

        let project = project.and_then(|p| Self::load_file(&p.join(".claude/CLAUDE.md")));

        Ok(Rules { global, project })
    }

    /// Load a single rules file if it exists
    fn load_file(path: &Path) -> Option<RulesFile> {
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let size = fs::metadata(path).ok()?.len();

        Some(RulesFile {
            path: path.to_path_buf(),
            content,
            size,
        })
    }

    /// Get preview lines (first N lines) from a rules file
    pub fn preview(file: &RulesFile, max_lines: usize) -> Vec<String> {
        file.content
            .lines()
            .take(max_lines)
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_global_only() {
        let temp = TempDir::new().unwrap();
        let claude_home = temp.path();

        fs::write(
            claude_home.join("CLAUDE.md"),
            "# Global Rules\n\nBe helpful.",
        )
        .unwrap();

        let rules = Rules::load(claude_home, None).unwrap();
        assert!(rules.global.is_some());
        assert!(rules.project.is_none());

        let global = rules.global.unwrap();
        assert_eq!(global.size, 27);
        assert!(global.content.contains("Be helpful"));
    }

    #[test]
    fn test_load_with_project() {
        let temp = TempDir::new().unwrap();
        let claude_home = temp.path();
        let project_dir = temp.path().join("myproject");

        fs::create_dir_all(project_dir.join(".claude")).unwrap();
        fs::write(claude_home.join("CLAUDE.md"), "# Global\n").unwrap();
        fs::write(
            project_dir.join(".claude/CLAUDE.md"),
            "# Project Rules\n\nUse TypeScript.",
        )
        .unwrap();

        let rules = Rules::load(claude_home, Some(&project_dir)).unwrap();
        assert!(rules.global.is_some());
        assert!(rules.project.is_some());

        let project = rules.project.unwrap();
        assert!(project.content.contains("TypeScript"));
    }

    #[test]
    fn test_load_missing_files() {
        let temp = TempDir::new().unwrap();
        let rules = Rules::load(temp.path(), None).unwrap();
        assert!(rules.global.is_none());
        assert!(rules.project.is_none());
    }

    #[test]
    fn test_preview() {
        let file = RulesFile {
            path: PathBuf::from("/fake/path"),
            content: "Line 1\nLine 2\nLine 3\nLine 4".to_string(),
            size: 28,
        };

        let preview = Rules::preview(&file, 2);
        assert_eq!(preview.len(), 2);
        assert_eq!(preview[0], "Line 1");
        assert_eq!(preview[1], "Line 2");
    }
}
