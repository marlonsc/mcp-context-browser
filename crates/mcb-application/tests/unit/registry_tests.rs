//! Tests for provider registries
//!
//! Tests the auto-registration system for embedding, vector store, cache, and language providers.
//! Uses `extern crate mcb_providers` to force linkme registration of real providers.
//!
//! ## Key Principle
//!
//! These tests validate that the linkme distributed slice registry system works correctly
//! by actually resolving and using registered providers, not just testing config builders.

// Force linkme registration of all providers from mcb-providers
extern crate mcb_providers;

use mcb_application::ports::registry::cache::*;
use mcb_application::ports::registry::embedding::*;
use mcb_application::ports::registry::language::*;
use mcb_application::ports::registry::vector_store::*;

// ============================================================================
// Embedding Registry Tests - Real Provider Resolution
// ============================================================================

#[cfg(test)]
mod embedding_registry_tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = EmbeddingProviderConfig::new("test")
            .with_model("model-1")
            .with_api_key("secret")
            .with_base_url("http://localhost")
            .with_dimensions(384)
            .with_extra("custom", "value");

        assert_eq!(config.provider, "test");
        assert_eq!(config.model, Some("model-1".to_string()));
        assert_eq!(config.api_key, Some("secret".to_string()));
        assert_eq!(config.base_url, Some("http://localhost".to_string()));
        assert_eq!(config.dimensions, Some(384));
        assert_eq!(config.extra.get("custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_list_providers_includes_null_provider() {
        // With extern crate mcb_providers, providers should be registered
        let providers = list_embedding_providers();

        // Should have at least the null provider
        assert!(
            !providers.is_empty(),
            "Should have registered providers (linkme should work with extern crate)"
        );

        // Verify null provider is registered
        let has_null = providers.iter().any(|(name, _)| *name == "null");
        assert!(
            has_null,
            "Null provider should be registered. Available: {:?}",
            providers
        );
    }

    #[test]
    fn test_resolve_null_embedding_provider() {
        // Create config for null provider
        let config = EmbeddingProviderConfig::new("null");

        // Resolve should succeed with real factory function
        let result = resolve_embedding_provider(&config);

        assert!(
            result.is_ok(),
            "Should resolve null provider, got error: {}",
            result
                .as_ref()
                .err()
                .map(|e| e.as_str())
                .unwrap_or("unknown")
        );

        // Verify the resolved provider has expected properties
        let provider = result.expect("Provider should be valid");
        assert_eq!(provider.provider_name(), "null", "Should be null provider");
        assert_eq!(
            provider.dimensions(),
            384,
            "Null provider has 384 dimensions"
        );
    }

    #[test]
    fn test_resolve_unknown_provider_fails() {
        let config = EmbeddingProviderConfig::new("nonexistent_provider_xyz");

        let result = resolve_embedding_provider(&config);

        assert!(result.is_err(), "Should fail for unknown provider");

        // Error message should be helpful
        match result {
            Err(err) => {
                assert!(
                    err.contains("Unknown embedding provider"),
                    "Error should describe the issue: {}",
                    err
                );
            }
            Ok(_) => panic!("Expected error for unknown provider"),
        }
    }

    #[test]
    fn test_list_providers_has_descriptions() {
        let providers = list_embedding_providers();

        for (name, description) in &providers {
            assert!(!name.is_empty(), "Provider name should not be empty");
            assert!(
                !description.is_empty(),
                "Provider '{}' should have a description",
                name
            );
        }
    }
}

// ============================================================================
// Vector Store Registry Tests - Real Provider Resolution
// ============================================================================

#[cfg(test)]
mod vector_store_registry_tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = VectorStoreProviderConfig::new("milvus")
            .with_uri("http://localhost:19530")
            .with_collection("embeddings")
            .with_dimensions(384)
            .with_encryption("secret-key");

        assert_eq!(config.provider, "milvus");
        assert_eq!(config.uri, Some("http://localhost:19530".to_string()));
        assert_eq!(config.collection, Some("embeddings".to_string()));
        assert_eq!(config.dimensions, Some(384));
        assert_eq!(config.encrypted, Some(true));
    }

    #[test]
    fn test_list_vector_store_providers() {
        let providers = list_vector_store_providers();

        assert!(
            !providers.is_empty(),
            "Should have registered vector store providers"
        );

        // Check for memory/null provider
        let has_memory = providers
            .iter()
            .any(|(name, _)| *name == "memory" || *name == "null");
        assert!(
            has_memory,
            "Should have memory or null vector store provider. Available: {:?}",
            providers
        );
    }

    #[test]
    fn test_resolve_memory_vector_store_provider() {
        let config = VectorStoreProviderConfig::new("memory");

        let result = resolve_vector_store_provider(&config);

        assert!(
            result.is_ok(),
            "Should resolve memory vector store, got error: {}",
            result
                .as_ref()
                .err()
                .map(|e| e.as_str())
                .unwrap_or("unknown")
        );

        let provider = result.expect("Provider should be valid");
        // In-memory provider should work
        assert!(
            provider.provider_name() == "in_memory" || provider.provider_name() == "memory",
            "Should be in-memory provider"
        );
    }
}

