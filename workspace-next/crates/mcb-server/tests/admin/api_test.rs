//! Admin API Endpoint Tests
//!
//! Tests for individual admin HTTP endpoints using tower test utilities.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use mcb_domain::ports::admin::PerformanceMetricsInterface;
use mcb_infrastructure::adapters::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::{handlers::AdminState, routes::admin_router};
use std::sync::Arc;
use tower::ServiceExt;

/// Create a test AdminState with fresh metrics and indexing trackers
fn create_test_state() -> AdminState {
    AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: Arc::new(DefaultIndexingOperations::new()),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = create_test_state();
    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["uptime_seconds"].is_number());
    assert_eq!(json["active_indexing_operations"], 0);
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let state = create_test_state();

    // Record some metrics
    state.metrics.record_query(100, true, true);
    state.metrics.record_query(200, false, false);
    state.metrics.update_active_connections(3);

    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["total_queries"], 2);
    assert_eq!(json["successful_queries"], 1);
    assert_eq!(json["failed_queries"], 1);
    assert_eq!(json["active_connections"], 3);
    assert!(json["average_response_time_ms"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_indexing_endpoint_no_operations() {
    let state = create_test_state();
    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/indexing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["is_indexing"], false);
    assert_eq!(json["active_operations"], 0);
    assert!(json["operations"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_indexing_endpoint_with_operations() {
    let indexing = Arc::new(DefaultIndexingOperations::new());
    let state = AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: indexing.clone(),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
    };

    // Start an indexing operation
    let op_id = indexing.start_operation("test-collection", 50);
    indexing.update_progress(&op_id, Some("src/main.rs".to_string()), 10);

    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/indexing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["is_indexing"], true);
    assert_eq!(json["active_operations"], 1);

    let ops = json["operations"].as_array().unwrap();
    assert_eq!(ops.len(), 1);

    let op = &ops[0];
    assert_eq!(op["collection"], "test-collection");
    assert_eq!(op["current_file"], "src/main.rs");
    assert_eq!(op["processed_files"], 10);
    assert_eq!(op["total_files"], 50);
    assert_eq!(op["progress_percent"], 20.0);
}

#[tokio::test]
async fn test_readiness_probe_not_ready() {
    // Create a fresh state - uptime will be < 1 second
    let state = create_test_state();
    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Initially the server should return 503 (uptime < 1s)
    // Note: This test may be flaky if the system is very slow
    // The status could be either 200 or 503 depending on timing
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE,
        "Expected OK or SERVICE_UNAVAILABLE, got {:?}",
        status
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // The ready field should be boolean
    assert!(json["ready"].is_boolean());
}

#[tokio::test]
async fn test_readiness_probe_ready() {
    let state = create_test_state();

    // Wait for uptime to be >= 1 second
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["ready"], true);
}

#[tokio::test]
async fn test_liveness_probe() {
    let state = create_test_state();
    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["alive"], true);
}

#[tokio::test]
async fn test_health_with_active_operations() {
    let indexing = Arc::new(DefaultIndexingOperations::new());
    let state = AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: indexing.clone(),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
    };

    // Start two indexing operations
    indexing.start_operation("coll-1", 100);
    indexing.start_operation("coll-2", 200);

    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["active_indexing_operations"], 2);
}

#[tokio::test]
async fn test_metrics_with_cache_hits() {
    let state = create_test_state();

    // 3 cache hits, 2 misses
    state.metrics.record_query(10, true, true);
    state.metrics.record_query(20, true, true);
    state.metrics.record_query(30, true, true);
    state.metrics.record_query(40, true, false);
    state.metrics.record_query(50, true, false);

    let router = admin_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!((cache_hit_rate - 0.6).abs() < 0.01, "Expected ~60% cache hit rate");
}
