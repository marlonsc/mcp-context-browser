//! Tests for circuit breaker module
//!
//! Tests for the circuit breaker pattern implementation using Actor pattern.

use mcp_context_browser::adapters::providers::routing::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, CircuitBreakerTrait,
};
use mcp_context_browser::domain::error::{Error, Result};
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_starts_closed() {
    let id = format!("test_start_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(id, config).await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_successful_operations(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let id = format!("test_success_{}", uuid::Uuid::new_v4());
    let config = CircuitBreakerConfig {
        persistence_enabled: false,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(id, config).await;
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
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        persistence_enabled: false,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(id, config).await;

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
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout: Duration::from_millis(500),
        success_threshold: 1,
        persistence_enabled: false,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(id, config).await;

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
