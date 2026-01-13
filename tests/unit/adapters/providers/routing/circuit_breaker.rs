//! Tests for circuit breaker module
//!
//! Tests for the circuit breaker pattern implementation using Actor pattern.

use mcp_context_browser::adapters::providers::routing::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, CircuitBreakerTrait,
};
use mcp_context_browser::domain::error::{Error, Result};
use std::time::Duration;

/// Helper to create a circuit breaker for testing (uses temp directory)
async fn create_test_circuit_breaker(id: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
    let temp_dir = std::env::temp_dir().join("test_circuit_breakers").join(id);
    CircuitBreaker::with_config_and_path(id, config, temp_dir).await
}

/// Helper to create a test config with persistence disabled
fn test_config_no_persist() -> CircuitBreakerConfig {
    CircuitBreakerConfig::new(5, Duration::from_secs(60), 3, 10, false)
}

#[tokio::test]
async fn test_circuit_breaker_starts_closed() {
    let id = format!("test_start_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(5, Duration::from_secs(60), 3, 10, false);
    let cb = create_test_circuit_breaker(&id, config).await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_successful_operations(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let id = format!("test_success_{}", uuid::Uuid::new_v4());
    let config = test_config_no_persist();
    let cb = create_test_circuit_breaker(&id, config).await;
    let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
    let value = result?;
    assert_eq!(value, 42);
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    assert_eq!(cb.metrics().await.successful_requests, 1);
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_failure_threshold() {
    let id = format!("test_failure_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        2,                       // failure_threshold
        Duration::from_secs(60), // recovery_timeout
        3,                       // success_threshold
        10,                      // half_open_max_requests
        false,                   // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // First failure
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

    // Give the actor time to process
    tokio::time::sleep(Duration::from_millis(10)).await;

    assert_eq!(cb.state().await, CircuitBreakerState::Closed);

    // Second failure - should open
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

    // Give the actor time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_reset() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let id = format!("test_reset_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        1,                          // failure_threshold
        Duration::from_millis(500), // recovery_timeout
        1,                          // success_threshold
        10,                         // half_open_max_requests
        false,                      // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Open circuit
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

    // Give the actor time to process the failure message (less than recovery timeout)
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

    // Wait for recovery timeout (more than 500ms)
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Should transition to half-open and then close on success
    let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
    assert!(result.is_ok(), "Call should succeed in half-open state");
    let value = result?;
    assert_eq!(value, 42);

    // Give the actor time to process the success message
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    Ok(())
}

// ===== State Transition Tests =====

#[tokio::test]
async fn test_state_transition_half_open_to_open_on_failure() {
    let id = format!("test_halfopen_fail_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        1,                          // failure_threshold
        Duration::from_millis(100), // recovery_timeout
        3,                          // success_threshold - Need 3 successes to close
        10,                         // half_open_max_requests
        false,                      // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Trip circuit to Open
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

    // Wait for recovery timeout to transition to HalfOpen
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // Failure in HalfOpen should go back to Open
    let _: Result<()> = cb
        .call(|| async { Err(Error::generic("fail in half-open")) })
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

    // Verify circuit_opened_count incremented twice
    let metrics = cb.metrics().await;
    assert_eq!(metrics.circuit_opened_count, 2);
}

#[tokio::test]
async fn test_state_transition_open_blocks_calls() {
    let id = format!("test_open_blocks_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        1,                       // failure_threshold
        Duration::from_secs(60), // Long timeout so it stays Open
        3,                       // success_threshold
        10,                      // half_open_max_requests
        false,                   // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Trip circuit to Open
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

    // Attempt calls - they should be rejected
    for _ in 0..5 {
        let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
        assert!(result.is_err());
    }

    // Verify rejected_requests counter
    tokio::time::sleep(Duration::from_millis(50)).await;
    let metrics = cb.metrics().await;
    assert_eq!(metrics.rejected_requests, 5);
}

#[tokio::test]
async fn test_state_transition_half_open_limits_requests() {
    let id = format!("test_halfopen_limit_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        1,                          // failure_threshold
        Duration::from_millis(100), // recovery_timeout
        10,                         // High threshold so circuit stays HalfOpen
        3,                          // Only allow 3 requests in HalfOpen
        false,                      // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Trip circuit to Open
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Wait for transition to HalfOpen
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // First 3 calls should succeed (within limit)
    for i in 0..3 {
        let result: Result<i32> = cb.call(|| async { Ok(i) }).await;
        assert!(result.is_ok(), "Call {} should be allowed in HalfOpen", i);
    }

    // Subsequent calls should be rejected
    let result: Result<i32> = cb.call(|| async { Ok(999) }).await;
    assert!(result.is_err(), "Call beyond limit should be rejected");

    // Verify rejected_requests includes the blocked call
    tokio::time::sleep(Duration::from_millis(50)).await;
    let metrics = cb.metrics().await;
    assert!(metrics.rejected_requests >= 1);
}

#[tokio::test]
async fn test_state_transition_multiple_successes_to_close() {
    let id = format!("test_multi_success_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        1,                          // failure_threshold
        Duration::from_millis(100), // recovery_timeout
        3,                          // Need 3 successes to close
        10,                         // half_open_max_requests
        false,                      // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Trip circuit to Open
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Wait for transition to HalfOpen
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // First success - still HalfOpen
    let _: Result<i32> = cb.call(|| async { Ok(1) }).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // Second success - still HalfOpen
    let _: Result<i32> = cb.call(|| async { Ok(2) }).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // Third success - should transition to Closed
    let _: Result<i32> = cb.call(|| async { Ok(3) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);

    // Verify circuit_closed_count
    let metrics = cb.metrics().await;
    assert_eq!(metrics.circuit_closed_count, 1);
}

#[tokio::test]
async fn test_consecutive_failures_reset_on_success() {
    let id = format!("test_reset_failures_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        3,                       // failure_threshold
        Duration::from_secs(60), // recovery_timeout
        3,                       // success_threshold
        10,                      // half_open_max_requests
        false,                   // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Two failures - just below threshold
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail 1")) }).await;
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail 2")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.consecutive_failures, 2);
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);

    // Success resets the counter
    let _: Result<i32> = cb.call(|| async { Ok(42) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.consecutive_failures, 0);
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);

    // Can now have 2 more failures without tripping
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail 3")) }).await;
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail 4")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    assert_eq!(cb.metrics().await.consecutive_failures, 2);
}

#[tokio::test]
async fn test_metrics_tracking_complete() {
    let id = format!("test_metrics_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig::new(
        2,                          // failure_threshold
        Duration::from_millis(100), // recovery_timeout
        1,                          // success_threshold
        10,                         // half_open_max_requests
        false,                      // persistence_enabled
    );
    let cb = create_test_circuit_breaker(&id, config).await;

    // Initial metrics
    let metrics = cb.metrics().await;
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.circuit_opened_count, 0);

    // 3 successes
    for _ in 0..3 {
        let _: Result<i32> = cb.call(|| async { Ok(42) }).await;
    }
    tokio::time::sleep(Duration::from_millis(20)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.total_requests, 3);
    assert_eq!(metrics.successful_requests, 3);
    assert_eq!(metrics.failed_requests, 0);

    // 2 failures to trip circuit
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.total_requests, 5);
    assert_eq!(metrics.successful_requests, 3);
    assert_eq!(metrics.failed_requests, 2);
    assert_eq!(metrics.circuit_opened_count, 1);

    // 2 rejected calls in Open state
    let _: Result<i32> = cb.call(|| async { Ok(1) }).await;
    let _: Result<i32> = cb.call(|| async { Ok(2) }).await;
    tokio::time::sleep(Duration::from_millis(20)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.rejected_requests, 2);
    // total_requests doesn't include rejected
    assert_eq!(metrics.total_requests, 5);

    // Wait for HalfOpen, then success to close
    tokio::time::sleep(Duration::from_millis(150)).await;
    let _: Result<i32> = cb.call(|| async { Ok(42) }).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let metrics = cb.metrics().await;
    assert_eq!(metrics.circuit_closed_count, 1);
    assert_eq!(metrics.total_requests, 6);
    assert_eq!(metrics.successful_requests, 4);
}

#[tokio::test]
async fn test_state_display_formatting() {
    // Test Display trait implementation for CircuitBreakerState
    let closed = CircuitBreakerState::Closed;
    assert_eq!(format!("{}", closed), "closed");

    let open = CircuitBreakerState::Open {
        opened_at: std::time::Instant::now(),
    };
    assert_eq!(format!("{}", open), "open");

    let half_open = CircuitBreakerState::HalfOpen;
    assert_eq!(format!("{}", half_open), "half-open");
}

#[tokio::test]
async fn test_production_config_values() {
    let config = CircuitBreakerConfig::production();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.recovery_timeout, Duration::from_secs(60));
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.half_open_max_requests, 10);
    assert!(config.persistence_enabled);
}

#[tokio::test]
async fn test_circuit_breaker_with_custom_id() {
    let custom_id = "my-custom-service-breaker";
    let config = CircuitBreakerConfig::new(
        5,                       // failure_threshold
        Duration::from_secs(60), // recovery_timeout
        3,                       // success_threshold
        10,                      // half_open_max_requests
        false,                   // persistence_enabled
    );
    let cb = create_test_circuit_breaker(custom_id, config).await;

    // Verify it works with custom ID
    let result: Result<i32> = cb.call(|| async { Ok(123) }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
}
