//! Configuration Management System
//!
//! Enterprise-grade configuration system with validation and environment support.
//! Provides type-safe configuration management similar to convict.js.
//!
//! ## Features
//!
//! - **Schema Validation**: Type-safe configuration with validation
//! - **Environment Variables**: Support for .env files and environment variables
//! - **Provider Configuration**: AI and vector store provider settings
//! - **Runtime Validation**: Configuration validation at startup
//! - **Documentation**: Comprehensive configuration documentation
//!
//! ## Configuration Sources (in priority order)
//!
//! 1. Environment variables (highest priority)
//! 2. Configuration files (~/.mcp-context-browser/config.toml)
//! 3. Default values (lowest priority)

use crate::core::auth::AuthConfig;
use crate::core::cache::CacheConfig;
use crate::core::database::DatabaseConfig;
use crate::core::error::{Error, Result};
use crate::core::hybrid_search::HybridSearchConfig;
use crate::core::limits::ResourceLimitsConfig;
use crate::core::rate_limit::RateLimitConfig;
use crate::daemon::DaemonConfig;
use crate::sync::SyncConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Module declarations
pub mod environment;
pub mod providers;
pub mod validation;

/// Embedding provider configuration types (similar to Claude Context)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum EmbeddingProviderConfig {
    #[serde(rename = "openai")]
    OpenAI {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "ollama")]
    Ollama {
        model: String,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "voyageai")]
    VoyageAI {
        model: String,
        api_key: String,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "gemini")]
    Gemini {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "mock")]
    Mock {
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "fastembed")]
    FastEmbed {
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
}

/// Vector store provider configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum VectorStoreProviderConfig {
    #[serde(rename = "edgevec")]
    EdgeVec {
        #[serde(default)]
        max_vectors: Option<usize>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        hnsw_m: Option<usize>,
        #[serde(default)]
        hnsw_ef_construction: Option<usize>,
        #[serde(default)]
        distance_metric: Option<String>,
        #[serde(default)]
        use_quantization: Option<bool>,
    },
    #[serde(rename = "milvus")]
    Milvus {
        address: String,
        #[serde(default)]
        token: Option<String>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "pinecone")]
    Pinecone {
        api_key: String,
        environment: String,
        index_name: String,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "qdrant")]
    Qdrant {
        url: String,
        #[serde(default)]
        api_key: Option<String>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "in-memory")]
    InMemory {
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "filesystem")]
    Filesystem {
        #[serde(default)]
        base_path: Option<String>,
        #[serde(default)]
        max_vectors_per_shard: Option<usize>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        compression_enabled: Option<bool>,
        #[serde(default)]
        index_cache_size: Option<usize>,
        #[serde(default)]
        memory_mapping_enabled: Option<bool>,
    },
}

/// Global configuration file structure (similar to ~/.context/.env in Claude Context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: GlobalProviderConfig,
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Sync configuration
    #[serde(default)]
    pub sync: SyncConfig,
    /// Daemon configuration
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProviderConfig {
    pub embedding: EmbeddingProviderConfig,
    pub vector_store: VectorStoreProviderConfig,
}

/// Main application configuration
///
/// Central configuration structure containing all settings for the MCP Context Browser.
/// Supports hierarchical configuration with validation and environment variable overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Server configuration (host, port, etc.)
    pub server: ServerConfig,
    /// AI and vector store provider configurations
    pub providers: ProviderConfig,
    /// Metrics and monitoring configuration
    pub metrics: MetricsConfig,
    /// Authentication and authorization settings
    #[serde(default)]
    pub auth: AuthConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Sync coordination configuration
    pub sync: SyncConfig,
    /// Background daemon configuration
    pub daemon: DaemonConfig,

    /// Resource limits configuration
    #[serde(default)]
    pub resource_limits: ResourceLimitsConfig,

    /// Advanced caching configuration
    #[serde(default)]
    pub cache: CacheConfig,
    pub hybrid_search: HybridSearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Legacy provider config (maintained for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub embedding: crate::core::types::EmbeddingConfig,
    pub vector_store: crate::core::types::VectorStoreConfig,
}

