//! Multi-Provider Strategy Example
//!
//! This example demonstrates how to use the basic provider routing system
//! with health monitoring and failover capabilities.

use mcp_context_browser::core::error::Error;
use mcp_context_browser::core::error::Result;
use mcp_context_browser::di::registry::ProviderRegistry;
use mcp_context_browser::providers::embedding::NullEmbeddingProvider;
use mcp_context_browser::providers::routing::{
    ContextualStrategy, ProviderContext, ProviderRouter,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 MCP Context Browser - Multi-Provider Strategy Demo");
    println!("====================================================");

    // Initialize core components
    let registry = Arc::new(ProviderRegistry::new());

    // Create provider router
    let mut router = ProviderRouter::with_defaults(Arc::clone(&registry))?;

    // Set selection strategy
    let strategy = ContextualStrategy::new();
    router.set_selection_strategy(Box::new(strategy));

    println!("\n📦 Registering Providers...");

    // Register a mock embedding provider
    let mock_provider = Arc::new(NullEmbeddingProvider::new());
    registry.register_embedding_provider("mock", mock_provider)?;

    println!("  ✅ Registered mock embedding provider");

    println!("\n🎯 Testing Provider Selection...");

    let context = ProviderContext::default();

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
    println!("  • Basic provider routing with registry integration");
    println!("  • Health monitoring with success/failure tracking");
    println!("  • Multiple selection strategies");
    println!("  • Integration with existing project patterns");

    Ok(())
}
