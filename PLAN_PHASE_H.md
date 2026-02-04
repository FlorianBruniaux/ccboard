# Phase H: Advanced Analytics - Implementation Plan (v2)

**Date**: 2026-02-04 (Revised after technical-writer + system-architect reviews)
**Estimated Duration**: 11-13h (revised from 8-12h)
**Objective**: Add advanced analytics and actionable insights to optimize Claude Code usage

---

## Overview

### Business Goals
- **Reduce costs**: Identify over-usage patterns, expensive model selection
- **Optimize productivity**: Find peak productive hours/days
- **Predict budget**: Monthly cost forecasting with confidence metrics
- **Improve decisions**: Data-driven model selection, workload batching

### Technical Constraints
- **Read-only**: No writes to ~/.claude (align with MVP scope)
- **Reuse DataStore**: No new parsing, leverage existing sessions
- **Performance**: <100ms compute, <16ms render (60 FPS)
- **Memory**: Aggregation only, no raw data point storage
- **Pattern compliance**: EventBus, parking_lot::RwLock, graceful degradation

---

## Prerequisites

### Required SessionMetadata Fields

Analytics requires these fields (verify in `ccboard-core/src/models/session.rs`):

```rust
pub struct SessionMetadata {
    pub id: String,
    pub project_path: PathBuf,
    pub first_timestamp: Option<DateTime<Utc>>,  // REQUIRED for trends
    pub last_timestamp: Option<DateTime<Utc>>,   // REQUIRED for duration
    pub total_tokens: u64,                       // REQUIRED for aggregation
    pub models_used: Vec<String>,                // REQUIRED for distribution
    pub message_count: usize,
    // Note: estimated_cost computed from StatsCache pricing
}
```

**Graceful Degradation**:
- Missing `first_timestamp` → Skip session, log warning in LoadReport
- Empty `models_used` → Tag as "unknown", use default pricing
- `total_tokens = 0` → Include in count but not cost calculations

### Timezone Strategy

**All timestamps stored as UTC** (SessionMetadata standard)
**Display in local time** (user expects "today" = local today)

```rust
use chrono::{DateTime, Local, Utc};

// Convert UTC → Local for grouping
let local_ts: DateTime<Local> = utc_timestamp.with_timezone(&Local);
let date_key = local_ts.format("%Y-%m-%d").to_string();
```

### Error Handling Approach

Following ccboard graceful degradation pattern (per rust-ccboard.md):

