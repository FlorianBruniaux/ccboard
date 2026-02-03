//! Unified theme and color system for ccboard TUI
//!
//! Provides consistent color language across all tabs:
//! - 🟢 Green: Running, Healthy, Success
//! - 🔴 Red: Failed, Error, Critical
//! - 🟡 Yellow: Warning, Pending, Attention
//! - ⚪ Gray: Unknown, Disabled, Neutral
//! - 🔵 Cyan: Selected, Focus, Interactive
//! - 🟣 Magenta: High value, Important

use ratatui::style::Color;

/// Status color palette following k9s/lazygit conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusColor {
    /// Green: Running, Healthy, Success
    Success,
    /// Red: Failed, Error, Critical
    Error,
    /// Yellow: Warning, Pending, Attention
    Warning,
    /// Gray: Unknown, Disabled, Neutral
    Neutral,
    /// Cyan: Selected, Focus, Interactive
    Focus,
    /// Magenta: High value, Important, Cost alerts
    Important,
}

impl StatusColor {
    /// Convert to Ratatui Color
    pub fn to_color(self) -> Color {
        match self {
            StatusColor::Success => Color::Green,
            StatusColor::Error => Color::Red,
            StatusColor::Warning => Color::Yellow,
            StatusColor::Neutral => Color::DarkGray,
            StatusColor::Focus => Color::Cyan,
            StatusColor::Important => Color::Magenta,
        }
    }
}

/// Server status semantic color
pub enum ServerStatusColor {
    Running,
    Stopped,
    Unknown,
}

impl ServerStatusColor {
    pub fn to_color(self) -> Color {
        match self {
            ServerStatusColor::Running => StatusColor::Success.to_color(),
            ServerStatusColor::Stopped => StatusColor::Error.to_color(),
            ServerStatusColor::Unknown => StatusColor::Neutral.to_color(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ServerStatusColor::Running => "●",
            ServerStatusColor::Stopped => "○",
            ServerStatusColor::Unknown => "?",
        }
    }
}

/// Session status semantic color
pub enum SessionStatusColor {
    Active,
    Completed,
    Error,
}

impl SessionStatusColor {
    pub fn to_color(self) -> Color {
        match self {
            SessionStatusColor::Active => StatusColor::Focus.to_color(),
            SessionStatusColor::Completed => StatusColor::Success.to_color(),
            SessionStatusColor::Error => StatusColor::Error.to_color(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SessionStatusColor::Active => "▶",
            SessionStatusColor::Completed => "✓",
            SessionStatusColor::Error => "✗",
        }
    }
}

/// Hook event type semantic color
pub enum HookEventColor {
    PreToolUse,
    UserPromptSubmit,
    Other,
}

impl HookEventColor {
    pub fn to_color(self) -> Color {
        match self {
            HookEventColor::PreToolUse => StatusColor::Focus.to_color(),
            HookEventColor::UserPromptSubmit => StatusColor::Success.to_color(),
            HookEventColor::Other => StatusColor::Neutral.to_color(),
        }
    }
}

/// Cost threshold semantic color
pub enum CostLevelColor {
    /// <$50/day
    Low,
    /// $50-$100/day
    Medium,
    /// >$100/day
    High,
}

impl CostLevelColor {
    pub fn from_daily_cost(cost: f64) -> Self {
        if cost > 100.0 {
            CostLevelColor::High
        } else if cost > 50.0 {
            CostLevelColor::Medium
        } else {
            CostLevelColor::Low
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            CostLevelColor::Low => StatusColor::Success.to_color(),
            CostLevelColor::Medium => StatusColor::Warning.to_color(),
            CostLevelColor::High => StatusColor::Error.to_color(),
        }
    }
}

/// Model usage intensity semantic color
pub enum UsageIntensityColor {
    /// <25% of total tokens
    Low,
    /// 25-75% of total tokens
    Medium,
    /// >75% of total tokens
    High,
}

impl UsageIntensityColor {
    pub fn from_percentage(pct: f64) -> Self {
        if pct > 75.0 {
            UsageIntensityColor::High
        } else if pct > 25.0 {
            UsageIntensityColor::Medium
        } else {
            UsageIntensityColor::Low
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            UsageIntensityColor::Low => StatusColor::Success.to_color(),
            UsageIntensityColor::Medium => StatusColor::Warning.to_color(),
            UsageIntensityColor::High => StatusColor::Important.to_color(),
        }
    }
}

/// Context window saturation semantic color
pub enum ContextSaturationColor {
    /// <60% saturation - Safe zone
    Safe,
    /// 60-85% saturation - Warning zone
    Warning,
    /// >85% saturation - Critical zone (near limit)
    Critical,
}

impl ContextSaturationColor {
    pub fn from_percentage(pct: f64) -> Self {
        if pct >= 85.0 {
            ContextSaturationColor::Critical
        } else if pct >= 60.0 {
            ContextSaturationColor::Warning
        } else {
            ContextSaturationColor::Safe
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            ContextSaturationColor::Safe => StatusColor::Success.to_color(), // Green
            ContextSaturationColor::Warning => StatusColor::Warning.to_color(), // Yellow
            ContextSaturationColor::Critical => StatusColor::Error.to_color(), // Red
        }
    }

