//! Configuration validation module
//!
//! This module provides comprehensive validation for all configuration components
//! including server settings, provider configurations, and system parameters.

use crate::config::{Config, EmbeddingProviderConfig, VectorStoreProviderConfig};
use crate::core::error::{Error, Result};
use crate::core::types::{EmbeddingConfig, VectorStoreConfig};

/// Configuration validator with comprehensive validation rules
pub struct ConfigValidator {
    strict_mode: bool,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self {
            strict_mode: true,
        }
    }

    /// Create a validator with relaxed validation rules
    pub fn relaxed() -> Self {
        Self {
            strict_mode: false,
        }
    }

    /// Validate a complete configuration
    pub fn validate(&self, config: &Config) -> Result<()> {
        // Validate server configuration
        self.validate_server_config(&config.server)?;

        // Validate metrics configuration
        self.validate_metrics_config(&config.metrics)?;

        // Validate provider configurations
        self.validate_provider_configs(&config.providers)?;

        // Validate sync configuration
        self.validate_sync_config(&config.sync)?;

        // Validate daemon configuration
        self.validate_daemon_config(&config.daemon)?;

        Ok(())
    }

    /// Validate server configuration
    pub fn validate_server_config(&self, server: &crate::config::ServerConfig) -> Result<()> {
        if server.port == 0 {
            return Err(Error::config("Server port cannot be zero"));
        }

        if server.host.is_empty() {
            return Err(Error::config("Server host cannot be empty"));
        }

        // Validate host format (basic check)
        if server.host.contains("://") {
            return Err(Error::config("Server host should not include protocol (http/https)"));
        }

        Ok(())
    }

    /// Validate metrics configuration
    pub fn validate_metrics_config(&self, metrics: &crate::config::MetricsConfig) -> Result<()> {
        if metrics.port == 0 {
            return Err(Error::config("Metrics port cannot be zero"));
        }

        // Ensure metrics port doesn't conflict with server port
        // This would be checked at the config level, not here

        Ok(())
    }

    /// Validate provider configurations
    pub fn validate_provider_configs(&self, providers: &crate::config::ProviderConfig) -> Result<()> {
        // Validate embedding provider
        if let Some(ref embedding) = providers.embedding {
            self.validate_embedding_provider(embedding)?;
        } else if self.strict_mode {
            return Err(Error::config("Embedding provider configuration is required"));
        }

        // Validate vector store provider
        if let Some(ref vector_store) = providers.vector_store {
            self.validate_vector_store_provider(vector_store)?;
        } else if self.strict_mode {
            return Err(Error::config("Vector store provider configuration is required"));
        }

        Ok(())
    }

    /// Validate embedding provider configuration
    pub fn validate_embedding_provider(&self, config: &EmbeddingProviderConfig) -> Result<()> {
        match config {
            EmbeddingProviderConfig::OpenAI { model, api_key, .. } => {
                if model.is_empty() {
                    return Err(Error::config("OpenAI model cannot be empty"));
                }
                if api_key.is_empty() {
                    return Err(Error::config("OpenAI API key cannot be empty"));
                }
            }
            EmbeddingProviderConfig::Ollama { model, .. } => {
                if model.is_empty() {
                    return Err(Error::config("Ollama model cannot be empty"));
                }
            }
            EmbeddingProviderConfig::VoyageAI { model, api_key, .. } => {
                if model.is_empty() {
                    return Err(Error::config("VoyageAI model cannot be empty"));
                }
                if api_key.is_empty() {
                    return Err(Error::config("VoyageAI API key cannot be empty"));
                }
            }
            EmbeddingProviderConfig::Gemini { model, api_key, .. } => {
                if model.is_empty() {
                    return Err(Error::config("Gemini model cannot be empty"));
                }
                if api_key.is_empty() {
                    return Err(Error::config("Gemini API key cannot be empty"));
                }
            }
            EmbeddingProviderConfig::FastEmbed { model, .. } => {
                if let Some(model) = model {
                    if model.is_empty() {
                        return Err(Error::config("FastEmbed model cannot be empty"));
                    }
                }
            }
            EmbeddingProviderConfig::Mock { .. } => {
                // Mock provider has no validation requirements
            }
        }
        Ok(())
    }

    /// Validate vector store provider configuration
    pub fn validate_vector_store_provider(&self, config: &VectorStoreProviderConfig) -> Result<()> {
        match config {
            VectorStoreProviderConfig::Milvus { address, .. } => {
                if address.is_empty() {
                    return Err(Error::config("Milvus address cannot be empty"));
                }
            }
            VectorStoreProviderConfig::Pinecone { token, collection, .. } => {
                if token.is_empty() {
                    return Err(Error::config("Pinecone API token cannot be empty"));
                }
                if collection.is_empty() {
                    return Err(Error::config("Pinecone collection name cannot be empty"));
                }
            }
            VectorStoreProviderConfig::Qdrant { url, .. } => {
                if url.is_empty() {
                    return Err(Error::config("Qdrant URL cannot be empty"));
                }
            }
            VectorStoreProviderConfig::EdgeDB { .. } => {
                // EdgeDB validation would go here if needed
            }
            VectorStoreProviderConfig::InMemory { .. } => {
                // In-memory store has no validation requirements
            }
        }
        Ok(())
    }

    /// Validate sync configuration
    pub fn validate_sync_config(&self, sync: &crate::config::SyncConfig) -> Result<()> {
        if sync.interval_ms == 0 {
            return Err(Error::config("Sync interval cannot be zero"));
        }

        if sync.interval_ms < 1000 && self.strict_mode {
            return Err(Error::config("Sync interval should be at least 1000ms for performance"));
        }

        Ok(())
    }

    /// Validate daemon configuration
    pub fn validate_daemon_config(&self, daemon: &crate::config::DaemonConfig) -> Result<()> {
        if daemon.cleanup_interval_secs == 0 {
            return Err(Error::config("Daemon cleanup interval cannot be zero"));
        }

        if daemon.monitoring_interval_secs == 0 {
            return Err(Error::config("Daemon monitoring interval cannot be zero"));
        }

        if daemon.cleanup_interval_secs < daemon.monitoring_interval_secs && self.strict_mode {
            return Err(Error::config("Cleanup interval should be greater than or equal to monitoring interval"));
        }

        Ok(())
    }

    /// Validate embedding config (legacy compatibility)
    pub fn validate_embedding_config(&self, config: &EmbeddingConfig) -> Result<()> {
        // Convert to new format for validation
        let provider_config = match config.provider.to_lowercase().as_str() {
            "openai" => EmbeddingProviderConfig::OpenAI {
                model: config.model.clone(),
                api_key: config.api_key.clone().unwrap_or_default(),
                base_url: config.base_url.clone(),
                dimensions: config.dimensions,
                max_tokens: config.max_tokens,
            },
            "ollama" => EmbeddingProviderConfig::Ollama {
                model: config.model.clone(),
                host: config.base_url.clone(),
                dimensions: config.dimensions,
                max_tokens: config.max_tokens,
            },
            "voyageai" => EmbeddingProviderConfig::VoyageAI {
                model: config.model.clone(),
                api_key: config.api_key.clone().unwrap_or_default(),
                base_url: config.base_url.clone(),
                dimensions: config.dimensions,
                max_tokens: config.max_tokens,
            },
            "gemini" => EmbeddingProviderConfig::Gemini {
                model: config.model.clone(),
                api_key: config.api_key.clone().unwrap_or_default(),
                base_url: config.base_url.clone(),
                dimensions: config.dimensions,
                max_tokens: config.max_tokens,
            },
            "fastembed" => EmbeddingProviderConfig::FastEmbed {
                model: Some(config.model.clone()),
                dimensions: config.dimensions,
                max_tokens: config.max_tokens,
            },
            _ => return Err(Error::config(format!("Unknown embedding provider: {}", config.provider))),
        };

        self.validate_embedding_provider(&provider_config)
    }

    /// Validate vector store config (legacy compatibility)
    pub fn validate_vector_store_config(&self, config: &VectorStoreConfig) -> Result<()> {
        // Convert to new format for validation
        let provider_config = match config.provider.to_lowercase().as_str() {
            "milvus" => VectorStoreProviderConfig::Milvus {
                address: config.address.clone().unwrap_or_default(),
                database: config.database.clone(),
                collection: config.collection.clone().unwrap_or_else(|| "default".to_string()),
                dimensions: config.dimensions,
            },
            "pinecone" => VectorStoreProviderConfig::Pinecone {
                token: config.token.clone().unwrap_or_default(),
                collection: config.collection.clone().unwrap_or_default(),
                dimensions: config.dimensions,
                base_url: config.base_url.clone(),
            },
            "qdrant" => VectorStoreProviderConfig::Qdrant {
                url: config.address.clone().unwrap_or_default(),
                collection: config.collection.clone().unwrap_or_else(|| "default".to_string()),
                dimensions: config.dimensions,
                api_key: config.token.clone(),
            },
            "in-memory" => VectorStoreProviderConfig::InMemory {
                dimensions: config.dimensions.unwrap_or(1536),
            },
            _ => return Err(Error::config(format!("Unknown vector store provider: {}", config.provider))),
        };

        self.validate_vector_store_provider(&provider_config)
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, MetricsConfig, ProviderConfig, SyncConfig, DaemonConfig};

    #[test]
    fn test_validator_creation() {
        let validator = ConfigValidator::new();
        assert!(validator.strict_mode);

        let relaxed = ConfigValidator::relaxed();
        assert!(!relaxed.strict_mode);
    }

    #[test]
    fn test_server_config_validation() {
        let validator = ConfigValidator::new();

        // Valid config
        let valid_config = ServerConfig {
            host: "localhost".to_string(),
            port: 3000,
        };
        assert!(validator.validate_server_config(&valid_config).is_ok());

        // Invalid port
        let invalid_config = ServerConfig {
            host: "localhost".to_string(),
            port: 0,
        };
        assert!(validator.validate_server_config(&invalid_config).is_err());

        // Invalid host
        let invalid_host = ServerConfig {
            host: "".to_string(),
            port: 3000,
        };
        assert!(validator.validate_server_config(&invalid_host).is_err());
    }

    #[test]
    fn test_embedding_provider_validation() {
        let validator = ConfigValidator::new();

        // Valid OpenAI config
        let openai_config = EmbeddingProviderConfig::OpenAI {
            model: "text-embedding-3-small".to_string(),
            api_key: "sk-test123".to_string(),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };
        assert!(validator.validate_embedding_provider(&openai_config).is_ok());

        // Invalid OpenAI config (empty API key)
        let invalid_openai = EmbeddingProviderConfig::OpenAI {
            model: "text-embedding-3-small".to_string(),
            api_key: "".to_string(),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8191),
        };
        assert!(validator.validate_embedding_provider(&invalid_openai).is_err());
    }

    #[test]
    fn test_vector_store_provider_validation() {
        let validator = ConfigValidator::new();

        // Valid InMemory config
        let in_memory_config = VectorStoreProviderConfig::InMemory {
            dimensions: 1536,
        };
        assert!(validator.validate_vector_store_provider(&in_memory_config).is_ok());

        // Invalid Milvus config (empty address)
        let invalid_milvus = VectorStoreProviderConfig::Milvus {
            address: "".to_string(),
            database: None,
            collection: "default".to_string(),
            dimensions: Some(1536),
        };
        assert!(validator.validate_vector_store_provider(&invalid_milvus).is_err());
    }
}