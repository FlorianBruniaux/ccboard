//! Integration tests for metadata cache with real sessions

use ccboard_core::cache::MetadataCache;
use ccboard_core::parsers::SessionIndexParser;
use ccboard_core::DataStore;
use std::sync::Arc;

#[tokio::test]
async fn test_cache_write_real_file() {
    let home = dirs::home_dir().unwrap().join(".claude");
    if !home.exists() {
        eprintln!("SKIP: ~/.claude not found");
        return;
    }

    let cache_dir = home.join("cache-test");
    let _ = std::fs::remove_dir_all(&cache_dir); // Clean start

    // Create cache
    let cache = MetadataCache::new(&cache_dir).unwrap();

    // Find first real session file
    let projects_dir = home.join("projects");
    let session_path = walkdir::WalkDir::new(&projects_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf());

    let Some(session_path) = session_path else {
        eprintln!("SKIP: no session files found");
        return;
    };

    eprintln!("Testing with session: {}", session_path.display());

    // Parse and cache it
    let parser = SessionIndexParser::new().with_cache(Arc::new(cache));
    let meta = parser.scan_session(&session_path).await.unwrap();

    eprintln!("Parsed session: id={}, tokens={}", meta.id, meta.total_tokens);

    // Get cache reference
    let cache_path = cache_dir.join("session-metadata.db");

    // Wait a bit for async writes
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify in DB
    let output = std::process::Command::new("sqlite3")
        .arg(&cache_path)
        .arg("SELECT COUNT(*) FROM session_metadata;")
        .output()
        .expect("sqlite3 not installed");

    let count: usize = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    eprintln!("Cache entries after write: {}", count);
    assert!(count > 0, "Cache should have at least 1 entry after write!");

    // Cleanup
    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn test_datastore_uses_cache() {
    let home = dirs::home_dir().unwrap().join(".claude");
    if !home.exists() {
        eprintln!("SKIP: ~/.claude not found");
        return;
    }

    // Clear cache
    let cache_path = home.join("cache/session-metadata.db");
    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(cache_path.with_extension("db-shm"));
    let _ = std::fs::remove_file(cache_path.with_extension("db-wal"));

    eprintln!("Cache cleared");

    // Load
    let start = std::time::Instant::now();
    let store = DataStore::with_defaults(home.clone(), None);
    let report = store.initial_load().await;
    let elapsed = start.elapsed();

    eprintln!(
        "Initial load took {:?} (sessions: {})",
        elapsed, report.sessions_scanned
    );

    // Wait for async writes to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check cache was populated
    if cache_path.exists() {
        let output = std::process::Command::new("sqlite3")
            .arg(&cache_path)
            .arg("SELECT COUNT(*) FROM session_metadata;")
            .output()
            .expect("sqlite3 not installed");

        let count: usize = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);

        eprintln!("Cache entries after DataStore load: {}", count);

        if count == 0 {
            eprintln!("⚠️  WARNING: Cache exists but empty - writes are not happening!");
        } else {
            eprintln!("✅ Cache populated with {} entries", count);
        }

        // Now test cache hit on second load
        let start2 = std::time::Instant::now();
        let store2 = DataStore::with_defaults(home.clone(), None);
        let report2 = store2.initial_load().await;
        let elapsed2 = start2.elapsed();

        eprintln!(
            "Second load took {:?} (sessions: {})",
            elapsed2, report2.sessions_scanned
        );

        if elapsed2 < elapsed / 2 {
            eprintln!("✅ Cache speedup observed: {:?} → {:?}", elapsed, elapsed2);
        } else {
            eprintln!("⚠️  WARNING: No speedup - cache not being used for reads!");
        }
    } else {
        eprintln!("❌ ERROR: Cache file not created!");
        panic!("Cache should be created by DataStore");
    }
}

#[tokio::test]
async fn test_cache_hit_speedup() {
    let home = dirs::home_dir().unwrap().join(".claude");
    if !home.exists() {
        eprintln!("SKIP: ~/.claude not found");
        return;
    }

    let cache_dir = home.join("cache-speedtest");
    let _ = std::fs::remove_dir_all(&cache_dir);

    let cache = Arc::new(MetadataCache::new(&cache_dir).unwrap());

    // Find 10 real session files
    let projects_dir = home.join("projects");
    let sessions: Vec<_> = walkdir::WalkDir::new(&projects_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .take(10)
        .map(|e| e.path().to_path_buf())
        .collect();

    if sessions.is_empty() {
        eprintln!("SKIP: no sessions found");
        return;
    }

    eprintln!("Testing with {} sessions", sessions.len());

    // First pass: parse without cache
    let parser_nocache = SessionIndexParser::new();
    let start = std::time::Instant::now();
    for path in &sessions {
        let _ = parser_nocache.scan_session(path).await;
    }
    let uncached_time = start.elapsed();

    eprintln!("Uncached parse: {:?}", uncached_time);

    // Second pass: parse WITH cache (cold)
    let parser_cache = SessionIndexParser::new().with_cache(cache.clone());
    let start = std::time::Instant::now();
    for path in &sessions {
        let _ = parser_cache.scan_session(path).await;
    }
    let cached_cold_time = start.elapsed();

    eprintln!("Cached (cold) parse: {:?}", cached_cold_time);

    // Wait for writes
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Third pass: parse WITH cache (warm - should be FAST)
    let start = std::time::Instant::now();
    for path in &sessions {
        let _ = parser_cache.scan_session(path).await;
    }
    let cached_warm_time = start.elapsed();

    eprintln!("Cached (warm) parse: {:?}", cached_warm_time);

    let speedup = uncached_time.as_millis() as f64 / cached_warm_time.as_millis().max(1) as f64;
    eprintln!("Speedup: {:.2}x", speedup);

    if speedup > 5.0 {
        eprintln!("✅ Cache provides significant speedup");
    } else {
        eprintln!("⚠️  WARNING: Cache speedup is only {:.2}x (expected >5x)", speedup);
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&cache_dir);
}
