//! Tests for refactored configuration modules
//!
//! This module tests the separated configuration components:
//! - validation.rs: Input validation logic
//! - providers.rs: Provider-specific configuration
//! - environment.rs: Environment variable handling
//! - config.rs: Core configuration struct

use mcp_context_browser::config::{
    Config, GlobalConfig, ServerConfig, ProviderConfig, MetricsConfig,
    EmbeddingProviderConfig, VectorStoreProviderConfig,
};
use mcp_context_browser::config::validation::ConfigValidator;
use mcp_context_browser::config::providers::{ProviderConfigManager, ProviderHealth};
use mcp_context_browser::config::environment::EnvironmentLoader;
use std::collections::HashMap;

/// Test validation module functionality
#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_config_validation_basic() {
        let config = Config::default();
        let validator = ConfigValidator::new();

        let result = validator.validate(&config);
        assert!(result.is_ok(), "Default config should be valid");
    }

    #[test]
    fn test_config_validation_invalid_provider() {
        let mut config = Config::default();
        // Set invalid provider configuration
        config.provider.embedding = Some(EmbeddingProviderConfig::OpenAI {
            model: "".to_string(), // Invalid: empty model
            api_key: "".to_string(), // Invalid: empty API key
            base_url: None,
            dimensions: None,
            max_tokens: None,
        });

        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        assert!(result.is_err(), "Config with invalid provider should fail validation");
    }

    #[test]
    fn test_config_validation_missing_required_fields() {
        let mut config = Config::default();
        config.server.host = "".to_string(); // Invalid: empty host

        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        assert!(result.is_err(), "Config with missing required fields should fail validation");
    }
}

/// Test provider configuration module functionality
#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn test_provider_config_manager_creation() {
        let manager = ProviderConfigManager::new();
        assert!(manager.is_ready(), "Provider config manager should be ready after creation");
    }

    #[test]
    fn test_provider_health_check() {
        let manager = ProviderConfigManager::new();

        // Test embedding provider health
        let embedding_health = manager.check_embedding_provider_health();
        assert!(embedding_health.is_some(), "Should have embedding provider health info");

        // Test vector store provider health
        let vector_store_health = manager.check_vector_store_provider_health();
        assert!(vector_store_health.is_some(), "Should have vector store provider health info");
    }

    #[test]
    fn test_provider_configuration_loading() {
        let manager = ProviderConfigManager::new();

        // Test loading OpenAI provider config
        let openai_config = EmbeddingProviderConfig::OpenAI {
            model: "text-embedding-3-small".to_string(),
            api_key: "sk-test123".to_string(),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };

        let result = manager.validate_embedding_config(&openai_config);
        assert!(result.is_ok(), "Valid OpenAI config should pass validation");
    }
}

/// Test environment loading module functionality
#[cfg(test)]
mod environment_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_environment_loader_creation() {
        let loader = EnvironmentLoader::new();
        assert!(loader.is_ready(), "Environment loader should be ready after creation");
    }

    #[test]
    fn test_environment_variable_loading() {
        let loader = EnvironmentLoader::new();

        // Set test environment variables
        env::set_var("MCP_SERVER_HOST", "127.0.0.1");
        env::set_var("MCP_SERVER_PORT", "3000");
        env::set_var("MCP_OPENAI_API_KEY", "test-key");

        let result = loader.load_server_config();
        assert!(result.is_ok(), "Should successfully load server config from environment");

        let server_config = result.unwrap();
        assert_eq!(server_config.host, "127.0.0.1");
        assert_eq!(server_config.port, 3000);

        // Clean up
        env::remove_var("MCP_SERVER_HOST");
        env::remove_var("MCP_SERVER_PORT");
        env::remove_var("MCP_OPENAI_API_KEY");
    }

    #[test]
    fn test_environment_variable_precedence() {
        let loader = EnvironmentLoader::new();

        // Set environment variables
        env::set_var("MCP_SERVER_PORT", "8080");

        // Config file should override environment
        let mut config = Config::default();
        config.server.port = 3000; // Config file value

        let result = loader.merge_with_environment(config);
        assert!(result.is_ok(), "Should successfully merge config with environment");

        let merged_config = result.unwrap();
        assert_eq!(merged_config.server.port, 3000, "Config file should take precedence");

        // Clean up
        env::remove_var("MCP_SERVER_PORT");
    }
}

/// Test core configuration struct (reduced size)
#[cfg(test)]
mod core_config_tests {
    use super::*;

    #[test]
    fn test_config_builder_pattern() {
        let config = Config::builder()
            .server_host("localhost".to_string())
            .server_port(8080)
            .build()
            .expect("Builder should create valid config");

        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        // Test that defaults are sensible
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(config.metrics.enabled_by_default);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Test TOML serialization
        let toml = toml::to_string(&config);
        assert!(toml.is_ok(), "Config should serialize to TOML");

        // Test deserialization
        let deserialized: Result<Config, _> = toml::from_str(&toml.unwrap());
        assert!(deserialized.is_ok(), "Config should deserialize from TOML");
    }
}

/// Integration tests for the refactored config system
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_config_workflow() {
        // Create config via builder
        let config = Config::builder()
            .server_host("0.0.0.0".to_string())
            .server_port(3001)
            .embedding_provider(EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test123".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            })
            .vector_store_provider(VectorStoreProviderConfig::InMemory {
                dimensions: 1536,
                max_chunks: Some(10000),
            })
            .build()
            .expect("Full config should build successfully");

        // Validate the config
        let validator = ConfigValidator::new();
        let validation_result = validator.validate(&config);
        assert!(validation_result.is_ok(), "Built config should be valid");

        // Test provider manager with the config
        let provider_manager = ProviderConfigManager::new();
        let embedding_health = provider_manager.check_embedding_provider_health();
        assert!(embedding_health.is_some());

        // Test environment merging
        let env_loader = EnvironmentLoader::new();
        let merged_result = env_loader.merge_with_environment(config);
        assert!(merged_result.is_ok(), "Should successfully merge with environment");
    }

    #[test]
    fn test_config_error_propagation() {
        // Test that validation errors are properly propagated
        let mut config = Config::default();
        config.server.port = 0; // Invalid port

        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        assert!(result.is_err(), "Invalid config should produce error");

        let error = result.unwrap_err();
        assert!(error.to_string().contains("port"), "Error should mention port validation");
    }
}