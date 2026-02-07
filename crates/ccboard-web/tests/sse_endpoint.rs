//! Integration test for SSE endpoint

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ccboard_core::DataStore;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_sse_endpoint_exists() {
    // Create store with temp dir
    let temp_dir = std::env::temp_dir().join("ccboard-test-sse");
    std::fs::create_dir_all(&temp_dir).ok();

    let store = Arc::new(DataStore::with_defaults(temp_dir.clone(), None));

    // Create router
    let router = ccboard_web::create_router(store);

    // Test /api/events endpoint
    let request = Request::builder()
        .uri("/api/events")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should return 200 OK with text/event-stream header
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());

    assert!(content_type.is_some());
    assert!(content_type.unwrap().contains("text/event-stream"));

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
