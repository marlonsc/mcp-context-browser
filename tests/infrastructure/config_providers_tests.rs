//! Provider configuration tests
//!
//! Tests migrated from src/infrastructure/config/providers/mod.rs

use mcp_context_browser::infrastructure::config::providers::{
    EmbeddingProviderConfig, ProviderConfigManager, ProviderHealth, ProviderRequirements,
};

#[test]
fn test_provider_config_manager_creation() {
    let manager = ProviderConfigManager::new();
    assert!(manager.is_ready());
}

#[test]
fn test_provider_health_updates() {
    let manager = ProviderConfigManager::new();

    // Initially no health data
    assert!(manager.check_embedding_provider_health().is_none());

    // Update health
    manager.update_provider_health("embedding", ProviderHealth::Healthy);
    assert_eq!(
        manager.check_embedding_provider_health(),
        Some(ProviderHealth::Healthy)
    );

    // Update to unhealthy
    manager.update_provider_health("embedding", ProviderHealth::Unhealthy);
    assert_eq!(
        manager.check_embedding_provider_health(),
        Some(ProviderHealth::Unhealthy)
    );
}

#[test]
fn test_provider_validation() {
    let manager = ProviderConfigManager::new();

    // Valid OpenAI config
    let openai_config = mcp_context_browser::domain::types::EmbeddingConfig {
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        api_key: Some("sk-test123".to_string()),
        base_url: None,
        dimensions: Some(1536),
        max_tokens: Some(8191),
    };
    assert!(manager.validate_embedding_config(&openai_config).is_ok());

    // Invalid config (empty provider)
    let invalid_config = mcp_context_browser::domain::types::EmbeddingConfig {
        provider: "".to_string(),
        model: "text-embedding-3-small".to_string(),
        api_key: Some("sk-test123".to_string()),
        base_url: None,
        dimensions: Some(1536),
        max_tokens: Some(8191),
    };
    let result = manager.validate_embedding_config(&invalid_config);
    println!("Validation result for empty provider: {:?}", result);
    assert!(result.is_err());
}

#[test]
fn test_provider_compatibility() {
    let manager = ProviderConfigManager::new();

    let openai_config = EmbeddingProviderConfig::OpenAI {
        model: "text-embedding-3-small".to_string(),
        api_key: "sk-test123".to_string(),
        base_url: None,
        dimensions: Some(1536),
        max_tokens: Some(8191),
    };

    // Compatible with default requirements
    let requirements = ProviderRequirements::default();
    assert!(manager.is_provider_compatible(&openai_config, &requirements));

    // Incompatible with high dimension requirements
    let high_req = ProviderRequirements {
        min_dimensions: Some(4096), // Higher than config supports
        max_tokens: Some(512),
        required_features: vec!["embeddings".to_string()],
    };
    assert!(!manager.is_provider_compatible(&openai_config, &high_req));
}

#[test]
fn test_recommended_provider() {
    let manager = ProviderConfigManager::new();

    assert_eq!(
        manager.get_recommended_provider("embedding"),
        Some("openai".to_string())
    );
    assert_eq!(
        manager.get_recommended_provider("vector_store"),
        Some("in-memory".to_string())
    );
    assert_eq!(manager.get_recommended_provider("unknown"), None);
}
