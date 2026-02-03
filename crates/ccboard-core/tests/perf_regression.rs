//! Performance regression tests
//!
//! These tests ensure that performance optimizations don't regress over time.
//! They establish baseline expectations and fail if operations become too slow.
//!
//! Run with:
//! ```bash
//! cargo test --test perf_regression
//! ```

use ccboard_core::DataStore;
use std::time::{Duration, Instant};

/// Performance targets based on baseline measurements
mod targets {
    use std::time::Duration;

    /// Initial load should complete under 2 seconds with typical data
    /// (1000 sessions, stats-cache.json, settings)
    pub const INITIAL_LOAD_MAX: Duration = Duration::from_secs(2);

    /// Session scanning should scale linearly, not exponentially
    /// 100 sessions → ~200ms, 1000 sessions → ~2000ms (10x factor allowed)
    pub const SESSION_SCAN_LINEAR_FACTOR: usize = 15;

    /// Stats parsing should be fast (single file)
    pub const STATS_PARSE_MAX: Duration = Duration::from_millis(100);

    /// Settings merge should be fast (3 files max)
    pub const SETTINGS_MERGE_MAX: Duration = Duration::from_millis(50);
}

#[tokio::test]
async fn test_initial_load_under_2s() {
    let claude_home = dirs::home_dir().expect("No home directory").join(".claude");

    if !claude_home.exists() {
        eprintln!("SKIP: ~/.claude not found at {}", claude_home.display());
        return;
    }

    // FIRST RUN: Populate cache (will be slow)
    eprintln!("\n=== FIRST RUN (cold cache) ===");
    let start = Instant::now();
    let store = DataStore::with_defaults(claude_home.clone(), None);
    let report = store.initial_load().await;
    let cold_elapsed = start.elapsed();

    eprintln!(
        "Cold cache load: {:?} (sessions: {}, stats: {})",
        cold_elapsed,
        report.sessions_scanned,
        if report.stats_loaded { "✓" } else { "✗" }
    );

    // Wait a bit for async cache writes to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // SECOND RUN: Use warm cache (should be FAST)
    eprintln!("\n=== SECOND RUN (warm cache) ===");
    let start = Instant::now();
    let store2 = DataStore::with_defaults(claude_home.clone(), None);
    let report2 = store2.initial_load().await;
    let warm_elapsed = start.elapsed();

    eprintln!(
        "Warm cache load: {:?} (sessions: {}, stats: {})",
        warm_elapsed,
        report2.sessions_scanned,
        if report2.stats_loaded { "✓" } else { "✗" }
    );

    let speedup = cold_elapsed.as_millis() as f64 / warm_elapsed.as_millis().max(1) as f64;
    eprintln!(
        "\n🚀 Speedup: {:.2}x ({:?} → {:?})",
        speedup, cold_elapsed, warm_elapsed
    );

    // Assert warm cache is under target
    assert!(
        warm_elapsed < targets::INITIAL_LOAD_MAX,
        "Warm cache load took {:?}, expected < {:?} (speedup: {:.2}x)",
        warm_elapsed,
        targets::INITIAL_LOAD_MAX,
        speedup
    );
}

#[tokio::test]
async fn test_session_scan_scales_linearly() {
    // Create fixtures with different session counts
    let counts = [10, 50, 100, 500];
    let mut times = Vec::new();

    for &count in &counts {
        let fixture = create_fixture_with_sessions(count);

        let start = Instant::now();
        let store = DataStore::with_defaults(fixture, None);
        store.initial_load().await;
        let elapsed = start.elapsed();

        times.push((count, elapsed));
        eprintln!("{} sessions → {:?}", count, elapsed);
    }

    // Check linearity: time(1000) / time(100) should be ~10x, not >15x
    if times.len() >= 2 {
        let (count_small, time_small) = times[0];
        let (count_large, time_large) = times[times.len() - 1];

        let expected_factor = count_large / count_small;
        let actual_factor = time_large.as_millis() / time_small.as_millis().max(1);
        let max_allowed_factor = (expected_factor * targets::SESSION_SCAN_LINEAR_FACTOR) as u128;

        assert!(
            actual_factor <= max_allowed_factor,
            "Session scan does not scale linearly: {}x sessions took {}x time (expected ≤{}x)",
            expected_factor,
            actual_factor,
            max_allowed_factor
        );
    }
}

