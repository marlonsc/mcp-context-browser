//! Tests for failover management module
//!
//! Tests for failover strategies and the failover manager.

use mcp_context_browser::adapters::providers::routing::failover::{
    FailoverContext, FailoverManager, PriorityBasedStrategy, RoundRobinStrategy,
};
use mcp_context_browser::adapters::providers::routing::health::{
    HealthCheckResult, HealthMonitor, HealthMonitorTrait, ProviderHealthChecker,
    ProviderHealthStatus,
};
use mcp_context_browser::domain::error::{Error, Result};
use mcp_context_browser::infrastructure::di::registry::ProviderRegistry;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_priority_based_failover() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(ProviderRegistry::new());
    let health_monitor = Arc::new(HealthMonitor::with_registry(registry));

    // Mark providers as healthy
    let _ = health_monitor.check_provider("primary").await;
    let _ = health_monitor.check_provider("secondary").await;
    let _ = health_monitor.check_provider("tertiary").await;

    let strategy = PriorityBasedStrategy::new();
    strategy.set_priority("primary", 1);
    strategy.set_priority("secondary", 2);
    strategy.set_priority("tertiary", 3);

    let manager = FailoverManager::with_strategy(health_monitor.clone(), Box::new(strategy));

    let candidates = vec![
        "primary".to_string(),
        "secondary".to_string(),
        "tertiary".to_string(),
    ];

    // Register providers as healthy (unknown providers are now considered unhealthy)
    for provider in &candidates {
        health_monitor
            .record_result(HealthCheckResult {
                provider_id: provider.clone(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
            .await;
    }

    let context = FailoverContext::new("test");
    let result = manager.select_provider(&candidates, &context).await?;

    // Should succeed since providers are registered as healthy
    assert_eq!(result, "primary");
    Ok(())
}

#[tokio::test]
async fn test_round_robin_failover() {
    let registry = Arc::new(ProviderRegistry::new());
    let health_monitor = Arc::new(HealthMonitor::with_registry(registry));
    let strategy = RoundRobinStrategy::new();
    let manager = FailoverManager::with_strategy(Arc::clone(&health_monitor), Box::new(strategy));

    let candidates = vec![
        "provider1".to_string(),
        "provider2".to_string(),
        "provider3".to_string(),
    ];

    // Register providers as healthy (unknown providers are now considered unhealthy)
    for provider in &candidates {
        health_monitor
            .record_result(HealthCheckResult {
                provider_id: provider.clone(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
            .await;
    }

    let context = FailoverContext::new("test");
    let result = manager.select_provider(&candidates, &context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_failover_candidates() {
    let registry = Arc::new(ProviderRegistry::new());
    let health_monitor = Arc::new(HealthMonitor::with_registry(registry));
    let manager = FailoverManager::new(Arc::clone(&health_monitor));

    let all_providers = vec![
        "healthy1".to_string(),
        "healthy2".to_string(),
        "unhealthy".to_string(),
    ];

    // Register healthy1 and healthy2 as healthy, unhealthy as unhealthy
    for provider in &["healthy1", "healthy2"] {
        health_monitor
            .record_result(HealthCheckResult {
                provider_id: provider.to_string(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
            .await;
    }
    health_monitor
        .record_result(HealthCheckResult {
            provider_id: "unhealthy".to_string(),
            status: ProviderHealthStatus::Unhealthy,
            response_time: Duration::from_millis(10),
            error_message: Some("test".to_string()),
        })
        .await;

    let exclude = vec!["unhealthy".to_string()];

    let candidates = manager
        .get_failover_candidates(&all_providers, &exclude)
        .await;
    assert_eq!(candidates.len(), 2); // Only healthy providers returned
}

#[tokio::test]
async fn test_failover_manager_creation() {
    let registry = Arc::new(ProviderRegistry::new());
    let _manager = FailoverManager::with_registry(registry);
}

#[tokio::test]
async fn test_execute_with_failover() {
    // Create a mock health checker
    struct MockHealthChecker;
    #[async_trait::async_trait]
    impl ProviderHealthChecker for MockHealthChecker {
        async fn check_health(&self, provider_id: &str) -> Result<HealthCheckResult> {
            Ok(HealthCheckResult {
                provider_id: provider_id.to_string(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
        }
    }

    let mock_checker = Arc::new(MockHealthChecker);
    let health_monitor = Arc::new(HealthMonitor::with_checker(mock_checker));

    // Initialize health status
    let _ = health_monitor.check_provider("failing").await;
    let _ = health_monitor.check_provider("success").await;

    let strategy = RoundRobinStrategy::new();
    let manager = FailoverManager::with_strategy(health_monitor, Box::new(strategy));

    let candidates = vec!["failing".to_string(), "success".to_string()];
    let mut context = FailoverContext::new("test");
    context.max_attempts = 2;

    let result = manager
        .execute_with_failover(&candidates, &context, |provider| async move {
            if provider == "failing" {
                Err(Error::generic("Operation failed"))
            } else {
                Ok(format!("Success with {}", provider))
            }
        })
        .await;

    assert!(result.is_ok());
}
