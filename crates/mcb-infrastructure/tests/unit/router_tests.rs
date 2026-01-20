//! Unit tests for provider routing and health monitoring
//!
//! Tests the InMemoryHealthMonitor state transitions and DefaultProviderRouter
//! provider selection logic based on health status.

use mcb_application::ports::infrastructure::routing::{
    ProviderContext, ProviderHealthStatus, ProviderRouter,
};
use mcb_infrastructure::routing::{
    DefaultProviderRouter, HealthMonitor, InMemoryHealthMonitor, NullHealthMonitor,
};
use std::sync::Arc;

// =============================================================================
// InMemoryHealthMonitor Tests
// =============================================================================

/// Test that new providers start as healthy
#[test]
fn test_new_provider_starts_healthy() {
    let monitor = InMemoryHealthMonitor::new();

    // Unknown provider should be healthy
    let status = monitor.get_health("unknown-provider");
    assert_eq!(status, ProviderHealthStatus::Healthy);
}

/// Test health transitions from Healthy → Degraded after failures
#[test]
fn test_health_transitions_to_degraded() {
    // Default thresholds: degraded=2, unhealthy=5
    let monitor = InMemoryHealthMonitor::new();

    // Record 2 failures to trigger degraded
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a");

    let status = monitor.get_health("provider-a");
    assert_eq!(status, ProviderHealthStatus::Degraded);
}

/// Test health transitions from Degraded → Unhealthy after more failures
#[test]
fn test_health_transitions_to_unhealthy() {
    // Default thresholds: degraded=2, unhealthy=5
    let monitor = InMemoryHealthMonitor::new();

    // Record 5 failures to trigger unhealthy
    for _ in 0..5 {
        monitor.record_failure("provider-a");
    }

    let status = monitor.get_health("provider-a");
    assert_eq!(status, ProviderHealthStatus::Unhealthy);
}

/// Test that success resets failure count and restores healthy status
#[test]
fn test_success_resets_to_healthy() {
    let monitor = InMemoryHealthMonitor::new();

    // Make provider degraded
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Degraded
    );

    // Record success - should reset to healthy
    monitor.record_success("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Healthy
    );
}

/// Test custom thresholds
#[test]
fn test_custom_thresholds() {
    // Custom thresholds: degraded=1, unhealthy=3
    let monitor = InMemoryHealthMonitor::with_thresholds(1, 3);

    // 1 failure should trigger degraded
    monitor.record_failure("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Degraded
    );

    // 2 more failures (total 3) should trigger unhealthy
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a");
    assert_eq!(
        monitor.get_health("provider-a"),
        ProviderHealthStatus::Unhealthy
    );
}

/// Test get_all_health returns all tracked providers
#[test]
fn test_get_all_health() {
    let monitor = InMemoryHealthMonitor::new();

    monitor.record_success("provider-a");
    monitor.record_failure("provider-b");
    monitor.record_failure("provider-b");

    let all_health = monitor.get_all_health();

    assert_eq!(all_health.len(), 2);
    assert_eq!(
        all_health.get("provider-a"),
        Some(&ProviderHealthStatus::Healthy)
    );
    assert_eq!(
        all_health.get("provider-b"),
        Some(&ProviderHealthStatus::Degraded)
    );
}

// =============================================================================
// NullHealthMonitor Tests (for comparison/completeness)
// =============================================================================

/// Test null health monitor always returns healthy
#[test]
fn test_null_health_monitor_returns_healthy() {
    let monitor = NullHealthMonitor::new();

    let status = monitor.get_health("any-provider");
    assert_eq!(status, ProviderHealthStatus::Healthy);

    let status = monitor.get_health("unknown");
    assert_eq!(status, ProviderHealthStatus::Healthy);
}

/// Test null health monitor ignores failures
#[test]
fn test_null_health_monitor_ignores_failures() {
    let monitor = NullHealthMonitor::new();

    // Record many failures
    for _ in 0..10 {
        monitor.record_failure("failing-provider");
    }

    // Still healthy (null impl ignores failures)
    assert_eq!(
        monitor.get_health("failing-provider"),
        ProviderHealthStatus::Healthy
    );
}

/// Test null health monitor returns empty stats
#[test]
fn test_null_health_monitor_empty_stats() {
    let monitor = NullHealthMonitor::new();
    let all_health = monitor.get_all_health();
    assert!(all_health.is_empty());
}

// =============================================================================
// DefaultProviderRouter Tests
// =============================================================================

