//! Tests for configuration modules
//!
//! This module tests the configuration components:
//! - providers.rs: Provider-specific configuration
//! - types.rs: Core configuration structs

use mcp_context_browser::infrastructure::config::providers::ProviderConfigManager;
use mcp_context_browser::infrastructure::config::providers::ProviderHealth;
use mcp_context_browser::infrastructure::config::Config;
// use mcp_context_browser::domain::types::EmbeddingConfig;
use validator::Validate;

/// Test validation functionality
#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_config_validation_basic() {
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Default config should be valid");
    }
}

/// Test provider configuration module functionality
#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn test_provider_config_manager_creation() {
        let manager = ProviderConfigManager::new();
        assert!(
            manager.is_ready(),
            "Provider config manager should be ready after creation"
        );
    }

    #[test]
    fn test_provider_health_check() {
        let manager = ProviderConfigManager::new();

        // Initially no health information should be available
        let embedding_health = manager.check_embedding_provider_health();
        assert!(
            embedding_health.is_none(),
            "Should not have embedding provider health info initially"
        );

        let vector_store_health = manager.check_vector_store_provider_health();
        assert!(
            vector_store_health.is_none(),
            "Should not have vector store provider health info initially"
        );

        // After updating health, it should be available
        manager.update_provider_health("embedding", ProviderHealth::Healthy);
        let embedding_health = manager.check_embedding_provider_health();
        assert!(
            embedding_health.is_some(),
            "Should have embedding provider health info after update"
        );
        assert!(matches!(embedding_health.unwrap(), ProviderHealth::Healthy));
    }
}

/// Test core configuration struct
#[cfg(test)]
mod core_config_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        // Test that defaults are sensible
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(config.metrics.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Test TOML serialization
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok(), "Config should serialize to TOML");

        // Test deserialization
        let deserialized: Result<Config, _> = toml::from_str(&toml_str.unwrap());
        assert!(deserialized.is_ok(), "Config should deserialize from TOML");
    }
}
