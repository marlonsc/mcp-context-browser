//! Configuration providers management
//!
//! Manages configuration for embedding and vector store providers.

use crate::config::data::*;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use std::collections::HashMap;

/// Provider configuration manager
#[derive(Clone)]
pub struct ProviderConfigManager {
    embedding_configs: HashMap<String, EmbeddingConfig>,
    vector_store_configs: HashMap<String, VectorStoreConfig>,
}

impl ProviderConfigManager {
    /// Create a new provider config manager
    pub fn new(
        embedding_configs: HashMap<String, EmbeddingConfig>,
        vector_store_configs: HashMap<String, VectorStoreConfig>,
    ) -> Self {
        Self {
            embedding_configs,
            vector_store_configs,
        }
    }

    /// Get embedding provider configuration by name
    pub fn get_embedding_config(&self, name: &str) -> Result<&EmbeddingConfig> {
        self.embedding_configs.get(name).ok_or_else(|| {
            Error::Configuration {
                message: format!("Embedding provider '{}' not found", name),
                source: None,
            }
        })
    }

    /// Get vector store provider configuration by name
    pub fn get_vector_store_config(&self, name: &str) -> Result<&VectorStoreConfig> {
        self.vector_store_configs.get(name).ok_or_else(|| {
            Error::Configuration {
                message: format!("Vector store provider '{}' not found", name),
                source: None,
            }
        })
    }

    /// List all embedding provider names
    pub fn list_embedding_providers(&self) -> Vec<String> {
        self.embedding_configs.keys().cloned().collect()
    }

    /// List all vector store provider names
    pub fn list_vector_store_providers(&self) -> Vec<String> {
        self.vector_store_configs.keys().cloned().collect()
    }

    /// Check if an embedding provider is configured
    pub fn has_embedding_provider(&self, name: &str) -> bool {
        self.embedding_configs.contains_key(name)
    }

    /// Check if a vector store provider is configured
    pub fn has_vector_store_provider(&self, name: &str) -> bool {
        self.vector_store_configs.contains_key(name)
    }

    /// Get default embedding provider configuration
    pub fn get_default_embedding_config(&self) -> Option<&EmbeddingConfig> {
        self.embedding_configs.values().next()
    }

    /// Get default vector store provider configuration
    pub fn get_default_vector_store_config(&self) -> Option<&VectorStoreConfig> {
        self.vector_store_configs.values().next()
    }

    /// Validate all provider configurations
    pub fn validate(&self) -> Result<()> {
        // Validate embedding providers
        for (name, config) in &self.embedding_configs {
            self.validate_embedding_config(name, config)?;
        }

        // Validate vector store providers
        for (name, config) in &self.vector_store_configs {
            self.validate_vector_store_config(name, config)?;
        }

        Ok(())
    }

    /// Validate a single embedding provider configuration
    fn validate_embedding_config(&self, name: &str, config: &EmbeddingConfig) -> Result<()> {
        // Validate model is not empty
        if config.model.trim().is_empty() {
            return Err(Error::Configuration {
                message: format!("Embedding provider '{}': model cannot be empty", name),
                source: None,
            });
        }

        // Validate dimensions if specified
        if let Some(dimensions) = config.dimensions {
            if dimensions == 0 {
                return Err(Error::Configuration {
                    message: format!("Embedding provider '{}': dimensions cannot be 0", name),
                    source: None,
                });
            }
        }

        // Validate max tokens if specified
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens == 0 {
                return Err(Error::Configuration {
                    message: format!("Embedding provider '{}': max_tokens cannot be 0", name),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Validate a single vector store provider configuration
    fn validate_vector_store_config(&self, name: &str, config: &VectorStoreConfig) -> Result<()> {
        // Validate collection if specified
        if let Some(ref collection) = config.collection {
            if collection.trim().is_empty() {
                return Err(Error::Configuration {
                    message: format!("Vector store provider '{}': collection cannot be empty", name),
                    source: None,
                });
            }
        }

        // Validate dimensions if specified
        if let Some(dimensions) = config.dimensions {
            if dimensions == 0 {
                return Err(Error::Configuration {
                    message: format!("Vector store provider '{}': dimensions cannot be 0", name),
                    source: None,
                });
            }
        }

        // Validate timeout if specified
        if let Some(timeout_secs) = config.timeout_secs {
            if timeout_secs == 0 {
                return Err(Error::Configuration {
                    message: format!("Vector store provider '{}': timeout_secs cannot be 0", name),
                    source: None,
                });
            }
        }

        Ok(())
    }
}

/// Provider configuration builder
pub struct ProviderConfigBuilder {
    embedding_configs: HashMap<String, EmbeddingConfig>,
    vector_store_configs: HashMap<String, VectorStoreConfig>,
}

impl ProviderConfigBuilder {
    /// Create a new provider config builder
    pub fn new() -> Self {
        Self {
            embedding_configs: HashMap::new(),
            vector_store_configs: HashMap::new(),
        }
    }

    /// Add an embedding provider configuration
    pub fn with_embedding_provider<S: Into<String>>(
        mut self,
        name: S,
        config: EmbeddingConfig,
    ) -> Self {
        self.embedding_configs.insert(name.into(), config);
        self
    }

    /// Add a vector store provider configuration
    pub fn with_vector_store_provider<S: Into<String>>(
        mut self,
        name: S,
        config: VectorStoreConfig,
    ) -> Self {
        self.vector_store_configs.insert(name.into(), config);
        self
    }

    /// Add a default OpenAI embedding provider
    pub fn with_openai_embedding<S: Into<String>>(self, name: S, api_key: S) -> Self {
        use mcb_domain::value_objects::EmbeddingProviderKind;

        let config = EmbeddingConfig {
            provider: EmbeddingProviderKind::OpenAI,
            model: "text-embedding-ada-002".to_string(),
            api_key: Some(api_key.into()),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };

        self.with_embedding_provider(name, config)
    }

    /// Add a default filesystem vector store
    pub fn with_filesystem_vector_store<S: Into<String>>(self, name: S) -> Self {
        use mcb_domain::value_objects::VectorStoreProviderKind;

        let config = VectorStoreConfig {
            provider: VectorStoreProviderKind::Filesystem,
            address: None,
            token: None,
            collection: Some("mcb_vectors".to_string()),
            dimensions: None,
            timeout_secs: Some(30),
        };

        self.with_vector_store_provider(name, config)
    }

    /// Build the provider configuration manager
    pub fn build(self) -> ProviderConfigManager {
        ProviderConfigManager::new(self.embedding_configs, self.vector_store_configs)
    }
}

impl Default for ProviderConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcb_domain::value_objects::{EmbeddingProviderKind, VectorStoreProviderKind};

    #[test]
    fn test_provider_config_manager() {
        let mut embedding_configs = HashMap::new();
        embedding_configs.insert(
            "openai".to_string(),
            EmbeddingConfig {
                provider: EmbeddingProviderKind::OpenAI,
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
                provider: VectorStoreProviderKind::Filesystem,
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
        let builder = ProviderConfigBuilder::new()
            .with_embedding_provider(
                "invalid",
                EmbeddingConfig {
                    provider: EmbeddingProviderKind::OpenAI,
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
}