/// Test router selects healthy provider over unhealthy
#[tokio::test]
async fn test_router_prefers_healthy_provider() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    // Make provider-a unhealthy
    for _ in 0..5 {
        monitor.record_failure("provider-a");
    }

    // provider-b stays healthy
    monitor.record_success("provider-b");

    let router = DefaultProviderRouter::new(
        monitor,
        vec!["provider-a".to_string(), "provider-b".to_string()],
        vec![],
    );

    let context = ProviderContext::new();
    let selected = router.select_embedding_provider(&context).await.unwrap();

    // Should select provider-b (healthy) over provider-a (unhealthy)
    assert_eq!(selected, "provider-b");
}

/// Test router respects excluded providers
#[tokio::test]
async fn test_router_excludes_providers() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    let router = DefaultProviderRouter::new(
        monitor,
        vec!["provider-a".to_string(), "provider-b".to_string()],
        vec![],
    );

    // Exclude provider-a
    let context = ProviderContext::new().exclude("provider-a");
    let selected = router.select_embedding_provider(&context).await.unwrap();

    // Should select provider-b since provider-a is excluded
    assert_eq!(selected, "provider-b");
}

/// Test router respects preferred providers when healthy
#[tokio::test]
async fn test_router_prefers_preferred_provider() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    let router = DefaultProviderRouter::new(
        monitor,
        vec!["provider-a".to_string(), "provider-b".to_string()],
        vec![],
    );

    // Prefer provider-b
    let context = ProviderContext::new().prefer("provider-b");
    let selected = router.select_embedding_provider(&context).await.unwrap();

    // Should select provider-b since it's preferred and healthy
    assert_eq!(selected, "provider-b");
}

/// Test router falls back when preferred provider is unhealthy
#[tokio::test]
async fn test_router_fallback_when_preferred_unhealthy() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    // Make preferred provider unhealthy
    for _ in 0..5 {
        monitor.record_failure("provider-b");
    }

    let router = DefaultProviderRouter::new(
        monitor,
        vec!["provider-a".to_string(), "provider-b".to_string()],
        vec![],
    );

    // Prefer provider-b (unhealthy)
    let context = ProviderContext::new().prefer("provider-b");
    let selected = router.select_embedding_provider(&context).await.unwrap();

    // Should fall back to provider-a since provider-b is unhealthy
    assert_eq!(selected, "provider-a");
}

/// Test router prefers degraded over unhealthy
#[tokio::test]
async fn test_router_prefers_degraded_over_unhealthy() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    // provider-a: degraded (2 failures)
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a");

    // provider-b: unhealthy (5 failures)
    for _ in 0..5 {
        monitor.record_failure("provider-b");
    }

    let router = DefaultProviderRouter::new(
        monitor,
        vec!["provider-a".to_string(), "provider-b".to_string()],
        vec![],
    );

    let context = ProviderContext::new();
    let selected = router.select_embedding_provider(&context).await.unwrap();

    // Should select provider-a (degraded) over provider-b (unhealthy)
    assert_eq!(selected, "provider-a");
}

/// Test router reports failures correctly
#[tokio::test]
async fn test_router_report_failure() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    let router =
        DefaultProviderRouter::new(monitor.clone(), vec!["provider-a".to_string()], vec![]);

    // Report failures via router
    for _ in 0..5 {
        router
            .report_failure("provider-a", "timeout")
            .await
            .unwrap();
    }

    // Check health through router
    let health = router.get_provider_health("provider-a").await.unwrap();
    assert_eq!(health, ProviderHealthStatus::Unhealthy);
}

/// Test router reports success correctly
#[tokio::test]
async fn test_router_report_success() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    // Make provider degraded first
    monitor.record_failure("provider-a");
    monitor.record_failure("provider-a");

    let router = DefaultProviderRouter::new(monitor, vec!["provider-a".to_string()], vec![]);

    // Report success via router
    router.report_success("provider-a").await.unwrap();

    // Should be healthy now
    let health = router.get_provider_health("provider-a").await.unwrap();
    assert_eq!(health, ProviderHealthStatus::Healthy);
}

/// Test router returns error when no providers available
#[tokio::test]
async fn test_router_error_no_providers() {
    let monitor = Arc::new(InMemoryHealthMonitor::new());

    let router = DefaultProviderRouter::new(monitor, vec!["provider-a".to_string()], vec![]);

    // Exclude the only provider
    let context = ProviderContext::new().exclude("provider-a");
    let result = router.select_embedding_provider(&context).await;

    assert!(result.is_err());
}