/// Configuration manager with schema validation (equivalent to Claude Context's convict.js)
pub struct ConfigManager {
    global_config_path: PathBuf,
    env_config: HashMap<String, String>,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| Error::config("Cannot determine home directory"))?;

        Ok(Self {
            global_config_path: home_dir.join(".context").join("config.toml"),
            env_config: std::env::vars().collect(),
        })
    }

    /// Load configuration with priority: Global file -> Environment -> Defaults
    pub fn load_config(&self) -> Result<Config> {
        // Start with defaults
        let mut config = Config::default();

        // Override with global config file if exists
        if let Ok(global_config) = self.load_global_config() {
            self.merge_global_config(&mut config, global_config);
        }

        // Override with environment variables (highest priority)
        self.merge_env_config(&mut config)?;

        // Validate configuration schema
        self.validate_config(&config)?;

        Ok(config)
    }

    /// Load global configuration file (~/.context/config.toml)
    fn load_global_config(&self) -> Result<GlobalConfig> {
        if !self.global_config_path.exists() {
            return Err(Error::config("Global config file not found"));
        }

        let content = fs::read_to_string(&self.global_config_path)
            .map_err(|e| Error::config(format!("Failed to read global config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| Error::config(format!("Invalid global config format: {}", e)))
    }

    /// Merge global config into main config
    fn merge_global_config(&self, config: &mut Config, global: GlobalConfig) {
        config.server = global.server;
        config.metrics = global.metrics;
        config.sync = global.sync;
        config.daemon = global.daemon;

        // Convert provider configs
        config.providers = ProviderConfig {
            embedding: self.provider_config_to_legacy(&global.providers.embedding),
            vector_store: self.vector_config_to_legacy(&global.providers.vector_store),
        };
    }

    /// Convert embedding provider config to legacy format
    fn provider_config_to_legacy(
        &self,
        config: &EmbeddingProviderConfig,
    ) -> crate::core::types::EmbeddingConfig {
        match config {
            EmbeddingProviderConfig::OpenAI {
                model,
                api_key,
                base_url,
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "openai".to_string(),
                model: model.clone(),
                api_key: Some(api_key.clone()),
                base_url: base_url.clone(),
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
            EmbeddingProviderConfig::Ollama {
                model,
                host,
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "ollama".to_string(),
                model: model.clone(),
                api_key: host.clone().map(|h| format!("http://{}", h)),
                base_url: host.clone(),
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
            EmbeddingProviderConfig::VoyageAI {
                model,
                api_key,
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "voyageai".to_string(),
                model: model.clone(),
                api_key: Some(api_key.clone()),
                base_url: None,
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
            EmbeddingProviderConfig::Gemini {
                model,
                api_key,
                base_url,
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "gemini".to_string(),
                model: model.clone(),
                api_key: Some(api_key.clone()),
                base_url: base_url.clone(),
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
            EmbeddingProviderConfig::FastEmbed {
                model,
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "fastembed".to_string(),
                model: model.clone().unwrap_or_else(|| "default".to_string()),
                api_key: None,
                base_url: None,
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
            EmbeddingProviderConfig::Mock {
                dimensions,
                max_tokens,
            } => crate::core::types::EmbeddingConfig {
                provider: "mock".to_string(),
                model: "mock".to_string(),
                api_key: None,
                base_url: None,
                dimensions: *dimensions,
                max_tokens: *max_tokens,
            },
        }
    }

    /// Convert vector store config to legacy format
    fn vector_config_to_legacy(
        &self,
        config: &VectorStoreProviderConfig,
    ) -> crate::core::types::VectorStoreConfig {
        match config {
            VectorStoreProviderConfig::EdgeVec {
                max_vectors: _,
                collection,
                hnsw_m: _,
                hnsw_ef_construction: _,
                distance_metric: _,
                use_quantization: _,
            } => crate::core::types::VectorStoreConfig {
                provider: "edgevec".to_string(),
                address: None,
                token: None,
                collection: collection.clone(),
                dimensions: Some(1536), // Default, will be overridden by EdgeVec config
            },
            VectorStoreProviderConfig::Milvus {
                address,
                token,
                collection,
                dimensions,
            } => crate::core::types::VectorStoreConfig {
                provider: "milvus".to_string(),
                address: Some(address.clone()),
                token: token.clone(),
                collection: collection.clone(),
                dimensions: *dimensions,
            },
            VectorStoreProviderConfig::Pinecone {
                api_key,
                environment,
                index_name,
                dimensions,
            } => crate::core::types::VectorStoreConfig {
                provider: "pinecone".to_string(),
                address: Some(format!("https://{}.pinecone.io", environment.clone())),
                token: Some(api_key.clone()),
                collection: Some(index_name.clone()),
                dimensions: *dimensions,
            },
            VectorStoreProviderConfig::Qdrant {
                url,
                api_key,
                collection,
                dimensions,
            } => crate::core::types::VectorStoreConfig {
                provider: "qdrant".to_string(),
                address: Some(url.clone()),
                token: api_key.clone(),
                collection: collection.clone(),
                dimensions: *dimensions,
            },
            VectorStoreProviderConfig::InMemory { dimensions } => {
                crate::core::types::VectorStoreConfig {
                    provider: "in-memory".to_string(),
                    address: None,
                    token: None,
                    collection: None,
                    dimensions: *dimensions,
                }
            }
            VectorStoreProviderConfig::Filesystem {
                base_path,
                max_vectors_per_shard: _,
                dimensions,
                compression_enabled: _,
                index_cache_size: _,
                memory_mapping_enabled: _,
            } => crate::core::types::VectorStoreConfig {
                provider: "filesystem".to_string(),
                address: base_path.clone(),
                token: None,
                collection: None,
                dimensions: *dimensions,
            },
        }
    }

    /// Merge environment variables into config
    fn merge_env_config(&self, config: &mut Config) -> Result<()> {
        // Server config
        if let Some(host) = self.env_config.get("MCP_HOST") {
            config.server.host = host.clone();
        }
        if let Some(port) = self.env_config.get("MCP_PORT") {
            config.server.port = port.parse().unwrap_or(3000);
        }

        // Metrics config
        if let Some(port) = self.env_config.get("CONTEXT_METRICS_PORT") {
            config.metrics.port = port.parse().unwrap_or(3001);
        }
        if let Some(enabled) = self.env_config.get("CONTEXT_METRICS_ENABLED") {
            config.metrics.enabled = enabled.parse().unwrap_or(true);
        }

        // Provider configs - use environment variables as override
        self.merge_provider_env_config(config)?;

        Ok(())
    }

    /// Merge provider-specific environment variables
    fn merge_provider_env_config(&self, config: &mut Config) -> Result<()> {
        // Embedding provider
        if let Some(provider) = self.env_config.get("EMBEDDING_PROVIDER") {
            config.providers.embedding.provider = provider.clone();
        }
        if let Some(model) = self.env_config.get("EMBEDDING_MODEL") {
            config.providers.embedding.model = model.clone();
        }

        // Provider-specific API keys
        match config.providers.embedding.provider.to_lowercase().as_str() {
            "openai" => {
                if let Some(key) = self.env_config.get("OPENAI_API_KEY") {
                    config.providers.embedding.api_key = Some(key.clone());
                }
                if let Some(url) = self.env_config.get("OPENAI_BASE_URL") {
                    config.providers.embedding.base_url = Some(url.clone());
                }
            }
            "ollama" => {
                if let Some(host) = self.env_config.get("OLLAMA_HOST") {
                    config.providers.embedding.base_url = Some(host.clone());
                }
                if let Some(model) = self.env_config.get("OLLAMA_MODEL") {
                    config.providers.embedding.model = model.clone();
                }
            }
            "voyageai" => {
                if let Some(key) = self.env_config.get("VOYAGEAI_API_KEY") {
                    config.providers.embedding.api_key = Some(key.clone());
                }
            }
            "gemini" => {
                if let Some(key) = self.env_config.get("GEMINI_API_KEY") {
                    config.providers.embedding.api_key = Some(key.clone());
                }
                if let Some(url) = self.env_config.get("GEMINI_BASE_URL") {
                    config.providers.embedding.base_url = Some(url.clone());
                }
            }
            _ => {}
        }

        // Vector store
        if let Some(provider) = self.env_config.get("VECTOR_STORE_PROVIDER") {
            config.providers.vector_store.provider = provider.clone();
        }

        match config.providers.vector_store.provider.as_str() {
            "milvus" => {
                if let Some(addr) = self.env_config.get("MILVUS_ADDRESS") {
                    config.providers.vector_store.address = Some(addr.clone());
                }
                if let Some(token) = self.env_config.get("MILVUS_TOKEN") {
                    config.providers.vector_store.token = Some(token.clone());
                }
            }
            "pinecone" => {
                if let Some(key) = self.env_config.get("PINECONE_API_KEY") {
                    config.providers.vector_store.token = Some(key.clone());
                }
                if let Some(env) = self.env_config.get("PINECONE_ENVIRONMENT") {
                    config.providers.vector_store.address =
                        Some(format!("https://{}.pinecone.io", env));
                }
            }
            "qdrant" => {
                if let Some(url) = self.env_config.get("QDRANT_URL") {
                    config.providers.vector_store.address = Some(url.clone());
                }
                if let Some(key) = self.env_config.get("QDRANT_API_KEY") {
                    config.providers.vector_store.token = Some(key.clone());
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate configuration schema (equivalent to Claude Context's convict.js)
    fn validate_config(&self, config: &Config) -> Result<()> {
        // Validate server config
        if config.server.port == 0 {
            return Err(Error::config("Server port cannot be zero"));
        }

        // Validate metrics config
        if config.metrics.port == 0 {
            return Err(Error::config("Metrics port cannot be zero"));
        }

        // Validate provider configs
        self.validate_embedding_config(&config.providers.embedding)?;
        self.validate_vector_store_config(&config.providers.vector_store)?;

        // Validate sync and daemon configs
        if config.sync.interval_ms == 0 {
            return Err(Error::config("Sync interval cannot be zero"));
        }
        if config.daemon.cleanup_interval_secs == 0 || config.daemon.monitoring_interval_secs == 0 {
            return Err(Error::config("Daemon intervals cannot be zero"));
        }

        Ok(())
    }

    /// Validate embedding provider configuration
    fn validate_embedding_config(
        &self,
        config: &crate::core::types::EmbeddingConfig,
    ) -> Result<()> {
        // Check required fields based on provider
        match config.provider.to_lowercase().as_str() {
            "openai" => {
                if config.api_key.is_none() || config.api_key.as_ref().unwrap().is_empty() {
                    return Err(Error::config("OpenAI API key is required"));
                }
                if config.model.is_empty() {
                    return Err(Error::config("OpenAI model is required"));
                }
            }
            "ollama" => {
                if config.model.is_empty() {
                    return Err(Error::config("Ollama model is required"));
                }
            }
            "voyageai" => {
                if config.api_key.is_none() || config.api_key.as_ref().unwrap().is_empty() {
                    return Err(Error::config("VoyageAI API key is required"));
                }
                if config.model.is_empty() {
                    return Err(Error::config("VoyageAI model is required"));
                }
            }
            "gemini" => {
                if config.api_key.is_none() || config.api_key.as_ref().unwrap().is_empty() {
                    return Err(Error::config("Gemini API key is required"));
                }
                if config.model.is_empty() {
                    return Err(Error::config("Gemini model is required"));
                }
            }
            "mock" => {} // Mock provider has no requirements
            _ => {
                return Err(Error::config(format!(
                    "Unknown embedding provider: {}",
                    config.provider
                )));
            }
        }

        // Validate dimensions if specified
        if let Some(dim) = config.dimensions
            && dim == 0
        {
            return Err(Error::config("Embedding dimensions cannot be zero"));
        }

        // Validate max tokens if specified
        if let Some(tokens) = config.max_tokens
            && tokens == 0
        {
            return Err(Error::config("Max tokens cannot be zero"));
        }

        Ok(())
    }

    /// Validate vector store configuration
    fn validate_vector_store_config(
        &self,
        config: &crate::core::types::VectorStoreConfig,
    ) -> Result<()> {
        match config.provider.to_lowercase().as_str() {
            "milvus" => {
                if config.address.is_none() || config.address.as_ref().unwrap().is_empty() {
                    return Err(Error::config("Milvus address is required"));
                }
            }
            "pinecone" => {
                if config.token.is_none() || config.token.as_ref().unwrap().is_empty() {
                    return Err(Error::config("Pinecone API key is required"));
                }
                if config.collection.is_none() || config.collection.as_ref().unwrap().is_empty() {
                    return Err(Error::config("Pinecone index name is required"));
                }
            }
            "qdrant" => {
                if config.address.is_none() || config.address.as_ref().unwrap().is_empty() {
                    return Err(Error::config("Qdrant URL is required"));
                }
            }
            "in-memory" => {} // In-memory has no requirements
            _ => {
                return Err(Error::config(format!(
                    "Unknown vector store provider: {}",
                    config.provider
                )));
            }
        }

        // Validate dimensions if specified
        if let Some(dim) = config.dimensions
            && dim == 0
        {
            return Err(Error::config("Vector dimensions cannot be zero"));
        }

        Ok(())
    }

    /// Create example configuration file at ~/.context/config.toml
    pub fn create_example_config(&self) -> Result<()> {
        let config_dir = self
            .global_config_path
            .parent()
            .ok_or_else(|| Error::config("Cannot determine config directory"))?;

        // Create ~/.context directory if it doesn't exist
        fs::create_dir_all(config_dir)
            .map_err(|e| Error::config(format!("Failed to create config directory: {}", e)))?;

        // Create example configuration
        let example_config = r#"# MCP Context Browser Configuration
# This file provides comprehensive configuration similar to Claude Context
# Save this file as: ~/.context/config.toml

# Server configuration
[server]
host = "127.0.0.1"
port = 3000

# Embedding provider configuration
[providers.embedding]
# Available providers: openai, ollama, voyageai, gemini, mock
provider = "openai"
model = "text-embedding-3-small"
api_key = "your-openai-api-key-here"
# Optional: custom base URL for OpenAI-compatible APIs
# base_url = "https://api.openai.com/v1"
dimensions = 1536
max_tokens = 8191

# Alternative embedding providers (uncomment to use):

# [providers.embedding]
# provider = "ollama"
# model = "nomic-embed-text"
# # host = "http://localhost:11434"  # Optional, defaults to localhost:11434
# dimensions = 768
# max_tokens = 2048

# [providers.embedding]
# provider = "voyageai"
# model = "voyage-code-3"
# api_key = "your-voyageai-api-key"
# dimensions = 1024
# max_tokens = 32000

# [providers.embedding]
# provider = "gemini"
# model = "gemini-embedding-001"
# api_key = "your-gemini-api-key"
# # base_url = "https://generativelanguage.googleapis.com"  # Optional
# dimensions = 768
# max_tokens = 2048

# [providers.embedding]
# provider = "mock"
# # No API key required - for development/testing
# dimensions = 128
# max_tokens = 512

# Vector store configuration
[providers.vector_store]
# Available providers: milvus, pinecone, qdrant, in-memory
provider = "milvus"
address = "http://localhost:19530"
# token = "your-milvus-token"  # Optional for local instances
collection = "mcp_context"
dimensions = 1536

# Alternative vector stores (uncomment to use):

# [providers.vector_store]
# provider = "pinecone"
# # address = "https://your-project.svc.us-east-1-aws.pinecone.io"  # Auto-generated from environment
# token = "your-pinecone-api-key"
# collection = "mcp-context-index"  # Your Pinecone index name
# dimensions = 1536

# [providers.vector_store]
# provider = "qdrant"
# address = "http://localhost:6334"
# # token = "your-qdrant-api-key"  # Optional
# collection = "mcp_context"
# dimensions = 1536

# [providers.vector_store]
# provider = "in-memory"
# # No configuration required - for development/testing
# dimensions = 128

# Metrics configuration
[metrics]
port = 3001
enabled = true

# Rate limiting for metrics API
[metrics.rate_limiting]
enabled = true

# For single-node deployments (default)
[metrics.rate_limiting.backend.memory]
max_entries = 10000

# For clustered deployments, uncomment and configure:
# [metrics.rate_limiting.backend.redis]
# url = "redis://127.0.0.1:6379"
# pool_size = 10

window_seconds = 60
max_requests_per_window = 100
burst_allowance = 20

# Sync coordination configuration
[sync]
interval_ms = 60000  # 1 minute
enable_lockfile = true
debounce_ms = 5000   # 5 seconds

# Background daemon configuration
[daemon]
cleanup_interval_secs = 300    # 5 minutes
monitoring_interval_secs = 60  # 1 minute
"#;

        fs::write(&self.global_config_path, example_config)
            .map_err(|e| Error::config(format!("Failed to write example config: {}", e)))?;

        println!(
            "✅ Example configuration created: {}",
            self.global_config_path.display()
        );
        println!("📝 Edit this file with your actual API keys and settings");
        println!("🔄 Then restart MCP Context Browser to use the new configuration");

        Ok(())
    }

    /// Print comprehensive configuration documentation
    pub fn print_config_guide(&self) {
        println!("🔧 MCP Context Browser - Professional Configuration Guide");
        println!("=========================================================");
        println!();
        println!("📁 Configuration Hierarchy (Claude Context style):");
        println!("  1. 🔴 Environment Variables (highest priority - overrides all)");
        println!("  2. 🟡 Global config file: ~/.context/config.toml");
        println!("  3. 🟢 Built-in defaults (lowest priority - fallback only)");
        println!();
        println!("📝 Quick Setup:");
        println!("  1. Run: mcp-context-browser --create-config");
        println!("  2. Edit: ~/.context/config.toml with your API keys");
        println!("  3. Restart the application");
        println!();
        println!("🌐 Environment Variables Reference:");
        println!();
        println!("# Core Providers");
        println!("EMBEDDING_PROVIDER=openai|ollama|voyageai|gemini|mock");
        println!(
            "EMBEDDING_MODEL=text-embedding-3-small|nomic-embed-text|voyage-code-3|gemini-embedding-001"
        );
        println!("VECTOR_STORE_PROVIDER=milvus|pinecone|qdrant|in-memory");
        println!();
        println!("# API Keys");
        println!("OPENAI_API_KEY=sk-your-openai-key");
        println!("OLLAMA_HOST=http://localhost:11434");
        println!("VOYAGEAI_API_KEY=your-voyageai-key");
        println!("GEMINI_API_KEY=your-gemini-key");
        println!("MILVUS_TOKEN=your-milvus-token");
        println!("PINECONE_API_KEY=your-pinecone-key");
        println!("QDRANT_API_KEY=your-qdrant-key");
        println!();
        println!("# Network Configuration");
        println!("MILVUS_ADDRESS=http://localhost:19530");
        println!("QDRANT_URL=http://localhost:6334");
        println!("MCP_HOST=127.0.0.1");
        println!("MCP_PORT=3000");
        println!("CONTEXT_METRICS_PORT=3001");
        println!();
        println!("✅ Schema Validation (like Claude Context's convict.js):");
        println!("  • 🔍 Provider-specific validation rules");
        println!("  • 🔐 API key presence verification");
        println!("  • 📏 Dimension and token limit validation");
        println!("  • 🌐 Network address format checking");
        println!("  • ⚡ Configuration merge with priority handling");
        println!();
        println!("📋 Provider-Specific Examples:");
        println!();
        println!("# OpenAI (Production recommended)");
        println!("EMBEDDING_PROVIDER=openai");
        println!("EMBEDDING_MODEL=text-embedding-3-small");
        println!("OPENAI_API_KEY=sk-your-key");
        println!();
        println!("# Ollama (Self-hosted)");
        println!("EMBEDDING_PROVIDER=ollama");
        println!("EMBEDDING_MODEL=nomic-embed-text");
        println!("OLLAMA_HOST=http://localhost:11434");
        println!();
        println!("# VoyageAI (Code-optimized)");
        println!("EMBEDDING_PROVIDER=voyageai");
        println!("EMBEDDING_MODEL=voyage-code-3");
        println!("VOYAGEAI_API_KEY=your-voyage-key");
        println!();
        println!("# Gemini (Google ecosystem)");
        println!("EMBEDDING_PROVIDER=gemini");
        println!("EMBEDDING_MODEL=gemini-embedding-001");
        println!("GEMINI_API_KEY=your-gemini-key");
        println!();
        println!("# Mock (Development only)");
        println!("EMBEDDING_PROVIDER=mock");
        println!("VECTOR_STORE_PROVIDER=in-memory");
        println!();
        println!("🔗 Useful Links:");
        println!("  📚 Documentation: https://github.com/marlonsc/mcp-context-browser");
        println!("  🐙 GitHub: https://github.com/marlonsc/mcp-context-browser");
        println!("  🎯 Claude Context Reference: https://github.com/zilliztech/claude-context");
    }
}

/// Metrics API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Port for metrics HTTP API
    pub port: u16,
    /// Enable metrics collection
    pub enabled: bool,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limiting: RateLimitConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
            metrics: MetricsConfig::default(),
            auth: AuthConfig::default(),
            database: DatabaseConfig::default(),
            sync: SyncConfig::default(),
            daemon: DaemonConfig::default(),
            resource_limits: ResourceLimitsConfig::default(),
            cache: CacheConfig::default(),
            hybrid_search: HybridSearchConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            embedding: crate::core::types::EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                api_key: None,
                base_url: Some("http://localhost:11434".to_string()),
                dimensions: Some(768),
                max_tokens: Some(8192),
            },
            vector_store: crate::core::types::VectorStoreConfig {
                provider: "in-memory".to_string(),
                address: None,
                token: None,
                collection: None,
                dimensions: Some(768),
            },
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            enabled: true,
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration using ConfigManager (professional approach like Claude Context)
    pub fn from_env() -> Result<Self> {
        let manager = ConfigManager::new()?;
        manager.load_config()
    }

    /// Legacy method for backward compatibility - DEPRECATED
    /// Use ConfigManager::new()?.load_config() instead
    #[deprecated(
        note = "Use ConfigManager::new()?.load_config() for professional configuration management"
    )]
    pub fn from_env_legacy() -> Result<Self> {
        Ok(Self {
            name: std::env::var("MCP_NAME").unwrap_or_else(|_| "MCP Context Browser".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig {
                host: std::env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("MCP_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
            },
            providers: ProviderConfig::default(),
            metrics: MetricsConfig {
                port: std::env::var("CONTEXT_METRICS_PORT")
                    .unwrap_or_else(|_| "3001".to_string())
                    .parse()
                    .unwrap_or(3001),
                enabled: std::env::var("CONTEXT_METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                rate_limiting: RateLimitConfig::default(),
            },
            auth: AuthConfig::default(),
            database: DatabaseConfig::default(),
            sync: SyncConfig::from_env(),
            daemon: DaemonConfig::from_env(),
            resource_limits: ResourceLimitsConfig::default(),
            cache: CacheConfig::default(),
            hybrid_search: HybridSearchConfig::default(),
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Basic validation
        if self.name.is_empty() {
            return Err(Error::invalid_argument("Name cannot be empty"));
        }

        if self.version.is_empty() {
            return Err(Error::invalid_argument("Version cannot be empty"));
        }

        // Validate metrics port
        if self.metrics.port == 0 {
            return Err(Error::invalid_argument("Metrics port cannot be zero"));
        }

        // Validate sync configuration
        if self.sync.interval_ms == 0 {
            return Err(Error::invalid_argument("Sync interval cannot be zero"));
        }

        // Validate daemon configuration
        if self.daemon.cleanup_interval_secs == 0 || self.daemon.monitoring_interval_secs == 0 {
            return Err(Error::invalid_argument("Daemon intervals cannot be zero"));
        }

        Ok(())
    }

    /// Get metrics port
    pub fn metrics_port(&self) -> u16 {
        self.metrics.port
    }

    /// Check if metrics are enabled
    pub fn metrics_enabled(&self) -> bool {
        self.metrics.enabled
    }

    /// Get server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get metrics server address string
    pub fn metrics_addr(&self) -> String {
        format!("0.0.0.0:{}", self.metrics.port)
    }

    /// Print detailed configuration summary (like Claude Context)
    pub fn print_summary(&self) {
        println!("🔧 MCP Context Browser - Configuration Summary");
        println!("=============================================");
        println!("📡 Server: {}:{}", self.server.host, self.server.port);
        println!(
            "📊 Metrics: {} (enabled: {})",
            self.metrics_addr(),
            self.metrics.enabled
        );
        println!(
            "🔄 Sync: {}ms (lockfile: {})",
            self.sync.interval_ms, self.sync.enable_lockfile
        );
        println!(
            "🤖 Daemon: cleanup={}s, monitoring={}s",
            self.daemon.cleanup_interval_secs, self.daemon.monitoring_interval_secs
        );

        // Provider information (similar to Claude Context logging)
        println!();
        println!(
            "🧠 Embedding Provider: {}",
            self.providers.embedding.provider
        );
        println!("   Model: {}", self.providers.embedding.model);

        match self.providers.embedding.provider.as_str() {
            "openai" => {
                println!(
                    "   API Key: {}",
                    self.providers
                        .embedding
                        .api_key
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("❌ Missing")
                );
                if let Some(base_url) = &self.providers.embedding.base_url {
                    println!("   Base URL: {}", base_url);
                }
            }
            "ollama" => {
                println!(
                    "   Host: {}",
                    self.providers
                        .embedding
                        .base_url
                        .as_ref()
                        .unwrap_or(&"http://127.0.0.1:11434".to_string())
                );
            }
            "voyageai" => {
                println!(
                    "   API Key: {}",
                    self.providers
                        .embedding
                        .api_key
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("❌ Missing")
                );
            }
            "gemini" => {
                println!(
                    "   API Key: {}",
                    self.providers
                        .embedding
                        .api_key
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("❌ Missing")
                );
                if let Some(base_url) = &self.providers.embedding.base_url {
                    println!("   Base URL: {}", base_url);
                }
            }
            "mock" => println!("   Status: Development mode"),
            _ => println!("   Status: Unknown provider"),
        }

        if let Some(dim) = self.providers.embedding.dimensions {
            println!("   Dimensions: {}", dim);
        }
        if let Some(tokens) = self.providers.embedding.max_tokens {
            println!("   Max Tokens: {}", tokens);
        }

        println!();
        println!("🗄️  Vector Store: {}", self.providers.vector_store.provider);

        match self.providers.vector_store.provider.as_str() {
            "milvus" => {
                println!(
                    "   Address: {}",
                    self.providers
                        .vector_store
                        .address
                        .as_ref()
                        .unwrap_or(&"Not configured".to_string())
                );
                println!(
                    "   Token: {}",
                    self.providers
                        .vector_store
                        .token
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("❌ Missing")
                );
            }
            "pinecone" => {
                println!(
                    "   API Key: {}",
                    self.providers
                        .vector_store
                        .token
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("❌ Missing")
                );
            }
            "qdrant" => {
                println!(
                    "   URL: {}",
                    self.providers
                        .vector_store
                        .address
                        .as_ref()
                        .unwrap_or(&"Not configured".to_string())
                );
                println!(
                    "   API Key: {}",
                    self.providers
                        .vector_store
                        .token
                        .as_ref()
                        .map(|_| "✅ Configured")
                        .unwrap_or("Optional")
                );
            }
            "in-memory" => println!("   Status: Development mode"),
            _ => println!("   Status: Unknown provider"),
        }

        if let Some(collection) = &self.providers.vector_store.collection {
            println!("   Collection: {}", collection);
        }
        if let Some(dim) = self.providers.vector_store.dimensions {
            println!("   Dimensions: {}", dim);
        }

        println!();
        println!("✅ Configuration validated successfully");
    }
}

/// Builder pattern implementation for Config
impl Config {
    /// Create a new configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// Fluent configuration builder
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set server host
    pub fn server_host(mut self, host: String) -> Self {
        self.config.server.host = host;
        self
    }

    /// Set server port
    pub fn server_port(mut self, port: u16) -> Self {
        self.config.server.port = port;
        self
    }

    /// Set embedding provider configuration
    pub fn embedding_provider(mut self, provider: crate::core::types::EmbeddingConfig) -> Self {
        self.config.providers.embedding = provider;
        self
    }

    /// Set vector store provider configuration
    pub fn vector_store_provider(
        mut self,
        provider: crate::core::types::VectorStoreConfig,
    ) -> Self {
        self.config.providers.vector_store = provider;
        self
    }

    /// Set metrics port
    pub fn metrics_port(mut self, port: u16) -> Self {
        self.config.metrics.port = port;
        self
    }

    /// Set metrics enabled
    pub fn metrics_enabled(mut self, enabled: bool) -> Self {
        self.config.metrics.enabled = enabled;
        self
    }

    /// Set sync interval
    pub fn sync_interval_ms(mut self, interval: u64) -> Self {
        self.config.sync.interval_ms = interval;
        self
    }

    /// Set daemon cleanup interval
    pub fn daemon_cleanup_interval_secs(mut self, interval: u64) -> Self {
        self.config.daemon.cleanup_interval_secs = interval;
        self
    }

    /// Set application name
    pub fn name(mut self, name: String) -> Self {
        self.config.name = name;
        self
    }

    /// Set application version
    pub fn version(mut self, version: String) -> Self {
        self.config.version = version;
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<Config> {
        let config = self.config;

        // Validate the built configuration
        use crate::config::validation::ConfigValidator;
        let validator = ConfigValidator::new();
        validator.validate(&config)?;

        Ok(config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
