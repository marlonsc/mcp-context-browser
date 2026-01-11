//! Tests for health monitoring module
//!
//! Tests for health monitoring using DashMap for non-blocking operation.

use mcp_context_browser::adapters::providers::routing::health::{
    HealthCheckResult, HealthMonitor, HealthMonitorTrait, ProviderHealthChecker,
    ProviderHealthStatus, RealProviderHealthChecker,
};
use mcp_context_browser::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_health_monitor_creation() {
    let monitor = HealthMonitor::new();
    // Unknown providers are considered unhealthy (fail-safe behavior)
    assert!(!monitor.is_healthy("any").await);
}

#[tokio::test]
async fn test_provider_health_check_unregistered() {
    let registry = Arc::new(ProviderRegistry::new());
    let checker = RealProviderHealthChecker::new(registry);
    let result = checker.check_health("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_healthy_providers() {
    let monitor = HealthMonitor::new();
    monitor
        .record_result(HealthCheckResult {
            provider_id: "p1".to_string(),
            status: ProviderHealthStatus::Healthy,
            response_time: Duration::from_millis(10),
            error_message: None,
        })
        .await;

    let healthy = monitor.list_healthy_providers().await;
    assert_eq!(healthy.len(), 1);
    assert_eq!(healthy[0], "p1");
}

#[tokio::test]
async fn test_real_provider_health_checker() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let registry = Arc::new(ProviderRegistry::new());
    let mock_provider = Arc::new(
        mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider::new(),
    );
    registry.register_embedding_provider("mock".to_string(), mock_provider)?;

    let checker = RealProviderHealthChecker::new(registry);
    let result = checker.check_health("mock").await?;
    assert_eq!(result.status, ProviderHealthStatus::Healthy);
    assert_eq!(result.provider_id, "mock");
    Ok(())
}
