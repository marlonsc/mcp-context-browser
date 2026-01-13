//! Multi-Provider Strategy Example
//!
//! This example demonstrates how to use the basic provider routing system
//! with health monitoring and failover capabilities.

use mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider;
use mcp_context_browser::adapters::providers::routing::{
    CircuitBreaker, ContextualStrategy, CostTracker, CostTrackerConfig, FailoverManager,
    HealthMonitor, ProviderContext, ProviderMetricsCollector, ProviderRouter, ProviderRouterDeps,
};
use mcp_context_browser::domain::error::Error;
use mcp_context_browser::domain::error::Result;
use mcp_context_browser::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 MCP Context Browser - Multi-Provider Strategy Demo");
    println!("====================================================");

    // Initialize core components
    let registry = Arc::new(ProviderRegistry::new());

    // Create all dependencies for proper DI
    let health_monitor = Arc::new(HealthMonitor::with_registry(Arc::clone(&registry)));
    let persistence_dir = std::env::temp_dir().join("mcp-example");
    let circuit_breaker = Arc::new(CircuitBreaker::new("example", persistence_dir).await);
    let metrics = Arc::new(ProviderMetricsCollector::new()?);
    let cost_tracker = Arc::new(CostTracker::new(CostTrackerConfig::production()));
    let failover_manager = Arc::new(FailoverManager::new(Arc::clone(&health_monitor)));

    // Create provider router with explicit dependencies (proper DI pattern)
    let deps = ProviderRouterDeps::new(
        Arc::clone(&registry),
        Arc::clone(&health_monitor),
        circuit_breaker,
        metrics,
        cost_tracker,
        failover_manager,
    );
    let mut router = ProviderRouter::new(deps);

    // Set selection strategy
    let strategy = ContextualStrategy::new();
    router.set_selection_strategy(Box::new(strategy));

    println!("\n📦 Registering Providers...");

    // Register a mock embedding provider
    let mock_provider = Arc::new(NullEmbeddingProvider::new());
    registry.register_embedding_provider("mock".to_string(), mock_provider)?;

    println!("  ✅ Registered mock embedding provider");

    println!("\n🎯 Testing Provider Selection...");

    let context = ProviderContext::new("embedding");

    // Test provider selection
    match router.select_embedding_provider(&context).await {
        Ok(provider_id) => {
            println!("  ✅ Selected provider: {}", provider_id);
        }
        Err(e) => {
            println!("  ❌ Provider selection failed: {}", e);
        }
    }

    println!("\n🏥 Testing Health Monitoring...");

    // Test health monitoring
    router.record_success("mock", 0.1).await;
    println!("  ✅ Recorded success for mock provider");

    let error = Error::generic("Test failure");
    router.record_failure("mock", &error).await;
    println!("  ⚠️  Recorded failure for mock provider");

    println!("\n📊 Router Statistics:");
    let stats = router.get_statistics().await;
    println!("   Total Providers: {}", stats.total_providers);
    println!("   Healthy Providers: {}", stats.healthy_providers);
    println!("   Selection Strategy: {}", stats.strategy_name);

    println!("\n✅ Multi-Provider Strategy Demo Complete!");
    println!("\nKey Features Demonstrated:");
    println!("  • Provider routing with explicit dependency injection");
    println!("  • Health monitoring with success/failure tracking");
    println!("  • Multiple selection strategies");
    println!("  • Integration with existing project patterns");

    Ok(())
}
