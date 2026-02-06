//! Message filtering logic to exclude system/protocol messages
//!
//! Filters out Claude Code internal messages that don't represent
//! meaningful user interactions, improving session previews and search quality.

/// Check if a message represents meaningful user content
///
/// Returns false for:
/// - System commands: `<local-command`, `<command-`, `<system-reminder>`
/// - Protocol noise: `[Request interrupted`, `[Session resumed`, etc.
/// - Empty messages
///
/// # Examples
///
/// ```
/// use ccboard_core::parsers::filters::is_meaningful_user_message;
///
/// assert!(is_meaningful_user_message("Fix the bug in auth"));
/// assert!(!is_meaningful_user_message("<local-command>"));
/// assert!(!is_meaningful_user_message("[Request interrupted by user]"));
/// assert!(!is_meaningful_user_message(""));
/// ```
pub fn is_meaningful_user_message(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    // System/protocol prefixes (Claude Code internal commands)
    const SYSTEM_PREFIXES: &[&str] = &[
        "<local-command",
        "<command-",
        "<system-reminder>",
        "Caveat:",
    ];

    // Noise patterns (interruptions, system events)
    const NOISE_PATTERNS: &[&str] = &[
        "[Request interrupted",
        "[Session resumed",
        "[Tool output truncated",
        "[Session paused",
        "[Connection lost",
    ];

    // Exclude messages starting with system prefixes
    if SYSTEM_PREFIXES.iter().any(|p| content.starts_with(p)) {
        return false;
    }

    // Exclude messages containing noise patterns
    if NOISE_PATTERNS.iter().any(|p| content.contains(p)) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meaningful_messages() {
        // Regular user messages should pass
        assert!(is_meaningful_user_message("Fix the bug in auth"));
        assert!(is_meaningful_user_message("Implement JWT validation"));
        assert!(is_meaningful_user_message("go"));
        assert!(is_meaningful_user_message("What's the status?"));
    }

    #[test]
    fn test_system_commands_filtered() {
        // System commands should be filtered
        assert!(!is_meaningful_user_message("<local-command>"));
        assert!(!is_meaningful_user_message("<command-help>"));
        assert!(!is_meaningful_user_message("<system-reminder>"));
        assert!(!is_meaningful_user_message("Caveat: this is a warning"));
    }

    #[test]
    fn test_noise_patterns_filtered() {
        // Noise patterns should be filtered
        assert!(!is_meaningful_user_message(
            "[Request interrupted by user]"
        ));
        assert!(!is_meaningful_user_message("[Session resumed from previous state]"));
        assert!(!is_meaningful_user_message("[Tool output truncated]"));
        assert!(!is_meaningful_user_message("[Session paused]"));
        assert!(!is_meaningful_user_message("[Connection lost, retrying...]"));
    }

    #[test]
    fn test_empty_messages_filtered() {
        // Empty messages should be filtered
        assert!(!is_meaningful_user_message(""));
    }

    #[test]
    fn test_partial_matches_not_filtered() {
        // Messages containing keywords but not as patterns should pass
        assert!(is_meaningful_user_message(
            "How do I handle interrupted requests?"
        ));
        assert!(is_meaningful_user_message(
            "The session was resumed successfully"
        ));
    }
}
