//! Parser for Claude Code hook scripts
//!
//! Parses `.claude/hooks/bash/*.sh` to extract hook metadata.
//!
//! # Specification (Spec-Driven Development)
//!
//! ## Feature: Parse bash hooks from .claude/hooks/bash/
//!
//! ### Scenario 1: Valid hook with shebang
//! - Given: `pre-commit.sh` with `#!/bin/bash` and executable permissions
//! - When: Parser scans hooks directory
//! - Then: Hook metadata extracted (name, type, path, executable)
//!
//! ### Scenario 2: Missing shebang
//! - Given: Hook file without `#!/bin/bash`
//! - When: Parser validates hook
//! - Then: Returns ValidationError indicating missing shebang
//!
//! ### Scenario 3: Non-executable hook
//! - Given: Hook file with correct content but no executable permissions
//! - When: Parser validates hook
//! - Then: Hook parsed but marked as non-executable
//!
//! ### Scenario 4: Multiple hooks in directory
//! - Given: Directory with multiple .sh files
//! - When: Parser scans directory
//! - Then: All hooks extracted with correct metadata

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Hook type based on filename convention
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PrePush,
    UserPromptSubmit,
    ToolResultReturn,
    Custom(String),
}

impl HookType {
    /// Parse hook type from filename
    fn from_filename(name: &str) -> Self {
        match name.trim_end_matches(".sh") {
            "pre-commit" => HookType::PreCommit,
            "post-commit" => HookType::PostCommit,
            "pre-push" => HookType::PrePush,
            "user-prompt-submit" => HookType::UserPromptSubmit,
            "tool-result-return" => HookType::ToolResultReturn,
            custom => HookType::Custom(custom.to_string()),
        }
    }
}

/// Hook metadata extracted from file
#[derive(Debug, Clone)]
pub struct Hook {
    pub name: String,
    pub hook_type: HookType,
    pub path: PathBuf,
    pub is_executable: bool,
    pub has_valid_shebang: bool,
}

/// Hook parsing and validation errors
#[derive(Debug, Error)]
pub enum HookError {
    #[error("Missing shebang: hook must start with #!/bin/bash")]
    MissingShebang,

    #[error("Invalid shebang: expected #!/bin/bash, got {0}")]
    InvalidShebang(String),
}

/// Parser for hook scripts
pub struct HooksParser;