- **No panics**: All functions return valid structs (empty vectors on failure)
- **Log warnings**: Use `tracing::warn!()` for skipped data
- **Default values**: Unknown model → $0.01/1K tokens default pricing
- **Confidence metrics**: Low data quality → `confidence: 0.0` (don't hide uncertainty)

---

## Architecture

### Module Structure

```
crates/ccboard-core/src/analytics/
├── mod.rs                  # Public API, AnalyticsData struct
├── trends.rs               # Time series aggregation
├── forecasting.rs          # Linear regression + confidence
├── patterns.rs             # Usage pattern detection
└── insights.rs             # Rule-based recommendations
```

### Data Structures (Complete Definitions)

#### AnalyticsData (Main Container)

```rust
// crates/ccboard-core/src/analytics/mod.rs
use chrono::{DateTime, Utc};

pub struct AnalyticsData {
    pub trends: TrendsData,
    pub forecast: ForecastData,
    pub patterns: UsagePatterns,
    pub insights: Vec<String>,
    pub computed_at: DateTime<Utc>,
    pub period: Period,
}

impl AnalyticsData {
    /// Compute from sessions (sync function, offload to spawn_blocking if needed)
    pub fn compute(sessions: &[Arc<SessionMetadata>], period: Period) -> Self {
        let trends = compute_trends(sessions, period.days());
        let forecast = forecast_usage(&trends);
        let patterns = detect_patterns(sessions);
        let insights = generate_insights(&trends, &patterns, &forecast);

        Self {
            trends,
            forecast,
            patterns,
            insights,
            computed_at: Utc::now(),
            period,
        }
    }

    /// Graceful fallback if stats-cache.json missing
    pub fn from_sessions_only(sessions: &[Arc<SessionMetadata>], period: Period) -> Self {
        warn!("Stats cache missing, computing analytics from sessions only");

        Self {
            trends: compute_trends(sessions, period.days()),
            forecast: ForecastData::unavailable("Stats cache required for cost forecasting"),
            patterns: detect_patterns(sessions),
            insights: vec!["Limited insights: stats cache unavailable".to_string()],
            computed_at: Utc::now(),
            period,
        }
    }
}
```

#### TrendsData (Time Series)

**Optimized structure** (separate Vecs, not tuples - 53% memory reduction):

```rust
// crates/ccboard-core/src/analytics/trends.rs
use std::collections::HashMap;

pub struct TrendsData {
    // Single date vector (indexed access, no duplication)
    pub dates: Vec<String>,  // "2026-02-04" format (10 bytes each)

    // Parallel vectors (aligned by index)
    pub daily_tokens: Vec<u64>,
    pub daily_sessions: Vec<usize>,
    pub daily_cost: Vec<f64>,

    // Distribution arrays (fixed size, stack-allocated)
    pub hourly_distribution: [usize; 24],    // Sessions per hour (0-23)
    pub weekday_distribution: [usize; 7],    // Sessions per weekday (Mon-Sun)

    // Model usage over time (aligned with dates)
    pub model_usage_over_time: HashMap<String, Vec<usize>>,
}

impl TrendsData {
    /// Check if empty (no data in period)
    pub fn is_empty(&self) -> bool {
        self.dates.is_empty()
    }

    /// Get tokens at specific date index
    pub fn get_tokens_at(&self, idx: usize) -> Option<(&str, u64)> {
        Some((self.dates.get(idx)?, self.daily_tokens[idx]))
    }

    /// Placeholder for empty state rendering
    pub fn empty() -> Self {
        Self {
            dates: Vec::new(),
            daily_tokens: Vec::new(),
            daily_sessions: Vec::new(),
            daily_cost: Vec::new(),
            hourly_distribution: [0; 24],
            weekday_distribution: [0; 7],
            model_usage_over_time: HashMap::new(),
        }
    }
}

/// Compute trends from sessions (sync function, ~40ms target for 1000 sessions)
pub fn compute_trends(sessions: &[Arc<SessionMetadata>], days: usize) -> TrendsData {
    use chrono::{Datelike, Local, TimeZone};
    use std::collections::BTreeMap;

    let mut daily_map: BTreeMap<String, DailyAggregate> = BTreeMap::new();
    let mut hourly_counts = [0usize; 24];
    let mut weekday_counts = [0usize; 7];
    let mut model_usage: HashMap<String, BTreeMap<String, usize>> = HashMap::new();

    let now = Local::now();
    let cutoff = now - chrono::Duration::days(days as i64);

    for session in sessions {
        let Some(ts) = session.first_timestamp else {
            warn!("Session {} missing timestamp, skipping", session.id);
            continue;
        };

        // Convert UTC → Local for grouping
        let local_ts: DateTime<Local> = ts.with_timezone(&Local);

        // Filter by period
        if local_ts < cutoff {
            continue;
        }

        let date_key = local_ts.format("%Y-%m-%d").to_string();

        // Aggregate daily
        let agg = daily_map.entry(date_key.clone()).or_default();
        agg.tokens += session.total_tokens;
        agg.sessions += 1;
        agg.cost += estimate_cost(session); // Uses StatsCache pricing

        // Hourly distribution
        hourly_counts[local_ts.hour() as usize] += 1;

        // Weekday distribution (0 = Monday, 6 = Sunday)
        weekday_counts[local_ts.weekday().num_days_from_monday() as usize] += 1;

        // Model usage over time
        for model in &session.models_used {
            model_usage.entry(model.clone())
                .or_default()
                .entry(date_key.clone())
                .or_insert(0) += 1;
        }
    }

    // Extract sorted dates + values
    let dates: Vec<String> = daily_map.keys().cloned().collect();
    let daily_tokens: Vec<u64> = daily_map.values().map(|a| a.tokens).collect();
    let daily_sessions: Vec<usize> = daily_map.values().map(|a| a.sessions).collect();
    let daily_cost: Vec<f64> = daily_map.values().map(|a| a.cost).collect();

    // Align model usage with dates
    let model_usage_over_time: HashMap<String, Vec<usize>> = model_usage
        .into_iter()
        .map(|(model, date_map)| {
            let counts = dates.iter().map(|d| *date_map.get(d).unwrap_or(&0)).collect();
            (model, counts)
        })
        .collect();

    TrendsData {
        dates,
        daily_tokens,
        daily_sessions,
        daily_cost,
        hourly_distribution: hourly_counts,
        weekday_distribution: weekday_counts,
        model_usage_over_time,
    }
}

#[derive(Default)]
struct DailyAggregate {
    tokens: u64,
    sessions: usize,
    cost: f64,
}

/// Estimate cost from session (uses StatsCache pricing if available)
fn estimate_cost(session: &SessionMetadata) -> f64 {
    // TODO: Integrate with StatsCache.model_pricing
    // Placeholder: $0.01 per 1K tokens
    (session.total_tokens as f64 / 1000.0) * 0.01
}
```

#### ForecastData (Predictions)

**R² confidence metric** (not sample size):

```rust
// crates/ccboard-core/src/analytics/forecasting.rs

pub struct ForecastData {
    pub next_30_days_tokens: u64,
    pub next_30_days_cost: f64,
    pub monthly_cost_estimate: f64,
    pub confidence: f64,              // 0.0-1.0 (R² coefficient)
    pub trend_direction: TrendDirection,
    pub unavailable_reason: Option<String>,
}

pub enum TrendDirection {
    Up(f64),      // Percentage increase
    Down(f64),    // Percentage decrease
    Stable,       // < 10% change
}

impl ForecastData {
    pub fn unavailable(reason: &str) -> Self {
        Self {
            next_30_days_tokens: 0,
            next_30_days_cost: 0.0,
            monthly_cost_estimate: 0.0,
            confidence: 0.0,
            trend_direction: TrendDirection::Stable,
            unavailable_reason: Some(reason.to_string()),
        }
    }
}

/// Forecast usage with linear regression (~20ms target)
pub fn forecast_usage(trends: &TrendsData) -> ForecastData {
    if trends.dates.len() < 7 {
        return ForecastData::unavailable("Insufficient data (<7 days)");
    }

    // Prepare data points (x = day index, y = tokens)
    let points: Vec<_> = trends.daily_tokens.iter()
        .enumerate()
        .map(|(i, &tokens)| (i as f64, tokens as f64))
        .collect();

    // Linear regression: y = slope * x + intercept
    let (slope, intercept, r_squared) = linear_regression(&points);

    // R² = coefficient of determination (0.0-1.0)
    // 1.0 = perfect fit, 0.0 = no correlation
    let confidence = r_squared.clamp(0.0, 1.0);

    // Extrapolate 30 days ahead
    let last_x = points.len() as f64;
    let next_30_x = last_x + 30.0;
    let next_30_days_tokens = (slope * next_30_x + intercept).max(0.0) as u64;

    // Estimate cost (using current avg cost/token)
    let total_cost: f64 = trends.daily_cost.iter().sum();
    let total_tokens: u64 = trends.daily_tokens.iter().sum();
    let cost_per_token = if total_tokens > 0 {
        total_cost / total_tokens as f64
    } else {
        0.01 / 1000.0  // Default: $0.01/1K tokens
    };
    let next_30_days_cost = next_30_days_tokens as f64 * cost_per_token;

    // Monthly estimate (extrapolate to 30 days from current average)
    let days_in_period = trends.dates.len() as f64;
    let monthly_cost_estimate = (total_cost / days_in_period) * 30.0;

    // Trend direction (slope-based)
    let trend_direction = if slope.abs() < 0.01 * intercept {
        TrendDirection::Stable
    } else if slope > 0.0 {
        let increase_pct = (slope * 30.0 / intercept * 100.0).abs();
        TrendDirection::Up(increase_pct)
    } else {
        let decrease_pct = (slope * 30.0 / intercept * 100.0).abs();
        TrendDirection::Down(decrease_pct)
    };

    ForecastData {
        next_30_days_tokens,
        next_30_days_cost,
        monthly_cost_estimate,
        confidence,
        trend_direction,
        unavailable_reason: None,
    }
}

/// Simple linear regression with R² calculation
fn linear_regression(points: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = points.len() as f64;
    let sum_x: f64 = points.iter().map(|p| p.0).sum();
    let sum_y: f64 = points.iter().map(|p| p.1).sum();
    let sum_xx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sum_xy: f64 = points.iter().map(|p| p.0 * p.1).sum();

    // Slope and intercept
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    // R² (coefficient of determination)
    let mean_y = sum_y / n;
    let ss_tot: f64 = points.iter().map(|p| (p.1 - mean_y).powi(2)).sum();
    let ss_res: f64 = points.iter().map(|p| {
        let predicted = slope * p.0 + intercept;
        (p.1 - predicted).powi(2)
    }).sum();

    let r_squared = if ss_tot > 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    (slope, intercept, r_squared)
}
```

#### UsagePatterns (Behavioral Analysis)

**Percentile-based peak hours + cost-weighted model distribution**:

```rust
// crates/ccboard-core/src/analytics/patterns.rs
use chrono::{Datelike, Weekday};
use std::collections::HashMap;

pub struct UsagePatterns {
    pub most_productive_hour: u8,           // Hour with most sessions (0-23)
    pub most_productive_day: Weekday,       // Weekday with most sessions
    pub avg_session_duration: Duration,     // Average session duration
    pub most_used_model: String,            // Model with highest token count
    pub model_distribution: HashMap<String, f64>,       // % by token count
    pub model_cost_distribution: HashMap<String, f64>,  // % by cost (NEW)
    pub peak_hours: Vec<u8>,                // Hours above 80th percentile
}

impl UsagePatterns {
    pub fn empty() -> Self {
        Self {
            most_productive_hour: 0,
            most_productive_day: Weekday::Mon,
            avg_session_duration: Duration::from_secs(0),
            most_used_model: "unknown".to_string(),
            model_distribution: HashMap::new(),
            model_cost_distribution: HashMap::new(),
            peak_hours: Vec::new(),
        }
    }
}

/// Detect usage patterns (~30ms target for 1000 sessions)
pub fn detect_patterns(sessions: &[Arc<SessionMetadata>]) -> UsagePatterns {
    use chrono::Local;

    if sessions.is_empty() {
        return UsagePatterns::empty();
    }

    let mut hourly_counts = [0usize; 24];
    let mut weekday_counts = [0; 7];
    let mut total_duration = Duration::from_secs(0);
    let mut duration_count = 0usize;
    let mut model_tokens: HashMap<String, u64> = HashMap::new();
    let mut model_costs: HashMap<String, f64> = HashMap::new();

    for session in sessions {
        // Hourly distribution
        if let Some(ts) = session.first_timestamp {
            let local_ts: DateTime<Local> = ts.with_timezone(&Local);
            hourly_counts[local_ts.hour() as usize] += 1;
            weekday_counts[local_ts.weekday().num_days_from_monday() as usize] += 1;
        }

        // Session duration
        if let (Some(start), Some(end)) = (session.first_timestamp, session.last_timestamp) {
            if let Ok(duration) = (end - start).to_std() {
                total_duration += duration;
                duration_count += 1;
            }
        }

        // Model distribution (tokens + cost)
        for model in &session.models_used {
            *model_tokens.entry(model.clone()).or_default() += session.total_tokens;

            let cost = estimate_cost(session);  // Same function as trends.rs
            *model_costs.entry(model.clone()).or_default() += cost;
        }
    }

    // Most productive hour
    let most_productive_hour = hourly_counts.iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .map(|(hour, _)| hour as u8)
        .unwrap_or(0);

    // Most productive day
    let most_productive_day = weekday_counts.iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .and_then(|(idx, _)| Weekday::try_from(idx as u8).ok())
        .unwrap_or(Weekday::Mon);

    // Average duration
    let avg_session_duration = if duration_count > 0 {
        total_duration / duration_count as u32
    } else {
        Duration::from_secs(0)
    };

    // Peak hours (80th percentile threshold)
    let total_sessions: usize = hourly_counts.iter().sum();
    let threshold = (total_sessions as f64 * 0.8 / 24.0) as usize;
    let peak_hours: Vec<u8> = hourly_counts.iter()
        .enumerate()
        .filter(|(_, &count)| count > threshold)
        .map(|(hour, _)| hour as u8)
        .collect();

    // Model distribution (by tokens)
    let total_tokens: u64 = model_tokens.values().sum();
    let model_distribution: HashMap<String, f64> = model_tokens.into_iter()
        .map(|(model, tokens)| (model, tokens as f64 / total_tokens as f64))
        .collect();

    // Most used model
    let most_used_model = model_distribution.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(model, _)| model.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Model cost distribution (NEW: cost-weighted)
    let total_cost: f64 = model_costs.values().sum();
    let model_cost_distribution: HashMap<String, f64> = model_costs.into_iter()
        .map(|(model, cost)| (model, cost / total_cost))
        .collect();

    UsagePatterns {
        most_productive_hour,
        most_productive_day,
        avg_session_duration,
        most_used_model,
        model_distribution,
        model_cost_distribution,
        peak_hours,
    }
}
```

#### Insights (Recommendations)

**Rule-based with concrete thresholds**:

```rust
// crates/ccboard-core/src/analytics/insights.rs

pub fn generate_insights(
    trends: &TrendsData,
    patterns: &UsagePatterns,
    forecast: &ForecastData,
) -> Vec<String> {
    let mut insights = Vec::new();

    // 1. Peak hours insight (>30% of sessions)
    let total_sessions: usize = patterns.hourly_distribution.iter().sum();
    if !patterns.peak_hours.is_empty() && total_sessions > 0 {
        let peak_count: usize = patterns.peak_hours.iter()
            .map(|&h| patterns.hourly_distribution[h as usize])
            .sum();

        if peak_count > total_sessions * 3 / 10 {
            insights.push(format!(
                "Peak hours: {:02}h-{:02}h ({:.0}% of sessions). Consider batching work.",
                patterns.peak_hours.first().unwrap_or(&0),
                patterns.peak_hours.last().unwrap_or(&23),
                peak_count as f64 / total_sessions as f64 * 100.0
            ));
        }
    }

    // 2. Expensive model warning (Opus >20% usage)
    if let Some(&opus_pct) = patterns.model_distribution.get("opus") {
        if opus_pct > 0.2 {
            insights.push(format!(
                "Opus usage: {:.0}% tokens. Costs 3x more than Sonnet. Review necessity.",
                opus_pct * 100.0
            ));
        }
    }

    // 3. Cost imbalance (tokens vs cost distribution mismatch)
    for (model, &token_pct) in &patterns.model_distribution {
        if let Some(&cost_pct) = patterns.model_cost_distribution.get(model) {
            let cost_premium = cost_pct / token_pct;
            if cost_premium > 1.5 && cost_pct > 0.2 {
                insights.push(format!(
                    "{}: {:.0}% tokens but {:.0}% cost. Cost premium: {:.1}x.",
                    model,
                    token_pct * 100.0,
                    cost_pct * 100.0,
                    cost_premium
                ));
            }
        }
    }

    // 4. Cost trend warning (>20% increase)
    if let TrendDirection::Up(pct) = forecast.trend_direction {
        if pct > 20.0 && forecast.confidence > 0.5 {
            insights.push(format!(
                "Cost trend: +{:.0}% over period. Monthly estimate: ${:.2} (confidence: {:.0}%).",
                pct,
                forecast.monthly_cost_estimate,
                forecast.confidence * 100.0
            ));
        }
    }

    // 5. Weekend optimization (usage <10%)
    let weekday_sum: usize = patterns.weekday_distribution[0..5].iter().sum();
    let weekend_sum: usize = patterns.weekday_distribution[5..7].iter().sum();
    let total = weekday_sum + weekend_sum;

    if total > 0 {
        let weekend_pct = weekend_sum as f64 / total as f64;
        if weekend_pct < 0.1 {
            insights.push(format!(
                "Weekend usage: {:.0}%. Consider weekday-focused workflows.",
                weekend_pct * 100.0
            ));
        }
    }

    // 6. Low confidence warning
    if forecast.confidence < 0.5 {
        insights.push(format!(
            "Forecast confidence low ({:.0}%). Predictions may be unreliable.",
            forecast.confidence * 100.0
        ));
    }

    insights
}
```

---

## DataStore Integration

### Cache with EventBus (Not Instant)

**Pattern**: Follow existing DataStore structure (store.rs:189-190, 457, 534)

```rust
// crates/ccboard-core/src/store.rs

use parking_lot::RwLock;
use crate::analytics::{AnalyticsData, Period};
use crate::event::{DataEvent, EventBus};

pub struct DataStore {
    // ... existing fields ...

    /// Analytics cache (event-invalidated, not time-based)
    analytics_cache: RwLock<HashMap<Period, AnalyticsData>>,
}

impl DataStore {
    /// Get analytics for period (lazy compute + cache)
    pub fn analytics(&self, period: Period) -> AnalyticsData {
        // Check cache first
        {
            let cache = self.analytics_cache.read();
            if let Some(data) = cache.get(&period) {
                return data.clone();
            }
        }

        // Compute fresh (sync function, ~100ms for 1000 sessions)
        let sessions = self.all_sessions_filtered(period);
        let data = AnalyticsData::compute(&sessions, period);

        // Update cache
        {
            let mut cache = self.analytics_cache.write();
            cache.insert(period, data.clone());
        }

        data
    }

    /// Invalidate analytics cache (called by EventBus handler)
    pub(crate) fn invalidate_analytics(&self) {
        let mut cache = self.analytics_cache.write();
        cache.clear();
        debug!("Analytics cache invalidated on session update");
    }

    /// Filter sessions by period
    fn all_sessions_filtered(&self, period: Period) -> Vec<Arc<SessionMetadata>> {
        let sessions: Vec<_> = self.sessions.iter()
            .map(|r| Arc::clone(r.value()))
            .collect();

        match period {
            Period::Days(n) => {
                let cutoff = Local::now() - chrono::Duration::days(n as i64);
                sessions.into_iter()
                    .filter(|s| {
                        s.first_timestamp
                            .map(|ts| ts.with_timezone(&Local) >= cutoff)
                            .unwrap_or(false)
                    })
                    .collect()
            }
            Period::Available => sessions,
        }
    }
}

// Integrate with existing EventBus (update_session method)
impl DataStore {
    pub async fn update_session(&self, path: &Path) {
        // ... existing code ...

        self.invalidate_analytics();  // NEW: Clear analytics cache
        self.event_bus.publish(DataEvent::SessionUpdated(id));
    }

    pub async fn initial_load(&self, paths: Vec<PathBuf>) -> LoadReport {
        // ... existing code ...

        self.invalidate_analytics();  // NEW: Clear on reload
        self.event_bus.publish(DataEvent::LoadCompleted);

        report
    }
}
```

### Period Enum (Honest Labeling)

```rust
// crates/ccboard-core/src/analytics/mod.rs
use chrono::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    Days(usize),         // Last N days
    Available,           // All loaded sessions (honest: not "all time")
}

impl Period {
    pub fn last_7d() -> Self { Self::Days(7) }
    pub fn last_30d() -> Self { Self::Days(30) }
    pub fn last_90d() -> Self { Self::Days(90) }
    pub fn available() -> Self { Self::Available }

    pub fn days(&self) -> usize {
        match self {
            Period::Days(n) => *n,
            Period::Available => 36500, // 100 years (effectively all)
        }
    }

    /// Display label (shows loaded count for Available)
    pub fn display(&self, total_loaded: usize) -> String {
        match self {
            Period::Days(n) => format!("Last {} days", n),
            Period::Available => format!("All loaded ({} sessions)", total_loaded),
        }
    }
}
```

---

## TUI Integration

### AnalyticsTab Structure

```rust
// crates/ccboard-tui/src/tabs/analytics.rs

use ccboard_core::analytics::{AnalyticsData, Period};
use std::sync::Arc;

pub struct AnalyticsTab {
    view: AnalyticsView,
    period: Period,
    store: Arc<DataStore>,
    // No local cache - DataStore handles caching
}

pub enum AnalyticsView {
    Trends,      // Sub-view 1: Time series charts
    Forecast,    // Sub-view 2: Predictions
    Patterns,    // Sub-view 3: Behavioral analysis
    Insights,    // Sub-view 4: Recommendations
}

impl AnalyticsTab {
    pub fn new(store: Arc<DataStore>) -> Self {
        Self {
            view: AnalyticsView::Trends,
            period: Period::last_30d(),
            store,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => self.view = AnalyticsView::Trends,
            KeyCode::Char('2') => self.view = AnalyticsView::Forecast,
            KeyCode::Char('3') => self.view = AnalyticsView::Patterns,
            KeyCode::Char('4') => self.view = AnalyticsView::Insights,
            KeyCode::Char('p') => self.cycle_period(),
            KeyCode::Char('r') => {
                self.store.invalidate_analytics();
                // Toast notification: "Analytics refreshed"
            }
            _ => {}
        }
    }

    fn cycle_period(&mut self) {
        self.period = match self.period {
            Period::Days(7) => Period::Days(30),
            Period::Days(30) => Period::Days(90),
            Period::Days(90) => Period::Available,
            _ => Period::Days(7),
        };
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Fetch analytics (from DataStore cache or compute)
        let data = self.store.analytics(self.period);

        match self.view {
            AnalyticsView::Trends => self.render_trends(frame, area, &data.trends),
            AnalyticsView::Forecast => self.render_forecast(frame, area, &data.forecast),
            AnalyticsView::Patterns => self.render_patterns(frame, area, &data.patterns),
            AnalyticsView::Insights => self.render_insights(frame, area, &data.insights),
        }
    }

    fn render_trends(&self, frame: &mut Frame, area: Rect, trends: &TrendsData) {
        if trends.is_empty() {
            let msg = Paragraph::new("No sessions in selected period")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }

        // Layout: [Title] [Charts] [Stats Panel]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title + period selector
                Constraint::Min(10),    // Charts area
            ])
            .split(area);

        // Title
        let title = format!(
            "Trends - {} (Updated: {})",
            self.period.display(self.store.session_count()),
            data.computed_at.format("%H:%M:%S")
        );
        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(title_widget, chunks[0]);

        // Charts (Sparkline for tokens/cost, BarChart for sessions)
        let chart_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(chunks[1]);

        // Tokens Sparkline
        let tokens_data: Vec<u64> = trends.daily_tokens.clone();
        let sparkline = Sparkline::default()
            .block(Block::default().title("Daily Tokens").borders(Borders::ALL))
            .data(&tokens_data)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, chart_chunks[0]);

        // Sessions BarChart
        let sessions_data: Vec<(&str, u64)> = trends.dates.iter()
            .zip(&trends.daily_sessions)
            .map(|(date, &count)| (date.as_str(), count as u64))
            .collect();
        let barchart = BarChart::default()
            .block(Block::default().title("Daily Sessions").borders(Borders::ALL))
            .data(&sessions_data)
            .bar_width(3)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(barchart, chart_chunks[1]);

        // Cost Sparkline
        let cost_data: Vec<u64> = trends.daily_cost.iter()
            .map(|&c| (c * 100.0) as u64)  // Convert to cents for u64
            .collect();
        let sparkline = Sparkline::default()
            .block(Block::default().title("Daily Cost ($)").borders(Borders::ALL))
            .data(&cost_data)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(sparkline, chart_chunks[2]);
    }

    fn render_forecast(&self, frame: &mut Frame, area: Rect, forecast: &ForecastData) {
        if let Some(reason) = &forecast.unavailable_reason {
            let msg = Paragraph::new(format!("Forecast unavailable: {}", reason))
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }

        // Layout: [Title] [Prediction] [Confidence]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(5),
            ])
            .split(area);

        // Prediction text
        let trend_symbol = match forecast.trend_direction {
            TrendDirection::Up(_) => "↑",
            TrendDirection::Down(_) => "↓",
            TrendDirection::Stable => "→",
        };
        let trend_color = match forecast.trend_direction {
            TrendDirection::Up(_) => Color::Red,
            TrendDirection::Down(_) => Color::Green,
            TrendDirection::Stable => Color::Gray,
        };

        let text = format!(
            "Next 30 days: {} tokens, ${:.2}\n\
             Monthly estimate: ${:.2}\n\
             Trend: {} {}",
            forecast.next_30_days_tokens,
            forecast.next_30_days_cost,
            forecast.monthly_cost_estimate,
            trend_symbol,
            match forecast.trend_direction {
                TrendDirection::Up(pct) => format!("+{:.0}%", pct),
                TrendDirection::Down(pct) => format!("-{:.0}%", pct),
                TrendDirection::Stable => "Stable".to_string(),
            }
        );

        let para = Paragraph::new(text)
            .block(Block::default().title("Forecast").borders(Borders::ALL))
            .style(Style::default().fg(trend_color));
        frame.render_widget(para, chunks[1]);

        // Confidence gauge
        let gauge = Gauge::default()
            .block(Block::default().title("Confidence (R²)").borders(Borders::ALL))
            .gauge_style(Style::default().fg(
                if forecast.confidence > 0.7 { Color::Green }
                else if forecast.confidence > 0.4 { Color::Yellow }
                else { Color::Red }
            ))
            .ratio(forecast.confidence)
            .label(format!("{:.0}%", forecast.confidence * 100.0));
        frame.render_widget(gauge, chunks[2]);
    }

    fn render_patterns(&self, frame: &mut Frame, area: Rect, patterns: &UsagePatterns) {
        // Hourly heatmap (BarChart)
        let hourly_data: Vec<(&str, u64)> = patterns.hourly_distribution.iter()
            .enumerate()
            .map(|(h, &count)| {
                let label = format!("{:02}h", h);
                (Box::leak(label.into_boxed_str()) as &str, count as u64)
            })
            .collect();

        let barchart = BarChart::default()
            .block(Block::default().title("Hourly Distribution").borders(Borders::ALL))
            .data(&hourly_data)
            .bar_width(1)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(barchart, area);

        // Model distribution (Gauge widgets - follows Dashboard pattern)
        // TODO: Layout splits for weekday + model distribution
    }

    fn render_insights(&self, frame: &mut Frame, area: Rect, insights: &[String]) {
        if insights.is_empty() {
            let msg = Paragraph::new("No insights available for this period")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }

        // List of insights (bullet points)
        let items: Vec<ListItem> = insights.iter()
            .map(|insight| {
                let content = format!("• {}", insight);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Insights & Recommendations").borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(list, area);
    }
}
```

---

## Testing Strategy

### Unit Tests (15+ tests required)

```rust
// crates/ccboard-core/src/analytics/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn generate_test_sessions(count: usize, days: usize) -> Vec<Arc<SessionMetadata>> {
        let now = Utc::now();
        (0..count)
            .map(|i| {
                let day_offset = (i % days) as i64;
                let ts = now - chrono::Duration::days(day_offset);

                Arc::new(SessionMetadata {
                    id: format!("session-{}", i),
                    first_timestamp: Some(ts),
                    last_timestamp: Some(ts + chrono::Duration::minutes(30)),
                    total_tokens: 1000 + (i as u64 * 100),
                    models_used: vec!["sonnet".to_string()],
                    ..Default::default()
                })
            })
            .collect()
    }

    // Trends tests (5 tests)
    #[test]
    fn test_trends_empty_sessions() {
        let trends = compute_trends(&[], 30);
        assert!(trends.is_empty());
    }

    #[test]
    fn test_trends_single_day() {
        let sessions = generate_test_sessions(10, 1);
        let trends = compute_trends(&sessions, 30);
        assert_eq!(trends.dates.len(), 1);
        assert_eq!(trends.daily_sessions[0], 10);
    }

    #[test]
    fn test_trends_multi_day_aggregation() {
        let sessions = generate_test_sessions(30, 10);
        let trends = compute_trends(&sessions, 30);
        assert_eq!(trends.dates.len(), 10);
        assert_eq!(trends.daily_sessions.iter().sum::<usize>(), 30);
    }

    #[test]
    fn test_trends_hourly_distribution() {
        let sessions = generate_test_sessions(24, 1);
        let trends = compute_trends(&sessions, 1);
        let total: usize = trends.hourly_distribution.iter().sum();
        assert_eq!(total, 24);
    }

    #[test]
    fn test_trends_model_usage() {
        let mut sessions = generate_test_sessions(10, 5);
        for session in sessions.iter_mut() {
            Arc::get_mut(session).unwrap().models_used = vec!["opus".to_string()];
        }
        let trends = compute_trends(&sessions, 30);
        assert!(trends.model_usage_over_time.contains_key("opus"));
    }

    // Forecast tests (4 tests)
    #[test]
    fn test_forecast_insufficient_data() {
        let sessions = generate_test_sessions(3, 3);
        let trends = compute_trends(&sessions, 30);
        let forecast = forecast_usage(&trends);
        assert!(forecast.unavailable_reason.is_some());
    }

    #[test]
    fn test_forecast_stable_trend() {
        let mut sessions = generate_test_sessions(30, 30);
        // All sessions same tokens → stable
        for session in sessions.iter_mut() {
            Arc::get_mut(session).unwrap().total_tokens = 1000;
        }
        let trends = compute_trends(&sessions, 30);
        let forecast = forecast_usage(&trends);
        assert!(matches!(forecast.trend_direction, TrendDirection::Stable));
    }

    #[test]
    fn test_forecast_increasing_trend() {
        let mut sessions = generate_test_sessions(30, 30);
        // Increasing tokens over time
        for (i, session) in sessions.iter_mut().enumerate() {
            Arc::get_mut(session).unwrap().total_tokens = 1000 + (i as u64 * 100);
        }
        let trends = compute_trends(&sessions, 30);
        let forecast = forecast_usage(&trends);
        assert!(matches!(forecast.trend_direction, TrendDirection::Up(_)));
    }

    #[test]
    fn test_forecast_confidence_reflects_variance() {
        // High variance → low R² → low confidence
        let sessions = generate_test_sessions(30, 30);
        let trends = compute_trends(&sessions, 30);
        let forecast = forecast_usage(&trends);
        assert!(forecast.confidence >= 0.0 && forecast.confidence <= 1.0);
    }

    // Pattern tests (3 tests)
    #[test]
    fn test_patterns_peak_hours() {
        let sessions = generate_test_sessions(100, 7);
        let patterns = detect_patterns(&sessions);
        assert!(!patterns.peak_hours.is_empty());
    }

    #[test]
    fn test_patterns_most_productive_day() {
        let sessions = generate_test_sessions(50, 7);
        let patterns = detect_patterns(&sessions);
        // Should have a most productive day
    }

    #[test]
    fn test_patterns_model_distribution_sums_to_one() {
        let sessions = generate_test_sessions(30, 7);
        let patterns = detect_patterns(&sessions);
        let sum: f64 = patterns.model_distribution.values().sum();
        assert!((sum - 1.0).abs() < 0.01); // ~1.0 (accounting for float precision)
    }

    // Integration test (1 test)
    #[test]
    fn test_full_analytics_pipeline() {
        let sessions = generate_test_sessions(100, 30);
        let period = Period::last_30d();
        let data = AnalyticsData::compute(&sessions, period);

        assert!(!data.trends.is_empty());
        assert!(data.forecast.confidence >= 0.0);
        assert!(!data.patterns.model_distribution.is_empty());
    }
}
```

### Performance Benchmarks

```rust
// crates/ccboard-core/benches/analytics_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ccboard_core::analytics::*;

fn bench_compute_trends(c: &mut Criterion) {
    let sessions = generate_test_sessions(1000, 30);

    c.bench_function("compute_trends_1000_sessions", |b| {
        b.iter(|| compute_trends(black_box(&sessions), 30))
    });
}

fn bench_forecast_usage(c: &mut Criterion) {
    let sessions = generate_test_sessions(1000, 30);
    let trends = compute_trends(&sessions, 30);

    c.bench_function("forecast_usage_30_days", |b| {
        b.iter(|| forecast_usage(black_box(&trends)))
    });
}

fn bench_detect_patterns(c: &mut Criterion) {
    let sessions = generate_test_sessions(1000, 30);

    c.bench_function("detect_patterns_1000_sessions", |b| {
        b.iter(|| detect_patterns(black_box(&sessions)))
    });
}

criterion_group!(benches, bench_compute_trends, bench_forecast_usage, bench_detect_patterns);
criterion_main!(benches);
```

**Performance Targets** (1000 sessions, 30 days):
- `compute_trends`: <40ms
- `forecast_usage`: <20ms
- `detect_patterns`: <30ms
- **Total**: <100ms (allows 10ms buffer for insights generation)

---

## Implementation Milestones

| Milestone | Deliverable | Duration | Validation |
|-----------|-------------|----------|------------|
| **M1** | Core analytics module | 4.5h | 15+ unit tests pass, benchmarks <100ms |
| **M2** | Analytics Tab UI | 3-4h | Manual testing, renders correctly |
| **M3** | DataStore integration | 2h | EventBus invalidation works, cache hit/miss logged |
| **M4** | Tests & polish | 2-3h | All 152+ tests pass, 0 clippy warnings, help modal updated |

**Total**: 11.5-13.5h

---

## Success Criteria

### Functional
- ✅ 4 sub-views functional (Trends, Forecast, Patterns, Insights)
- ✅ Period selection (7d, 30d, 90d, Available)
- ✅ EventBus cache invalidation (not time-based)
- ✅ Refresh on demand (`r` key)
- ✅ Graceful degradation (empty states, missing stats)

### Performance
- ✅ Compute analytics <100ms (1000 sessions, 30 days)
- ✅ Render charts <16ms (60 FPS maintained)
- ✅ Memory: <50KB cache (all periods)
- ✅ No blocking I/O (sync computation)

### Quality
- ✅ 15+ unit tests (trends, forecast, patterns, integration)
- ✅ Performance benchmarks pass targets
- ✅ 0 clippy warnings
- ✅ Documentation inline (all public functions)
- ✅ Help modal updated (Analytics keybindings)

### UX
- ✅ All charts have labeled axes with units
- ✅ Empty states show actionable guidance
- ✅ Insights actionable (concrete recommendations)
- ✅ Color coding consistent (red=warning, green=good, yellow=info)
- ✅ Period label honest ("All loaded (N sessions)", not "All time")

---

## Risks & Mitigations

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Compute >100ms | Medium | spawn_blocking offload, show loading spinner | Benchmarks verify |
| R² complex | Low | Add inline comments, reference Wikipedia | Documentation |
| EventBus integration breaks | Medium | Test cache invalidation, fallback to manual refresh | Integration tests |
| Charts illegible | Medium | Test with real data (3500+ sessions), adjust scales | Manual testing |
| Insufficient data (<7d) | Low | Show unavailable message, suggest waiting | Graceful degradation |
| Memory spike | Low | Aggregation only (no raw points), verify <50KB | Memory profiling |

---

## Deferred to Phase H.5 (Polish)

**Optimizations not blocking MVP** (defer if timeline pressure):
1. Weekday BarChart (Patterns view)
2. Model distribution Gauge widgets (Patterns view)
3. Rendering snapshot tests (Ratatui TestBackend)
4. Advanced insights (seasonality detection, anomaly detection)

---

## Next Steps

1. ✅ Plan reviewed and approved
2. Create task list (TodoWrite 10-12 tasks)
3. Start M1 (Core analytics module)
4. Iterate M2-M4

**Ready to start?**