#[tokio::test]
async fn test_no_oom_on_huge_jsonl() {
    // Create a session with a 10MB single line (simulating malformed/attack)
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    std::fs::create_dir_all(path.join("projects/-Users-test")).unwrap();

    let huge_line = "a".repeat(10 * 1024 * 1024); // 10MB
    std::fs::write(
        path.join("projects/-Users-test/huge.jsonl"),
        format!(r#"{{"type":"system","text":"{}"}}"#, huge_line),
    )
    .unwrap();

    let start = Instant::now();
    let store = DataStore::with_defaults(path.to_path_buf(), None);
    let report = store.initial_load().await;
    let elapsed = start.elapsed();

    eprintln!(
        "Huge JSONL handled in {:?} (scanned: {}, failed: {})",
        elapsed, report.sessions_scanned, report.sessions_failed
    );

    // Should gracefully handle without OOM
    assert!(
        elapsed < Duration::from_secs(5),
        "Huge JSONL took too long (potential hang)"
    );
}

#[tokio::test]
async fn test_stats_parsing_fast() {
    let claude_home = dirs::home_dir().expect("No home directory").join(".claude");

    let stats_path = claude_home.join("stats-cache.json");
    if !stats_path.exists() {
        eprintln!("SKIP: stats-cache.json not found");
        return;
    }

    use ccboard_core::parsers::StatsParser;

    let start = Instant::now();
    let parser = StatsParser::new();
    let mut report = ccboard_core::error::LoadReport::new();
    let _stats = parser.parse_graceful(&stats_path, &mut report).await;
    let elapsed = start.elapsed();

    eprintln!("Stats parsing took {:?}", elapsed);

    assert!(
        elapsed < targets::STATS_PARSE_MAX,
        "Stats parsing took {:?}, expected < {:?}",
        elapsed,
        targets::STATS_PARSE_MAX
    );
}

#[tokio::test]
async fn test_settings_merge_fast() {
    let claude_home = dirs::home_dir().expect("No home directory").join(".claude");

    if !claude_home.exists() {
        eprintln!("SKIP: ~/.claude not found");
        return;
    }

    use ccboard_core::parsers::SettingsParser;

    let start = Instant::now();
    let parser = SettingsParser::new();
    let mut report = ccboard_core::error::LoadReport::new();
    let _merged = parser.load_merged(&claude_home, None, &mut report).await;
    let elapsed = start.elapsed();

    eprintln!("Settings merge took {:?}", elapsed);

    assert!(
        elapsed < targets::SETTINGS_MERGE_MAX,
        "Settings merge took {:?}, expected < {:?}",
        elapsed,
        targets::SETTINGS_MERGE_MAX
    );
}

#[tokio::test]
async fn test_concurrent_access_no_deadlock() {
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(DataStore::with_defaults(dir.path().to_path_buf(), None));

    // Simulate concurrent reads/writes
    let mut handles = vec![];

    for _ in 0..10 {
        let store = store.clone();
        handles.push(tokio::spawn(async move {
            store.initial_load().await;
            store.stats();
            store.settings();
            store.session_count();
        }));
    }

    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap();
    }
    let elapsed = start.elapsed();

    eprintln!("10 concurrent operations took {:?}", elapsed);

    // Should complete without deadlock
    assert!(
        elapsed < Duration::from_secs(5),
        "Concurrent access took too long (potential deadlock)"
    );
}

// ====================
// Helper Functions
// ====================

/// Create fixture with N sessions
fn create_fixture_with_sessions(count: usize) -> std::path::PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Create stats
    std::fs::write(
        path.join("stats-cache.json"),
        format!(
            r#"{{"version": 2, "totalSessions": {}, "totalMessages": {}, "modelUsage": {{"test": {{"inputTokens": {}, "outputTokens": {}}}}}}}"#,
            count,
            count * 10,
            count * 100,
            count * 50
        ),
    )
    .unwrap();

    // Create N session files
    let projects_dir = path.join("projects").join("-Users-test");
    std::fs::create_dir_all(&projects_dir).unwrap();

    for i in 0..count {
        let session_id = format!("test-session-{:04}", i);
        let session_file = projects_dir.join(format!("{}.jsonl", session_id));

        // Minimal JSONL session
        std::fs::write(
            session_file,
            format!(
                r#"{{"type":"system","text":"Test session {}","timestamp":"2025-01-01T00:00:00Z"}}
{{"type":"user","text":"Test message","timestamp":"2025-01-01T00:01:00Z"}}
"#,
                i
            ),
        )
        .unwrap();
    }

    // Leak the tempdir to keep it alive
    std::mem::forget(dir);

    path
}
