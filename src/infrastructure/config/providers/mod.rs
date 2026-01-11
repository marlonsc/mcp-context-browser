//! Provider configuration management module
//!
//! This module provides comprehensive management for AI and vector store providers
//! including health checking, configuration validation, and provider selection logic.

pub mod embedding;
pub mod vector_store;

pub use embedding::EmbeddingProviderConfig;
pub use vector_store::VectorStoreProviderConfig;

use crate::domain::error::{Error, Result};
use dashmap::DashMap;

/// Health status of a provider
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderHealth {
    /// Provider is healthy and operational
    Healthy,
    /// Provider is experiencing issues but may still work
    Degraded,
    /// Provider is completely unavailable
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Provider configuration manager
pub struct ProviderConfigManager {
    health_cache: DashMap<String, ProviderHealth>,
    last_health_check: DashMap<String, std::time::Instant>,
}

impl ProviderConfigManager {
    /// Create a new provider configuration manager
    pub fn new() -> Self {
        Self {
            health_cache: DashMap::new(),
            last_health_check: DashMap::new(),
        }
    }

    /// Check if the manager is ready for operations
    pub fn is_ready(&self) -> bool {
        true // Always ready for now
    }

    /// Validate embedding provider configuration
    pub fn validate_embedding_config(
        &self,
        config: &crate::domain::types::EmbeddingConfig,
    ) -> Result<()> {
        // Basic validation since ConfigValidator is gone
        if config.provider.is_empty() {
            return Err(Error::config("Provider name cannot be empty"));
        }
        Ok(())
    }

    /// Validate vector store provider configuration
    pub fn validate_vector_store_config(
        &self,
        config: &crate::domain::types::VectorStoreConfig,
    ) -> Result<()> {
        // Basic validation
        if config.provider.is_empty() {
            return Err(Error::config("Provider name cannot be empty"));
        }
        Ok(())
    }

    /// Check health of embedding provider
    pub fn check_embedding_provider_health(&self) -> Option<ProviderHealth> {
        self.health_cache
            .get("embedding")
            .map(|r| r.value().clone())
    }

    /// Check health of vector store provider
    pub fn check_vector_store_provider_health(&self) -> Option<ProviderHealth> {
        self.health_cache
            .get("vector_store")
            .map(|r| r.value().clone())
    }

    /// Update provider health status
    pub fn update_provider_health(&self, provider_type: &str, health: ProviderHealth) {
        self.health_cache.insert(provider_type.to_string(), health);
        self.last_health_check
            .insert(provider_type.to_string(), std::time::Instant::now());
    }

    /// Get all provider health statuses
    pub fn get_all_provider_health(&self) -> std::collections::HashMap<String, ProviderHealth> {
        self.health_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get recommended provider based on health and performance
    pub fn get_recommended_provider(&self, provider_type: &str) -> Option<String> {
        match provider_type {
            "embedding" => {
                // For now, just return a default recommendation
                // In the future, this could be based on actual performance metrics
                Some("openai".to_string())
            }
            "vector_store" => Some("in-memory".to_string()),
            _ => None,
        }
    }

    /// Check if a provider configuration is compatible with requirements
    pub fn is_provider_compatible(
        &self,
        config: &EmbeddingProviderConfig,
        requirements: &ProviderRequirements,
    ) -> bool {
        match config {
            EmbeddingProviderConfig::OpenAI {
                dimensions,
                max_tokens,
                ..
            } => {
                if let Some(req_dims) = requirements.min_dimensions {
                    if let Some(cfg_dims) = dimensions {
                        if *cfg_dims < req_dims {
                            return false;
                        }
                    }
                }
                if let Some(req_tokens) = requirements.max_tokens {
                    if let Some(cfg_tokens) = max_tokens {
                        if *cfg_tokens < req_tokens {
                            return false;
                        }
                    }
                }
                true
            }
            EmbeddingProviderConfig::Ollama {
                dimensions,
                max_tokens,
                ..
            } => {
                // Similar checks for Ollama
                if let Some(req_dims) = requirements.min_dimensions {
                    if let Some(cfg_dims) = dimensions {
                        if *cfg_dims < req_dims {
                            return false;
                        }
                    }
                }
                if let Some(req_tokens) = requirements.max_tokens {
                    if let Some(cfg_tokens) = max_tokens {
                        if *cfg_tokens < req_tokens {
                            return false;
                        }
                    }
                }
                true
            }
            EmbeddingProviderConfig::VoyageAI {
                dimensions,
                max_tokens,
                ..
            } => {
                if let Some(req_dims) = requirements.min_dimensions {
                    if let Some(cfg_dims) = dimensions {
                        if *cfg_dims < req_dims {
                            return false;
                        }
                    }
                }
                if let Some(req_tokens) = requirements.max_tokens {
                    if let Some(cfg_tokens) = max_tokens {
                        if *cfg_tokens < req_tokens {
                            return false;
                        }
                    }
                }
                true
            }
            EmbeddingProviderConfig::Gemini {
                dimensions,
                max_tokens,
                ..
            } => {
                if let Some(req_dims) = requirements.min_dimensions {
                    if let Some(cfg_dims) = dimensions {
                        if *cfg_dims < req_dims {
                            return false;
                        }
                    }
                }
                if let Some(req_tokens) = requirements.max_tokens {
                    if let Some(cfg_tokens) = max_tokens {
                        if *cfg_tokens < req_tokens {
                            return false;
                        }
                    }
                }
                true
            }
            EmbeddingProviderConfig::Mock { .. } | EmbeddingProviderConfig::FastEmbed { .. } => {
                // Mock and FastEmbed providers are always compatible for basic requirements
                true
            }
        }
    }
}

/// Requirements for provider compatibility
#[derive(Debug, Clone)]
pub struct ProviderRequirements {
    /// Minimum dimensions required
    pub min_dimensions: Option<usize>,
    /// Maximum tokens supported
    pub max_tokens: Option<usize>,
    /// Required features
    pub required_features: Vec<String>,
}

impl Default for ProviderRequirements {
    fn default() -> Self {
        Self {
            min_dimensions: Some(128), // Minimum reasonable embedding dimension
            max_tokens: Some(512),     // Minimum token support
            required_features: vec!["embeddings".to_string()],
        }
    }
}

impl Default for ProviderConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let openai_config = crate::domain::types::EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("sk-test123".to_string()),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };
        assert!(manager.validate_embedding_config(&openai_config).is_ok());

        // Invalid config (empty provider)
        let invalid_config = crate::domain::types::EmbeddingConfig {
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
}
