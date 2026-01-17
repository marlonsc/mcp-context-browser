//! Provider Configuration Tests

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use mcb_infrastructure::config::providers::{ProviderConfigBuilder, ProviderConfigManager};
use std::collections::HashMap;

#[test]
fn test_provider_config_manager() {
    let mut embedding_configs = HashMap::new();
    embedding_configs.insert(
        "openai".to_string(),
        EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        },
    );

    let mut vector_store_configs = HashMap::new();
    vector_store_configs.insert(
        "filesystem".to_string(),
        VectorStoreConfig {
            provider: "filesystem".to_string(),
            address: None,
            token: None,
            collection: Some("test".to_string()),
            dimensions: None,
            timeout_secs: Some(30),
        },
    );

    let manager = ProviderConfigManager::new(embedding_configs, vector_store_configs);

    assert!(manager.has_embedding_provider("openai"));
    assert!(manager.has_vector_store_provider("filesystem"));
    assert!(!manager.has_embedding_provider("nonexistent"));
}

#[test]
fn test_config_validation() {
    let builder = ProviderConfigBuilder::new().with_embedding_provider(
        "invalid",
        EmbeddingConfig {
            provider: "openai".to_string(),
            model: "".to_string(), // Invalid: empty model
            api_key: None,
            base_url: None,
            dimensions: Some(0), // Invalid: zero dimensions
            max_tokens: None,
        },
    );

    let manager = builder.build();
    assert!(manager.validate().is_err());
}

#[test]
fn test_provider_config_builder() {
    let manager = ProviderConfigBuilder::new()
        .with_openai_embedding("openai", "test-key")
        .with_filesystem_vector_store("fs")
        .build();

    assert!(manager.has_embedding_provider("openai"));
    assert!(manager.has_vector_store_provider("fs"));

    let embedding_config = manager.get_embedding_config("openai").unwrap();
    assert_eq!(embedding_config.model, "text-embedding-ada-002");
    assert_eq!(embedding_config.api_key.as_ref().unwrap(), "test-key");
}
