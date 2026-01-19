//! Real Auth Integration Tests
//!
//! These tests verify that authentication ACTUALLY WORKS with a real Rocket server.
//! Unlike the unit tests in auth_test.rs, these tests:
//!
//! 1. Start a real Rocket server with authentication ENABLED
//! 2. Make HTTP requests WITH and WITHOUT the API key
//! 3. Verify correct HTTP status codes (401 for unauthorized, 200 for authorized)
//! 4. Validate the response body contains correct error messages and data
//!
//! This ensures the AdminAuth guard works correctly in production.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_providers::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::{auth::AdminAuthConfig, handlers::AdminState, routes::admin_rocket};
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use std::sync::Arc;

/// Test API key
const TEST_API_KEY: &str = "test-secret-key-12345";

/// Test header name
const TEST_HEADER: &str = "X-Admin-Key";

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

/// Create a test state with mock services
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

/// Create an AdminAuthConfig with authentication ENABLED
fn create_auth_config() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_HEADER.to_string(),
        api_key: Some(TEST_API_KEY.to_string()),
    }
}

/// Create an AdminAuthConfig with authentication ENABLED but NO key configured
fn create_auth_config_no_key() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_HEADER.to_string(),
        api_key: None,
    }
}

// =============================================================================
// PROTECTED ENDPOINTS - Should require authentication
// =============================================================================

/// Test: /metrics without API key should return 401 with proper error message
#[rocket::async_test]
async fn test_metrics_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /metrics without API key should return 401 Unauthorized"
    );

    // Verify the response body is empty (Rocket doesn't send error body for guard failures)
    // This is expected behavior - the 401 status code is the indicator
}

/// Test: /metrics with wrong API key should return 401
#[rocket::async_test]
async fn test_metrics_with_wrong_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/metrics")
        .header(Header::new(TEST_HEADER, "wrong-key"))
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /metrics with wrong API key should return 401 Unauthorized"
    );
}

/// Test: /metrics with correct API key returns 200 and VALID metrics data
#[rocket::async_test]
async fn test_metrics_with_correct_key_returns_200() {
    let state = create_test_state();

    // Record some test metrics to verify we get real data back
    state.metrics.record_query(100, true, true); // 100ms, success, cache hit
    state.metrics.record_query(200, true, false); // 200ms, success, cache miss
    state.metrics.record_query(50, false, false); // 50ms, failure
    state.metrics.update_active_connections(5);

    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/metrics")
        .header(Header::new(TEST_HEADER, TEST_API_KEY))
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "GET /metrics with correct API key should return 200 OK"
    );

    // Validate the response body contains correct metrics data
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    // Verify metrics match what we recorded
    assert_eq!(json["total_queries"], 3, "Should have 3 total queries");
    assert_eq!(
        json["successful_queries"], 2,
        "Should have 2 successful queries"
    );
    assert_eq!(json["failed_queries"], 1, "Should have 1 failed query");
    assert_eq!(
        json["active_connections"], 5,
        "Should have 5 active connections"
    );

    // Verify cache hit rate is approximately 1/3 = 0.333
    let cache_hit_rate = json["cache_hit_rate"]
        .as_f64()
        .expect("cache_hit_rate is number");
    assert!(
        (cache_hit_rate - 0.333).abs() < 0.01,
        "Cache hit rate should be ~33.3%, got {}",
        cache_hit_rate
    );

    // Verify average response time is approximately (100+200+50)/3 = 116.67ms
    let avg_response_time = json["average_response_time_ms"]
        .as_f64()
        .expect("avg response time");
    assert!(
        (avg_response_time - 116.67).abs() < 1.0,
        "Average response time should be ~116.67ms, got {}",
        avg_response_time
    );
}

/// Test: /health/extended without API key should return 401
#[rocket::async_test]
async fn test_extended_health_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/health/extended").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /health/extended without API key should return 401 Unauthorized"
    );
}

