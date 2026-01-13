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
        let health = embedding_health.expect("Should have embedding provider health");
        assert!(matches!(health, ProviderHealth::Healthy));
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
    fn test_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();

        // Test TOML serialization
        let toml_str = toml::to_string(&config)?;

        // Test deserialization
        let deserialized: Config = toml::from_str(&toml_str)?;
        assert_eq!(config.server.host, deserialized.server.host);
        Ok(())
    }
}
