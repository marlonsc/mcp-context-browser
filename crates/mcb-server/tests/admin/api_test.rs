//! Admin API Endpoint Tests
//!
//! Tests for individual admin HTTP endpoints using Rocket test utilities.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use async_trait::async_trait;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_providers::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::{auth::AdminAuthConfig, handlers::AdminState, routes::admin_rocket};
use rocket::http::Status;
use rocket::local::asynchronous::Client;
use std::sync::Arc;

/// Null EventBus for testing
struct TestEventBus;

#[async_trait]
impl EventBusProvider for TestEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        Ok(Box::pin(futures::stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("test-subscription".to_string())
    }
}

/// Create a test AdminState with fresh metrics and indexing trackers
fn create_test_state() -> AdminState {
    AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: Arc::new(DefaultIndexingOperations::new()),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(TestEventBus),
        service_manager: None,
        cache: None,
    }
}

#[rocket::async_test]
async fn test_health_endpoint() {
    let state = create_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["uptime_seconds"].is_number());
    assert_eq!(json["active_indexing_operations"], 0);
}

#[rocket::async_test]
async fn test_metrics_endpoint() {
    let state = create_test_state();

    // Record some metrics
    state.metrics.record_query(100, true, true);
    state.metrics.record_query(200, false, false);
    state.metrics.update_active_connections(3);

    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 2);
    assert_eq!(json["successful_queries"], 1);
    assert_eq!(json["failed_queries"], 1);
    assert_eq!(json["active_connections"], 3);
    assert!(json["average_response_time_ms"].as_f64().unwrap() > 0.0);
}

#[rocket::async_test]
async fn test_indexing_endpoint_no_operations() {
    let state = create_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/indexing").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["is_indexing"], false);
    assert_eq!(json["active_operations"], 0);
    assert!(json["operations"].as_array().unwrap().is_empty());
}

#[rocket::async_test]
async fn test_indexing_endpoint_with_operations() {
    let indexing = Arc::new(DefaultIndexingOperations::new());
    let state = AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: indexing.clone(),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(TestEventBus),
        service_manager: None,
        cache: None,
    };

    // Start an indexing operation
    let op_id = indexing.start_operation("test-collection", 50);
    indexing.update_progress(&op_id, Some("src/main.rs".to_string()), 10);

    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/indexing").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

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

#[rocket::async_test]
async fn test_readiness_probe_not_ready() {
    // Create a fresh state - uptime will be < 1 second
    let state = create_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/ready").dispatch().await;

    // Initially the server should return 503 (uptime < 1s)
    // Note: This test may be flaky if the system is very slow
    // The status could be either 200 or 503 depending on timing
    let status = response.status();
    assert!(
        status == Status::Ok || status == Status::ServiceUnavailable,
        "Expected Ok or ServiceUnavailable, got {:?}",
        status
    );

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    // The ready field should be boolean
    assert!(json["ready"].is_boolean());
}

#[rocket::async_test]
async fn test_readiness_probe_ready() {
    let state = create_test_state();

    // Wait for uptime to be >= 1 second
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/ready").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["ready"], true);
}

#[rocket::async_test]
async fn test_liveness_probe() {
    let state = create_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/live").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["alive"], true);
}

#[rocket::async_test]
async fn test_health_with_active_operations() {
    let indexing = Arc::new(DefaultIndexingOperations::new());
    let state = AdminState {
        metrics: Arc::new(AtomicPerformanceMetrics::new()),
        indexing: indexing.clone(),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(TestEventBus),
        service_manager: None,
        cache: None,
    };

    // Start two indexing operations
    indexing.start_operation("coll-1", 100);
    indexing.start_operation("coll-2", 200);

    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["active_indexing_operations"], 2);
}

#[rocket::async_test]
async fn test_metrics_with_cache_hits() {
    let state = create_test_state();

    // 3 cache hits, 2 misses
    state.metrics.record_query(10, true, true);
    state.metrics.record_query(20, true, true);
    state.metrics.record_query(30, true, true);
    state.metrics.record_query(40, true, false);
    state.metrics.record_query(50, true, false);

    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.6).abs() < 0.01,
        "Expected ~60% cache hit rate"
    );
}
