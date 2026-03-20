//! Unified theme and color system for ccboard TUI
//!
//! Provides consistent color language across all tabs:
//! - 🟢 Green: Running, Healthy, Success
//! - 🔴 Red: Failed, Error, Critical
//! - 🟡 Yellow: Warning, Pending, Attention
//! - ⚪ Gray: Unknown, Disabled, Neutral
//! - 🔵 Cyan: Selected, Focus, Interactive
//! - 🟣 Magenta: High value, Important

use ccboard_core::models::config::ColorScheme;
use ratatui::style::Color;

/// Unified color palette bundle for a given color scheme.
///
/// Usage in render functions:
/// ```ignore
/// let p = Palette::new(scheme);
/// style.fg(p.fg).bg(p.bg)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Primary background — near-black with blue warmth
    pub bg: Color,
    /// Card/panel surface — one step above bg
    pub surface: Color,
    /// Primary foreground text — soft white
    pub fg: Color,
    /// Muted/secondary text — clearly distinct from border
    pub muted: Color,
    /// Unfocused border color — subtle but visible
    pub border: Color,
    /// Focus/accent color — desaturated cyan
    pub focus: Color,
    /// Success/healthy color
    pub success: Color,
    /// Error/critical color
    pub error: Color,
    /// Warning/pending color — warm amber
    pub warning: Color,
    /// Important/cost alert color — soft violet
    pub important: Color,
}

impl Palette {
    pub fn new(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Dark => Self {
                bg: Color::Rgb(13, 17, 23), // GitHub dark — near-black with blue warmth
                surface: Color::Rgb(22, 27, 34), // Card/panel layer above bg
                fg: Color::Rgb(220, 220, 235), // Soft white, less fatiguing
                muted: Color::Rgb(90, 95, 120), // Clearly distinct from border
                border: Color::Rgb(48, 54, 72), // Subtle but visible
                focus: Color::Rgb(80, 190, 210), // Desaturated cyan — readable without glare
                success: Color::Rgb(80, 200, 120), // Softer green
                error: Color::Rgb(220, 80, 80), // Slightly desaturated red
                warning: Color::Rgb(220, 175, 60), // Warm amber vs terminal yellow
                important: Color::Rgb(180, 100, 220), // Soft violet
            },
            ColorScheme::Light => Self {
                bg: Color::White,
                surface: Color::Rgb(245, 246, 248), // Slightly off-white for panels
                fg: Color::Black,
                muted: Color::Gray,
                border: Color::Gray,
                focus: Color::Rgb(0, 128, 128),
                success: Color::Rgb(0, 128, 0),
                error: Color::Rgb(200, 0, 0),
                warning: Color::Rgb(180, 120, 0),
                important: Color::Rgb(128, 0, 128),
            },
        }
    }
}

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
    /// Convert to Ratatui Color based on color scheme
    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Dark => match self {
                StatusColor::Success => Color::Rgb(80, 200, 120),
                StatusColor::Error => Color::Rgb(220, 80, 80),
                StatusColor::Warning => Color::Rgb(220, 175, 60),
                StatusColor::Neutral => Color::Rgb(90, 95, 120),
                StatusColor::Focus => Color::Rgb(80, 190, 210),
                StatusColor::Important => Color::Rgb(180, 100, 220),
            },
            ColorScheme::Light => match self {
                StatusColor::Success => Color::Rgb(0, 128, 0), // Dark green
                StatusColor::Error => Color::Rgb(200, 0, 0),   // Dark red
                StatusColor::Warning => Color::Rgb(180, 120, 0), // Dark yellow/orange
                StatusColor::Neutral => Color::Gray,
                StatusColor::Focus => Color::Rgb(0, 128, 128), // Dark cyan
                StatusColor::Important => Color::Rgb(128, 0, 128), // Dark magenta
            },
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
    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            ServerStatusColor::Running => StatusColor::Success.to_color(scheme),
            ServerStatusColor::Stopped => StatusColor::Error.to_color(scheme),
            ServerStatusColor::Unknown => StatusColor::Neutral.to_color(scheme),
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
    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            SessionStatusColor::Active => StatusColor::Focus.to_color(scheme),
            SessionStatusColor::Completed => StatusColor::Success.to_color(scheme),
            SessionStatusColor::Error => StatusColor::Error.to_color(scheme),
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
    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            HookEventColor::PreToolUse => StatusColor::Focus.to_color(scheme),
            HookEventColor::UserPromptSubmit => StatusColor::Success.to_color(scheme),
            HookEventColor::Other => StatusColor::Neutral.to_color(scheme),
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

    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            CostLevelColor::Low => StatusColor::Success.to_color(scheme),
            CostLevelColor::Medium => StatusColor::Warning.to_color(scheme),
            CostLevelColor::High => StatusColor::Error.to_color(scheme),
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

    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            UsageIntensityColor::Low => StatusColor::Success.to_color(scheme),
            UsageIntensityColor::Medium => StatusColor::Warning.to_color(scheme),
            UsageIntensityColor::High => StatusColor::Important.to_color(scheme),
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

    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            ContextSaturationColor::Safe => StatusColor::Success.to_color(scheme), // Green
            ContextSaturationColor::Warning => StatusColor::Warning.to_color(scheme), // Yellow
            ContextSaturationColor::Critical => StatusColor::Error.to_color(scheme), // Red
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

    pub fn to_color(self, scheme: ColorScheme) -> Color {
        match self {
            StalenessColor::Fresh => StatusColor::Success.to_color(scheme),
            StalenessColor::Aging => StatusColor::Warning.to_color(scheme),
            StalenessColor::Stale => StatusColor::Error.to_color(scheme),
        }
    }
}

/// Focus state colors
pub struct FocusStyle;

impl FocusStyle {
    /// Border color for focused pane
    pub fn focused_border(scheme: ColorScheme) -> Color {
        StatusColor::Focus.to_color(scheme)
    }

    /// Border color for unfocused pane
    pub fn unfocused_border(scheme: ColorScheme) -> Color {
        StatusColor::Neutral.to_color(scheme)
    }

    /// Background for focused item
    pub fn focused_bg(scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Dark => Color::Rgb(30, 36, 55), // Dark navy highlight
            ColorScheme::Light => Color::Rgb(220, 220, 220),
        }
    }
}

/// Base color helpers for backgrounds and foregrounds
pub struct BaseColors;

impl BaseColors {
    /// Primary background color
    pub fn bg(scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Dark => Color::Rgb(13, 17, 23),
            ColorScheme::Light => Color::White,
        }
    }

    /// Primary foreground/text color
    pub fn fg(scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Dark => Color::Rgb(220, 220, 235),
            ColorScheme::Light => Color::Black,
        }
    }

    /// Muted/secondary text color
    pub fn muted(scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Dark => Color::Rgb(90, 95, 120),
            ColorScheme::Light => Color::Gray,
        }
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