// ============================================================================
// Cache Registry Tests - Real Provider Resolution
// ============================================================================

#[cfg(test)]
mod cache_registry_tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = CacheProviderConfig::new("redis")
            .with_uri("redis://localhost:6379")
            .with_max_size(10000)
            .with_ttl_secs(3600)
            .with_namespace("mcb");

        assert_eq!(config.provider, "redis");
        assert_eq!(config.uri, Some("redis://localhost:6379".to_string()));
        assert_eq!(config.max_size, Some(10000));
        assert_eq!(config.ttl_secs, Some(3600));
        assert_eq!(config.namespace, Some("mcb".to_string()));
    }

    #[test]
    fn test_list_cache_providers() {
        let providers = list_cache_providers();

        assert!(
            !providers.is_empty(),
            "Should have registered cache providers"
        );

        // Check for null provider
        let has_null = providers.iter().any(|(name, _)| *name == "null");
        assert!(
            has_null,
            "Should have null cache provider. Available: {:?}",
            providers
        );
    }

    #[test]
    fn test_resolve_null_cache_provider() {
        let config = CacheProviderConfig::new("null");

        let result = resolve_cache_provider(&config);

        assert!(
            result.is_ok(),
            "Should resolve null cache provider, got error: {}",
            result
                .as_ref()
                .err()
                .map(|e| e.as_str())
                .unwrap_or("unknown")
        );

        let provider = result.expect("Provider should be valid");
        assert_eq!(
            provider.provider_name(),
            "null",
            "Should be null cache provider"
        );
    }
}

// ============================================================================
// Language Registry Tests - Real Provider Resolution
// ============================================================================

#[cfg(test)]
mod language_registry_tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = LanguageProviderConfig::new("universal")
            .with_max_chunk_size(4096)
            .with_min_chunk_size(100)
            .with_overlap(50);

        assert_eq!(config.provider, "universal");
        assert_eq!(config.max_chunk_size, Some(4096));
        assert_eq!(config.min_chunk_size, Some(100));
        assert_eq!(config.overlap, Some(50));
    }

    #[test]
    fn test_list_language_providers() {
        let providers = list_language_providers();

        assert!(
            !providers.is_empty(),
            "Should have registered language providers"
        );

        // Check for universal or null provider
        let has_universal = providers
            .iter()
            .any(|(name, _)| *name == "universal" || *name == "null");
        assert!(
            has_universal,
            "Should have universal or null language provider. Available: {:?}",
            providers
        );
    }

    #[test]
    fn test_resolve_universal_language_provider() {
        let config = LanguageProviderConfig::new("universal");

        let result = resolve_language_provider(&config);

        assert!(
            result.is_ok(),
            "Should resolve universal language provider, got error: {}",
            result
                .as_ref()
                .err()
                .map(|e| e.as_str())
                .unwrap_or("unknown")
        );
    }
}

// ============================================================================
// Integration Tests - Cross-Registry Validation
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_all_registries_have_providers() {
        // All registries should have at least one provider registered
        let embedding_providers = list_embedding_providers();
        let vector_store_providers = list_vector_store_providers();
        let cache_providers = list_cache_providers();
        let language_providers = list_language_providers();

        assert!(
            !embedding_providers.is_empty(),
            "Embedding registry should not be empty"
        );
        assert!(
            !vector_store_providers.is_empty(),
            "Vector store registry should not be empty"
        );
        assert!(
            !cache_providers.is_empty(),
            "Cache registry should not be empty"
        );
        assert!(
            !language_providers.is_empty(),
            "Language registry should not be empty"
        );
    }

    #[test]
    fn test_null_providers_available_for_testing() {
        // Null providers should be available for testing scenarios
        let null_embedding = resolve_embedding_provider(&EmbeddingProviderConfig::new("null"));
        let null_cache = resolve_cache_provider(&CacheProviderConfig::new("null"));

        assert!(
            null_embedding.is_ok(),
            "Null embedding provider should be resolvable for tests"
        );
        assert!(
            null_cache.is_ok(),
            "Null cache provider should be resolvable for tests"
        );
    }
}
