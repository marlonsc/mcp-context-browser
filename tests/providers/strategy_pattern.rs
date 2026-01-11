//! Tests for Strategy pattern implementation in provider architecture
//!
//! This module tests the generic provider services that use trait bounds
//! instead of dynamic dispatch, implementing the Strategy pattern.

use mcp_context_browser::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use mcp_context_browser::application::context::{ContextService, GenericContextService};
use mcp_context_browser::domain::error::Result;
use mcp_context_browser::{CodeChunk, Embedding, SearchResult};
use std::sync::Arc;

/// Test generic context service with strategy pattern
#[cfg(test)]
mod generic_context_service_tests {
    use super::*;
    use mcp_context_browser::adapters::providers::{
        InMemoryVectorStoreProvider, MockEmbeddingProvider,
    };

    use mcp_context_browser::domain::ports::HybridSearchProvider;

    fn dummy_hybrid_search() -> Arc<dyn HybridSearchProvider> {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        Arc::new(mcp_context_browser::adapters::HybridSearchAdapter::new(
            sender,
        ))
    }

    #[test]
    fn test_generic_context_service_creation() {
        // Create concrete provider instances
        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        // Create generic context service with compile-time strategy types
        let context_service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        assert_eq!(context_service.embedding_dimensions(), 384); // Mock provider dimensions
    }

    #[test]
    fn test_generic_context_service_operations() -> Result<(), Box<dyn std::error::Error>> {
        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        let context_service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        // Test that we can embed text
        let text = "fn hello() { println!(\"Hello, world!\"); }";
        let result = tokio::runtime::Runtime::new()?
            .block_on(async { context_service.embed_text(text).await });

        assert!(result.is_ok());
        let embedding = result?;
        assert_eq!(embedding.vector.len(), 384);
        Ok(())
    }

    #[test]
    fn test_generic_context_service_with_different_providers() {
        // This test would verify that the same generic service can work with different provider types
        // For now, we just test with the same providers but the structure allows for different concrete types

        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        let context_service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        // Verify the service is properly constructed
        assert!(context_service.embedding_dimensions() > 0);
    }
}

/// Test strategy pattern interfaces and composition
#[cfg(test)]
mod strategy_pattern_tests {
    use super::*;

    #[test]
    fn test_provider_trait_bounds() {
        // Test that providers implement the required traits
        fn accepts_embedding_provider<E: EmbeddingProvider>(_provider: Arc<E>) {}
        fn accepts_vector_store_provider<V: VectorStoreProvider>(_provider: Arc<V>) {}

        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        // These should compile without trait bound errors
        accepts_embedding_provider(embedding_provider);
        accepts_vector_store_provider(vector_store_provider);
    }

    #[test]
    fn test_provider_strategy_composition() {
        // Test that we can compose different strategies at compile time
        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        // Create a service that composes these strategies
        let service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        // The service should be able to perform operations using both strategies
        assert_eq!(service.embedding_dimensions(), 384);
    }

    #[test]
    fn test_strategy_pattern_benefits() {
        // Test that the strategy pattern allows for:
        // 1. Compile-time type safety
        // 2. No dynamic dispatch overhead for core operations
        // 3. Better optimization opportunities

        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        let service = GenericContextService::new(
            embedding_provider.clone(),
            vector_store_provider.clone(),
            dummy_hybrid_search(),
        );

        // Test that we can call methods directly on the concrete types
        // while still using the trait bounds for polymorphism
        assert!(service.embedding_dimensions() > 0);
    }
}

/// Test provider validation with strategy pattern
#[cfg(test)]
mod provider_validation_tests {
    use super::*;
    use mcp_context_browser::infrastructure::config::providers::ProviderConfigManager;

    #[test]
    fn test_strategy_based_provider_validation() {
        let manager = ProviderConfigManager::new();

        // Test validation works with strategy pattern
        // (This would be expanded with actual provider configurations)

        assert!(manager.is_ready());
    }

    #[test]
    fn test_provider_compatibility_with_strategies() {
        // Test that provider compatibility checking works with strategy pattern
        // This would validate that certain provider combinations are compatible

        let manager = ProviderConfigManager::new();
        assert!(manager.is_ready());
    }
}

/// Integration tests for strategy pattern implementation
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_strategy_pattern_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Test a complete workflow using the strategy pattern:
        // 1. Create providers (strategies)
        // 2. Compose them into a service
        // 3. Use the service for operations
        // 4. Verify results

        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        let context_service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        // Test basic functionality
        assert_eq!(context_service.embedding_dimensions(), 384);

        // Test that the service can be used in async context
        let result = tokio::runtime::Runtime::new()?
            .block_on(async { context_service.embed_text("test code").await });

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_strategy_pattern_performance_characteristics() {
        // Test that strategy pattern provides expected performance characteristics
        // (compile-time resolution, no dynamic dispatch for core paths)

        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

        let service = GenericContextService::new(
            embedding_provider,
            vector_store_provider,
            dummy_hybrid_search(),
        );

        // Performance should be consistent (no dynamic dispatch overhead)
        let start = std::time::Instant::now();
        let _dimensions = service.embedding_dimensions();
        let elapsed = start.elapsed();

        // Should be very fast (microseconds)
        assert!(elapsed.as_micros() < 1000);
    }
}
