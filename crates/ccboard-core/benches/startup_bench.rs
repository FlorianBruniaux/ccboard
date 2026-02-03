//! Startup performance benchmarks for ccboard-core
//!
//! Measures baseline performance of:
//! - Full initial_load() workflow
//! - Individual parsing operations
//! - Session scanning
//! - Stats/settings loading
//!
//! Run with:
//! ```bash
//! cargo bench --bench startup_bench
//! ```

use ccboard_core::DataStore;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::path::PathBuf;
use std::time::Duration;

/// Benchmark full initial load with real ~/.claude data
fn bench_initial_load_real(c: &mut Criterion) {
    let mut group = c.benchmark_group("initial_load_real");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let claude_home = dirs::home_dir().expect("No home directory").join(".claude");

    if !claude_home.exists() {
        eprintln!("SKIP: ~/.claude not found at {}", claude_home.display());
        return;
    }

    group.bench_function("full_workflow", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let store = DataStore::with_defaults(claude_home.clone(), None);
            store.initial_load().await
        });
    });

    group.finish();
}

/// Benchmark initial load with fixture data (controlled)
fn bench_initial_load_fixture(c: &mut Criterion) {
    let mut group = c.benchmark_group("initial_load_fixture");
    group.measurement_time(Duration::from_secs(5));

    // Create fixture directory
    let fixture = create_fixture();

    group.bench_function("controlled_data", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let store = DataStore::with_defaults(fixture.clone(), None);
            store.initial_load().await
        });
    });

    group.finish();
}

/// Benchmark session scanning scalability
fn bench_session_scan_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_scan_scaling");

    for session_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(session_count),
            session_count,
            |b, &count| {
                let fixture = create_fixture_with_sessions(count);
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&rt).iter(|| async {
                    let store = DataStore::with_defaults(fixture.clone(), None);
                    store.initial_load().await
                });
            },
        );
    }

    group.finish();
}

/// Benchmark individual parsing operations
fn bench_parsing_operations(c: &mut Criterion) {
    use ccboard_core::parsers::{SessionIndexParser, StatsParser};

    let mut group = c.benchmark_group("parsing");

    let claude_home = dirs::home_dir().expect("No home directory").join(".claude");

    if !claude_home.exists() {
        eprintln!("SKIP: ~/.claude not found");
        return;
    }

    // Bench stats parsing
    let stats_path = claude_home.join("stats-cache.json");
    if stats_path.exists() {
        group.bench_function("stats_parse", |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(&rt).iter(|| async {
                let parser = StatsParser::new();
                let mut report = ccboard_core::error::LoadReport::new();
                parser.parse_graceful(&stats_path, &mut report).await
            });
        });
    }

    // Bench session scanning
    let projects_dir = claude_home.join("projects");
    if projects_dir.exists() {
        group.bench_function("session_scan_all", |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(&rt).iter(|| async {
                let parser = SessionIndexParser::new();
                let mut report = ccboard_core::error::LoadReport::new();
                parser.scan_all(&projects_dir, &mut report).await
            });
        });
    }

    group.finish();
}

/// Create minimal fixture directory
fn create_fixture() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Create minimal stats-cache.json
    std::fs::write(
        path.join("stats-cache.json"),
        r#"{"version": 2, "totalSessions": 1, "totalMessages": 10, "modelUsage": {"test": {"inputTokens": 100, "outputTokens": 50}}}"#,
    ).unwrap();

    // Create empty projects directory
    std::fs::create_dir_all(path.join("projects")).unwrap();

    // Leak the tempdir to keep it alive for benchmarks
    std::mem::forget(dir);

    path
}

/// Create fixture with N sessions
fn create_fixture_with_sessions(count: usize) -> PathBuf {
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
    ).unwrap();

    // Create N session files
    let projects_dir = path.join("projects").join("-Users-test");
    std::fs::create_dir_all(&projects_dir).unwrap();

    for i in 0..count {
        let session_id = format!("test-session-{:04}", i);
        let session_file = projects_dir.join(format!("{}.jsonl", session_id));

        // Minimal JSONL session with first and last message
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

    // Leak the tempdir
    std::mem::forget(dir);

    path
}

criterion_group!(
    benches,
    bench_initial_load_real,
    bench_initial_load_fixture,
    bench_session_scan_scaling,
    bench_parsing_operations
);
criterion_main!(benches);
