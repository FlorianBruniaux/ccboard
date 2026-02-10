//! Performance benchmarks for analytics module
//!
//! Targets from PLAN_PHASE_H.md:
//! - compute_trends(1000 sessions, 30 days) → <100ms
//! - forecast_usage(30 days) → <20ms
//! - detect_patterns(1000 sessions) → <30ms
//! - generate_insights() → <10ms

use ccboard_core::analytics::{
    compute_trends, detect_patterns, forecast_usage, generate_insights, AnalyticsData, Period,
};
use ccboard_core::models::session::SessionMetadata;
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

/// Generate test sessions for benchmarking
fn generate_test_sessions(count: usize, days: usize) -> Vec<Arc<SessionMetadata>> {
    let now = Utc::now();
    (0..count)
        .map(|i| {
            let day_offset = (i % days) as i64;
            let ts = now - chrono::Duration::days(day_offset);

            Arc::new(SessionMetadata {
                id: format!("session-{}", i).into(),
                file_path: std::path::PathBuf::from(format!("/test/session-{}.jsonl", i)),
                project_path: "/test".into(),
                first_timestamp: Some(ts),
                last_timestamp: Some(ts + chrono::Duration::minutes(30)),
                message_count: 10,
                total_tokens: 1000 + (i as u64 * 100),
                input_tokens: 500 + (i as u64 * 50),
                output_tokens: 500 + (i as u64 * 50),
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                models_used: vec!["sonnet".to_string()],
                file_size_bytes: 1024 * (i as u64 + 1),
                first_user_message: None,
                has_subagents: false,
                tool_usage: std::collections::HashMap::new(),
                duration_seconds: Some(1800),
                branch: None,
            })
        })
        .collect()
}

/// Benchmark 1: compute_trends with varying session counts
fn trends_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute_trends");

    for count in [10, 100, 1000] {
        let sessions = generate_test_sessions(count, 30);
        group.bench_with_input(
            BenchmarkId::new("sessions", count),
            &sessions,
            |b, sessions| {
                b.iter(|| {
                    black_box(compute_trends(sessions, 30));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 2: forecast_usage with varying day counts
fn forecast_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("forecast_usage");

    for days in [7, 30, 90] {
        let sessions = generate_test_sessions(days * 3, days);
        let trends = compute_trends(&sessions, days);
        group.bench_with_input(BenchmarkId::new("days", days), &trends, |b, trends| {
            b.iter(|| {
                black_box(forecast_usage(trends));
            });
        });
    }

    group.finish();
}

/// Benchmark 3: detect_patterns with varying session counts
fn patterns_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("detect_patterns");

    for count in [10, 100, 1000] {
        let sessions = generate_test_sessions(count, 30);
        group.bench_with_input(
            BenchmarkId::new("sessions", count),
            &sessions,
            |b, sessions| {
                b.iter(|| {
                    black_box(detect_patterns(sessions, 30));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 4: generate_insights (full pipeline)
fn insights_benchmark(c: &mut Criterion) {
    let sessions = generate_test_sessions(1000, 30);
    let trends = compute_trends(&sessions, 30);
    let patterns = detect_patterns(&sessions, 30);
    let forecast = forecast_usage(&trends);

    c.bench_function("generate_insights", |b| {
        b.iter(|| {
            black_box(generate_insights(&trends, &patterns, &forecast));
        });
    });
}

/// Benchmark 5: Full analytics pipeline (AnalyticsData::compute)
fn full_pipeline_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    for count in [100, 1000] {
        let sessions = generate_test_sessions(count, 30);
        let period = Period::last_30d();
        group.bench_with_input(
            BenchmarkId::new("sessions", count),
            &sessions,
            |b, sessions| {
                b.iter(|| {
                    black_box(AnalyticsData::compute(sessions, period));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    trends_benchmark,
    forecast_benchmark,
    patterns_benchmark,
    insights_benchmark,
    full_pipeline_benchmark
);
criterion_main!(benches);
