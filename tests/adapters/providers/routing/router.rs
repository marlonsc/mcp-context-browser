//! Tests for provider router module
//!
//! Tests for intelligent provider routing with dependency injection.

use mcp_context_browser::adapters::providers::routing::cost_tracker::CostTracker;
use mcp_context_browser::adapters::providers::routing::health::{
    HealthCheckResult, HealthMonitor, HealthMonitorTrait, ProviderHealthStatus,
};
use mcp_context_browser::adapters::providers::routing::router::{
    ContextualStrategy, ProviderContext, ProviderRouter, ProviderSelectionStrategy,
};
use mcp_context_browser::infrastructure::di::registry::ProviderRegistry;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_provider_router_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(ProviderRegistry::new());
    let router = ProviderRouter::with_defaults(Arc::clone(&registry)).await?;

    let stats = router.get_statistics().await;
    assert_eq!(stats.total_providers, 0);
    assert_eq!(stats.healthy_providers, 0);
    Ok(())
}

#[tokio::test]
async fn test_provider_selection_with_no_providers(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(ProviderRegistry::new());
    let router = ProviderRouter::with_defaults(Arc::clone(&registry)).await?;

    let context = ProviderContext::default();
    let result = router.select_embedding_provider(&context).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_contextual_strategy() {
    let registry = Arc::new(ProviderRegistry::new());
    let strategy = ContextualStrategy::new();
    let health_monitor = Arc::new(HealthMonitor::with_registry(Arc::clone(&registry)));
    let cost_tracker = Arc::new(CostTracker::new());

    let candidates = vec!["ollama".to_string(), "openai".to_string()];

    // Register providers as healthy (unknown providers are now considered unhealthy)
    for provider in &candidates {
        health_monitor
            .as_ref()
            .record_result(HealthCheckResult {
                provider_id: provider.clone(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
            .await;
    }

    let context = ProviderContext {
        cost_sensitivity: 1.0, // High cost sensitivity
        ..Default::default()
    };

    // Providers are registered as healthy, selection will succeed
    let result = strategy
        .select_provider(&candidates, &context, &health_monitor, &cost_tracker)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_provider_context_default() {
    let context = ProviderContext::default();
    assert_eq!(context.operation_type, "general");
    assert_eq!(context.cost_sensitivity, 0.5);
    assert_eq!(context.quality_requirement, 0.5);
    assert!(context.excluded_providers.is_empty());
}
