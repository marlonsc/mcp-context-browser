//! Admin API Full Stack Integration Test
//!
//! Complete end-to-end test that starts the admin server, hits all endpoints,
//! and verifies the full stack works together.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use async_trait::async_trait;
use mcb_application::ports::admin::PerformanceMetricsInterface;
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

/// Helper to create test admin state with shared references
fn create_shared_test_state() -> (
    AdminState,
    Arc<AtomicPerformanceMetrics>,
    Arc<DefaultIndexingOperations>,
) {
    let metrics = Arc::new(AtomicPerformanceMetrics::new());
    let indexing = Arc::new(DefaultIndexingOperations::new());
    let state = AdminState {
        metrics: metrics.clone(),
        indexing: indexing.clone(),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(TestEventBus),
        service_manager: None,
        cache: None,
    };
    (state, metrics, indexing)
}

/// Full integration test exercising all admin endpoints and functionality
#[rocket::async_test]
async fn test_full_admin_stack_integration() {
    // 1. Create shared state for metrics and indexing
    let (state, metrics, indexing) = create_shared_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    // 2. Verify initial health status
    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["active_indexing_operations"], 0);

    // 3. Record some metrics
    metrics.record_query(100, true, true); // successful, cache hit
    metrics.record_query(150, true, false); // successful, cache miss
    metrics.record_query(200, false, false); // failed
    metrics.update_active_connections(5);

    // 4. Verify metrics endpoint reflects recorded data
    let response = client.get("/metrics").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 3);
    assert_eq!(json["successful_queries"], 2);
    assert_eq!(json["failed_queries"], 1);
    assert_eq!(json["active_connections"], 5);
    // Cache hit rate: 1/3 = 0.333...
    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.333).abs() < 0.01,
        "Expected ~33% cache hit rate, got {}",
        cache_hit_rate
    );

    // 5. Start indexing operations and verify
    let op1 = indexing.start_operation("project-alpha", 100);
    let _op2 = indexing.start_operation("project-beta", 200);
    indexing.update_progress(&op1, Some("src/main.rs".to_string()), 25);

    // 6. Verify indexing endpoint shows operations
    let response = client.get("/indexing").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["is_indexing"], true);
    assert_eq!(json["active_operations"], 2);

    let ops = json["operations"].as_array().unwrap();
    assert_eq!(ops.len(), 2);

    // Find the project-alpha operation
    let alpha_op = ops
        .iter()
        .find(|op| op["collection"] == "project-alpha")
        .expect("Should find project-alpha operation");
    assert_eq!(alpha_op["processed_files"], 25);
    assert_eq!(alpha_op["total_files"], 100);
    assert_eq!(alpha_op["progress_percent"], 25.0);
    assert_eq!(alpha_op["current_file"], "src/main.rs");

    // 7. Health should now show active indexing operations
    let response = client.get("/health").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_indexing_operations"], 2);

    // 8. Complete one operation
    indexing.complete_operation(&op1);

    let response = client.get("/indexing").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_operations"], 1);

    // 9. Verify liveness probe (always OK)
    let response = client.get("/live").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["alive"], true);

    // 10. Wait for readiness (needs uptime > 1s)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let response = client.get("/ready").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["ready"], true);
}

/// Test that metrics accumulate correctly across multiple recording batches
#[rocket::async_test]
async fn test_metrics_accumulation_integration() {
    let (state, metrics, _) = create_shared_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    // First batch of metrics
    for _ in 0..10 {
        metrics.record_query(50, true, true);
    }

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["total_queries"], 10);
    assert_eq!(json["cache_hit_rate"], 1.0);

    // Second batch with some failures and cache misses
    for _ in 0..5 {
        metrics.record_query(100, false, false);
    }

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 15);
    assert_eq!(json["successful_queries"], 10);
    assert_eq!(json["failed_queries"], 5);
    // Cache hits: 10/15 = 0.666...
    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.666).abs() < 0.01,
        "Expected ~66.6% cache hit rate"
    );
}

/// Test concurrent indexing operations lifecycle
#[rocket::async_test]
async fn test_indexing_lifecycle_integration() {
    let (state, _, indexing) = create_shared_test_state();
    let client = Client::tracked(admin_rocket(
        state,
        Arc::new(AdminAuthConfig::default()),
        None,
    ))
    .await
    .expect("valid rocket instance");

    // Start 3 concurrent operations
    let op1 = indexing.start_operation("repo-1", 50);
    let op2 = indexing.start_operation("repo-2", 100);
    let op3 = indexing.start_operation("repo-3", 150);

    // Verify all 3 are tracked
    let response = client.get("/indexing").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_operations"], 3);

    // Progress updates
    indexing.update_progress(&op1, Some("file1.rs".to_string()), 50); // 100% complete
    indexing.update_progress(&op2, Some("file2.rs".to_string()), 50); // 50% complete
    indexing.update_progress(&op3, Some("file3.rs".to_string()), 75); // 50% complete

    // Complete op1
    indexing.complete_operation(&op1);

    let response = client.get("/indexing").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_operations"], 2);

    // Complete remaining
    indexing.complete_operation(&op2);
    indexing.complete_operation(&op3);

    let response = client.get("/indexing").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_operations"], 0);
    assert_eq!(json["is_indexing"], false);
}