    /// Warning icon for display (only for Warning/Critical)
    pub fn icon(&self) -> &'static str {
        match self {
            ContextSaturationColor::Safe => "",
            ContextSaturationColor::Warning => "⚠️",
            ContextSaturationColor::Critical => "🚨",
        }
    }
}

/// Staleness color based on time since last update
pub enum StalenessColor {
    /// <5s
    Fresh,
    /// 5s-30s
    Aging,
    /// >30s
    Stale,
}

impl StalenessColor {
    pub fn from_seconds(seconds: u64) -> Self {
        if seconds < 5 {
            StalenessColor::Fresh
        } else if seconds < 30 {
            StalenessColor::Aging
        } else {
            StalenessColor::Stale
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            StalenessColor::Fresh => StatusColor::Success.to_color(),
            StalenessColor::Aging => StatusColor::Warning.to_color(),
            StalenessColor::Stale => StatusColor::Error.to_color(),
        }
    }
}

/// Focus state colors
pub struct FocusStyle;

impl FocusStyle {
    /// Border color for focused pane
    pub fn focused_border() -> Color {
        StatusColor::Focus.to_color()
    }

    /// Border color for unfocused pane
    pub fn unfocused_border() -> Color {
        StatusColor::Neutral.to_color()
    }

    /// Background for focused item
    pub fn focused_bg() -> Color {
        Color::DarkGray
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_level_thresholds() {
        assert!(matches!(
            CostLevelColor::from_daily_cost(30.0),
            CostLevelColor::Low
        ));
        assert!(matches!(
            CostLevelColor::from_daily_cost(75.0),
            CostLevelColor::Medium
        ));
        assert!(matches!(
            CostLevelColor::from_daily_cost(150.0),
            CostLevelColor::High
        ));
    }

    #[test]
    fn test_usage_intensity_thresholds() {
        assert!(matches!(
            UsageIntensityColor::from_percentage(10.0),
            UsageIntensityColor::Low
        ));
        assert!(matches!(
            UsageIntensityColor::from_percentage(50.0),
            UsageIntensityColor::Medium
        ));
        assert!(matches!(
            UsageIntensityColor::from_percentage(85.0),
            UsageIntensityColor::High
        ));
    }

    #[test]
    fn test_context_saturation_thresholds() {
        assert!(matches!(
            ContextSaturationColor::from_percentage(45.0),
            ContextSaturationColor::Safe
        ));
        assert!(matches!(
            ContextSaturationColor::from_percentage(72.5),
            ContextSaturationColor::Warning
        ));
        assert!(matches!(
            ContextSaturationColor::from_percentage(91.0),
            ContextSaturationColor::Critical
        ));

        // Boundary tests
        assert!(matches!(
            ContextSaturationColor::from_percentage(59.9),
            ContextSaturationColor::Safe
        ));
        assert!(matches!(
            ContextSaturationColor::from_percentage(60.0),
            ContextSaturationColor::Warning
        ));
        assert!(matches!(
            ContextSaturationColor::from_percentage(85.0),
            ContextSaturationColor::Critical
        ));
    }

    #[test]
    fn test_context_saturation_icons() {
        assert_eq!(ContextSaturationColor::Safe.icon(), "");
        assert_eq!(ContextSaturationColor::Warning.icon(), "⚠️");
        assert_eq!(ContextSaturationColor::Critical.icon(), "🚨");
    }

    #[test]
    fn test_staleness_thresholds() {
        assert!(matches!(
            StalenessColor::from_seconds(3),
            StalenessColor::Fresh
        ));
        assert!(matches!(
            StalenessColor::from_seconds(15),
            StalenessColor::Aging
        ));
        assert!(matches!(
            StalenessColor::from_seconds(45),
            StalenessColor::Stale
        ));
    }

    #[test]
    fn test_server_status_icons() {
        assert_eq!(ServerStatusColor::Running.icon(), "●");
        assert_eq!(ServerStatusColor::Stopped.icon(), "○");
        assert_eq!(ServerStatusColor::Unknown.icon(), "?");
    }

    #[test]
    fn test_session_status_icons() {
        assert_eq!(SessionStatusColor::Active.icon(), "▶");
        assert_eq!(SessionStatusColor::Completed.icon(), "✓");
        assert_eq!(SessionStatusColor::Error.icon(), "✗");
    }
}