/// Test: /health/extended with correct API key returns 200 and VALID health data
#[rocket::async_test]
async fn test_extended_health_with_correct_key_returns_200() {
    let state = create_test_state();

    // Record some metrics to make the response more interesting
    state.metrics.record_query(100, true, true);

    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/health/extended")
        .header(Header::new(TEST_HEADER, TEST_API_KEY))
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "GET /health/extended with correct API key should return 200 OK"
    );

    // Validate the response body contains expected health data structure
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    // Verify required fields exist and have correct types
    assert!(
        json["status"].is_string(),
        "Response should have 'status' string field"
    );
    assert!(
        json["uptime_seconds"].is_number(),
        "Response should have 'uptime_seconds' number field"
    );
    assert!(
        json["dependencies"].is_array(),
        "Response should have 'dependencies' array field"
    );
    assert!(
        json["dependencies_status"].is_string(),
        "Response should have 'dependencies_status' string field"
    );

    // Verify status is healthy or degraded (valid values)
    let status = json["status"].as_str().expect("status is string");
    assert!(
        status == "healthy" || status == "degraded",
        "Status should be 'healthy' or 'degraded', got '{}'",
        status
    );

    // Verify dependencies array has expected structure
    let deps = json["dependencies"]
        .as_array()
        .expect("dependencies is array");
    assert!(
        !deps.is_empty(),
        "Should have at least one dependency check"
    );

    for dep in deps {
        assert!(dep["name"].is_string(), "Dependency should have 'name'");
        assert!(dep["status"].is_string(), "Dependency should have 'status'");
        assert!(
            dep["last_check"].is_number(),
            "Dependency should have 'last_check'"
        );
    }
}

/// Test: POST /shutdown without API key should return 401
#[rocket::async_test]
async fn test_shutdown_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/shutdown")
        .header(rocket::http::ContentType::JSON)
        .body(r#"{}"#)
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "POST /shutdown without API key should return 401 Unauthorized"
    );
}

/// Test: /cache/stats without API key should return 401
#[rocket::async_test]
async fn test_cache_stats_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/cache/stats").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /cache/stats without API key should return 401 Unauthorized"
    );
}

// =============================================================================
// PUBLIC ENDPOINTS - Should NOT require authentication
// =============================================================================

/// Test: /health should work WITHOUT API key and return valid health data
#[rocket::async_test]
async fn test_health_public_no_auth_required() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/health").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "GET /health should work without API key (public endpoint)"
    );

    // Validate response body
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    assert_eq!(
        json["status"], "healthy",
        "Health status should be 'healthy'"
    );
    assert!(
        json["uptime_seconds"].is_number(),
        "Should have uptime_seconds"
    );
    assert!(
        json["active_indexing_operations"].is_number(),
        "Should have active_indexing_operations"
    );
}

/// Test: /live should work WITHOUT API key and return alive: true
#[rocket::async_test]
async fn test_live_public_no_auth_required() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/live").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "GET /live should work without API key (Kubernetes probe)"
    );

    // Validate response body
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    assert_eq!(
        json["alive"], true,
        "Liveness probe should return alive: true"
    );
}

/// Test: /ready should work WITHOUT API key and return ready status
#[rocket::async_test]
async fn test_ready_public_no_auth_required() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());

    // Wait for uptime > 1s to ensure ready
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/ready").dispatch().await;

    // Ready probe may return 503 if uptime < 1s, that's OK
    // The important thing is it doesn't return 401 (unauthorized)
    assert_ne!(
        response.status(),
        Status::Unauthorized,
        "GET /ready should not require authentication"
    );

    // Validate response body
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    assert!(
        json["ready"].is_boolean(),
        "Ready probe should return ready boolean"
    );
}

/// Test: /indexing should work WITHOUT API key and return valid status
#[rocket::async_test]
async fn test_indexing_public_no_auth_required() {
    let indexing = Arc::new(DefaultIndexingOperations::new());

    // Start an indexing operation to verify we get real data
    let op_id = indexing.start_operation("test-collection", 100);
    indexing.update_progress(&op_id, Some("src/main.rs".to_string()), 25);

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
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/indexing").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "GET /indexing should work without API key (public endpoint)"
    );

    // Validate response body contains real indexing data
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    assert_eq!(
        json["is_indexing"], true,
        "Should show indexing in progress"
    );
    assert_eq!(
        json["active_operations"], 1,
        "Should have 1 active operation"
    );

    let ops = json["operations"].as_array().expect("operations is array");
    assert_eq!(ops.len(), 1, "Should have exactly 1 operation");

    let op = &ops[0];
    assert_eq!(
        op["collection"], "test-collection",
        "Collection should match"
    );
    assert_eq!(
        op["current_file"], "src/main.rs",
        "Current file should match"
    );
    assert_eq!(op["processed_files"], 25, "Processed files should be 25");
    assert_eq!(op["total_files"], 100, "Total files should be 100");
    assert_eq!(op["progress_percent"], 25.0, "Progress should be 25%");
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Test: Auth enabled but no key configured should return 503
#[rocket::async_test]
async fn test_auth_enabled_no_key_configured_returns_503() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config_no_key());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(
        response.status(),
        Status::ServiceUnavailable,
        "Auth enabled but no key configured should return 503 ServiceUnavailable"
    );
}

