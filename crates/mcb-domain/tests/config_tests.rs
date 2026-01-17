//! Unit tests for Configuration value objects

#[cfg(test)]
mod tests {
    use mcb_domain::{EmbeddingConfig, VectorStoreConfig};

    #[test]
    fn test_embedding_config_creation() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: Some("sk-...".to_string()),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "text-embedding-ada-002");
        assert_eq!(config.api_key, Some("sk-...".to_string()));
        assert_eq!(config.base_url, None);
        assert_eq!(config.dimensions, Some(1536));
        assert_eq!(config.max_tokens, Some(8191));
    }

    #[test]
    fn test_embedding_config_minimal() {
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "default-model".to_string(),
            api_key: None,
            base_url: None,
            dimensions: None,
            max_tokens: None,
        };

        assert_eq!(config.provider, "fastembed");
        assert_eq!(config.model, "default-model");
        assert_eq!(config.api_key, None);
        assert_eq!(config.base_url, None);
        assert_eq!(config.dimensions, None);
        assert_eq!(config.max_tokens, None);
    }

    #[test]
    fn test_embedding_config_with_base_url() {
        let config = EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            dimensions: Some(4096),
            max_tokens: Some(4096),
        };

        assert_eq!(config.provider, "ollama");
        assert_eq!(config.base_url, Some("http://localhost:11434".to_string()));
        assert_eq!(config.dimensions, Some(4096));
    }

    #[test]
    fn test_vector_store_config_creation() {
        let config = VectorStoreConfig {
            provider: "qdrant".to_string(),
            address: Some("localhost:6334".to_string()),
            token: None,
            collection: Some("my-collection".to_string()),
            dimensions: Some(1536),
            timeout_secs: Some(30),
        };

        assert_eq!(config.provider, "qdrant");
        assert_eq!(config.address, Some("localhost:6334".to_string()));
        assert_eq!(config.token, None);
        assert_eq!(config.collection, Some("my-collection".to_string()));
        assert_eq!(config.dimensions, Some(1536));
        assert_eq!(config.timeout_secs, Some(30));
    }

    #[test]
    fn test_vector_store_config_filesystem() {
        let config = VectorStoreConfig {
            provider: "filesystem".to_string(),
            address: None,
            token: None,
            collection: Some("local-vectors".to_string()),
            dimensions: Some(384),
            timeout_secs: None,
        };

        assert_eq!(config.provider, "filesystem");
        assert_eq!(config.address, None);
        assert_eq!(config.token, None);
        assert_eq!(config.collection, Some("local-vectors".to_string()));
    }

    #[test]
    fn test_vector_store_config_remote() {
        let config = VectorStoreConfig {
            provider: "milvus".to_string(),
            address: Some("milvus-service:19530".to_string()),
            token: Some("root:Milvus".to_string()),
            collection: Some("embeddings".to_string()),
            dimensions: Some(768),
            timeout_secs: Some(60),
        };

        assert_eq!(config.provider, "milvus");
        assert_eq!(config.address, Some("milvus-service:19530".to_string()));
        assert_eq!(config.token, Some("root:Milvus".to_string()));
        assert_eq!(config.timeout_secs, Some(60));
    }
}