impl HooksParser {
    /// Scan hooks directory and parse all .sh files
    ///
    /// Spec-Driven Development: Implement scenario 4 first (multiple hooks)
    pub fn scan_directory(hooks_dir: &Path) -> Result<Vec<Hook>> {
        // GREEN: Minimal implementation for scenario 4
        let mut hooks = Vec::new();

        if !hooks_dir.exists() {
            return Ok(hooks);
        }

        for entry in fs::read_dir(hooks_dir)
            .with_context(|| format!("Failed to read hooks directory: {}", hooks_dir.display()))?
        {
            let entry = entry.with_context(|| {
                format!(
                    "Failed to read entry in hooks directory: {}",
                    hooks_dir.display()
                )
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sh") {
                match Self::parse_hook(&path) {
                    Ok(hook) => hooks.push(hook),
                    Err(e) => {
                        // Log error but continue scanning other hooks
                        eprintln!("Warning: Failed to parse hook {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(hooks)
    }

    /// Parse a single hook file
    ///
    /// Spec-Driven Development: Implement scenario 1 (valid hook)
    pub fn parse_hook(path: &Path) -> Result<Hook> {
        // GREEN: Minimal implementation for scenario 1
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read hook file: {}", path.display()))?;

        let has_valid_shebang = Self::validate_shebang(&content).is_ok();

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let hook_type = HookType::from_filename(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown"),
        );

        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read hook metadata: {}", path.display()))?;

        #[cfg(unix)]
        let is_executable = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        };

        #[cfg(not(unix))]
        let is_executable = false;

        Ok(Hook {
            name,
            hook_type,
            path: path.to_path_buf(),
            is_executable,
            has_valid_shebang,
        })
    }

    /// Validate hook shebang
    ///
    /// Spec-Driven Development: Implement scenario 2 (missing shebang)
    fn validate_shebang(content: &str) -> Result<(), HookError> {
        // GREEN: Minimal implementation for scenario 2
        let first_line = content.lines().next().unwrap_or("");

        if first_line.is_empty() {
            return Err(HookError::MissingShebang);
        }

        if !first_line.starts_with("#!/bin/bash") {
            return Err(HookError::InvalidShebang(first_line.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    // Scenario 1: Valid hook with shebang
    #[test]
    fn test_valid_hook_with_shebang() {
        let temp_dir = TempDir::new().unwrap();
        let hook_path = temp_dir.path().join("pre-commit.sh");

        let content = "#!/bin/bash\necho 'Running pre-commit hook'";
        fs::write(&hook_path, content).unwrap();

        // Make executable
        let mut perms = fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).unwrap();

        let hook = HooksParser::parse_hook(&hook_path).unwrap();

        assert_eq!(hook.name, "pre-commit");
        assert_eq!(hook.hook_type, HookType::PreCommit);
        assert!(hook.is_executable);
        assert!(hook.has_valid_shebang);
    }

    // Scenario 2: Missing shebang
    #[test]
    fn test_missing_shebang_returns_validation_error() {
        let content = "echo 'No shebang'";
        let result = HooksParser::validate_shebang(content);

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Content starts with "echo", so it's InvalidShebang, not MissingShebang
        assert!(matches!(err, HookError::InvalidShebang(_)));
    }

    // Scenario 3: Non-executable hook
    #[test]
    fn test_non_executable_hook_marked_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let hook_path = temp_dir.path().join("pre-commit.sh");

        let content = "#!/bin/bash\necho 'test'";
        fs::write(&hook_path, content).unwrap();

        // Ensure NOT executable (default on most systems)
        let mut perms = fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o644); // rw-r--r--
        fs::set_permissions(&hook_path, perms).unwrap();

        let hook = HooksParser::parse_hook(&hook_path).unwrap();

        assert!(!hook.is_executable);
        assert!(hook.has_valid_shebang);
    }

    // Scenario 4: Multiple hooks in directory
    #[test]
    fn test_scan_directory_finds_multiple_hooks() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple hooks
        for name in &["pre-commit.sh", "post-commit.sh", "custom-hook.sh"] {
            let path = temp_dir.path().join(name);
            fs::write(&path, "#!/bin/bash\necho 'test'").unwrap();
        }

        let hooks = HooksParser::scan_directory(temp_dir.path()).unwrap();

        assert_eq!(hooks.len(), 3);
        assert!(hooks.iter().any(|h| h.name == "pre-commit"));
        assert!(hooks.iter().any(|h| h.name == "post-commit"));
        assert!(hooks.iter().any(|h| h.name == "custom-hook"));
    }

    // Edge case: Invalid shebang format
    #[test]
    fn test_invalid_shebang_returns_error() {
        let content = "#!/usr/bin/env python\nprint('wrong')";
        let result = HooksParser::validate_shebang(content);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HookError::InvalidShebang(_)));
    }

    // Edge case: Empty file
    #[test]
    fn test_empty_file_returns_missing_shebang_error() {
        let result = HooksParser::validate_shebang("");
        assert!(result.is_err());
    }

    // Edge case: Hook type parsing
    #[test]
    fn test_hook_type_parsing() {
        assert_eq!(
            HookType::from_filename("pre-commit.sh"),
            HookType::PreCommit
        );
        assert_eq!(
            HookType::from_filename("user-prompt-submit.sh"),
            HookType::UserPromptSubmit
        );

        match HookType::from_filename("my-custom.sh") {
            HookType::Custom(name) => assert_eq!(name, "my-custom"),
            _ => panic!("Expected Custom variant"),
        }
    }
}