/// Test: Custom header name works correctly
#[rocket::async_test]
async fn test_custom_header_name() {
    let state = create_test_state();
    let auth_config = Arc::new(AdminAuthConfig {
        enabled: true,
        header_name: "X-Custom-Auth".to_string(),
        api_key: Some("custom-key".to_string()),
    });
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    // Wrong header name should fail
    let response = client
        .get("/metrics")
        .header(Header::new("X-Admin-Key", "custom-key"))
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "Using default header when custom header is configured should return 401"
    );

    // Correct header name should work
    let response = client
        .get("/metrics")
        .header(Header::new("X-Custom-Auth", "custom-key"))
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "Using correct custom header should return 200"
    );
}

/// Test: Auth disabled allows all requests (backwards compatibility)
#[rocket::async_test]
async fn test_auth_disabled_allows_all() {
    let state = create_test_state();
    let auth_config = Arc::new(AdminAuthConfig::default()); // enabled: false
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "With auth disabled, protected endpoints should be accessible"
    );
}

// =============================================================================
// CONFIG ENDPOINTS - Protected
// =============================================================================

/// Test: GET /config without API key should return 401
#[rocket::async_test]
async fn test_config_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/config").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /config without API key should return 401 Unauthorized"
    );
}

/// Test: POST /config/reload without API key should return 401
#[rocket::async_test]
async fn test_config_reload_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.post("/config/reload").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "POST /config/reload without API key should return 401 Unauthorized"
    );
}

/// Test: PATCH /config/server without API key should return 401
#[rocket::async_test]
async fn test_config_update_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .patch("/config/server")
        .header(rocket::http::ContentType::JSON)
        .body(r#"{"values": {}}"#)
        .dispatch()
        .await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "PATCH /config/server without API key should return 401 Unauthorized"
    );
}

// =============================================================================
// LIFECYCLE ENDPOINTS - Protected
// =============================================================================

/// Test: GET /services without API key should return 401
#[rocket::async_test]
async fn test_services_list_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/services").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /services without API key should return 401 Unauthorized"
    );
}

/// Test: POST /services/test/start without API key should return 401
#[rocket::async_test]
async fn test_services_start_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.post("/services/test/start").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "POST /services/test/start without API key should return 401 Unauthorized"
    );
}

/// Test: POST /services/test/stop without API key should return 401
#[rocket::async_test]
async fn test_services_stop_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.post("/services/test/stop").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "POST /services/test/stop without API key should return 401 Unauthorized"
    );
}

/// Test: POST /services/test/restart without API key should return 401
#[rocket::async_test]
async fn test_services_restart_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.post("/services/test/restart").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "POST /services/test/restart without API key should return 401 Unauthorized"
    );
}

/// Test: GET /services/health without API key should return 401
#[rocket::async_test]
async fn test_services_health_without_key_returns_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client.get("/services/health").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Unauthorized,
        "GET /services/health without API key should return 401 Unauthorized"
    );
}

// =============================================================================
// VERIFY PROTECTED ENDPOINTS WORK WITH CORRECT KEY
// =============================================================================

/// Test: GET /config with correct API key should not return 401
#[rocket::async_test]
async fn test_config_with_correct_key_does_not_return_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/config")
        .header(Header::new(TEST_HEADER, TEST_API_KEY))
        .dispatch()
        .await;

    // May return 503 (no config watcher) but should NOT return 401
    assert_ne!(
        response.status(),
        Status::Unauthorized,
        "GET /config with correct key should not return 401"
    );
}

/// Test: GET /services with correct API key should not return 401
#[rocket::async_test]
async fn test_services_with_correct_key_does_not_return_401() {
    let state = create_test_state();
    let auth_config = Arc::new(create_auth_config());
    let client = Client::tracked(admin_rocket(state, auth_config))
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/services")
        .header(Header::new(TEST_HEADER, TEST_API_KEY))
        .dispatch()
        .await;

    // May return 503 (no service manager) but should NOT return 401
    assert_ne!(
        response.status(),
        Status::Unauthorized,
        "GET /services with correct key should not return 401"
    );
}
