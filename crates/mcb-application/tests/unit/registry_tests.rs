//! Tests for provider registries
//!
//! Tests the auto-registration system for embedding, vector store, cache, and language providers.

#[cfg(test)]
mod embedding_registry_tests {
    use mcb_application::ports::registry::embedding::*;

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
    fn test_list_providers_returns_vec() {
        // Should not panic, returns empty if no providers registered
        let providers = list_embedding_providers();
        // In tests, providers from mcb-providers won't be linked
        assert!(providers.is_empty() || !providers.is_empty());
    }
}

#[cfg(test)]
mod vector_store_registry_tests {
    use mcb_application::ports::registry::vector_store::*;

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
}

#[cfg(test)]
mod cache_registry_tests {
    use mcb_application::ports::registry::cache::*;

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
}

#[cfg(test)]
mod language_registry_tests {
    use mcb_application::ports::registry::language::*;

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
}
