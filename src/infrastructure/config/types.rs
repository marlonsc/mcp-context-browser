// Architecture Note: Configuration aggregation requires importing from adapters layer.
// This is an acceptable deviation for the root config type that aggregates all settings.
// The alternative (duplicating config types) would violate DRY principle.
use crate::adapters::database::DatabaseConfig;
use crate::adapters::hybrid_search::HybridSearchConfig;
use crate::daemon::DaemonConfig;
use crate::infrastructure::auth::AuthConfig;
use crate::infrastructure::cache::CacheConfig;
use crate::infrastructure::limits::ResourceLimitsConfig;
use crate::sync::SyncConfig;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::metrics::MetricsConfig;
use super::providers::{EmbeddingProviderConfig, VectorStoreProviderConfig};
use super::server::ServerConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GlobalConfig {
    /// Server configuration
    #[serde(default)]
    #[validate(nested)]
    pub server: ServerConfig,
    /// Provider configurations
    #[validate(nested)]
    pub providers: GlobalProviderConfig,
    /// Metrics configuration
    #[serde(default)]
    #[validate(nested)]
    pub metrics: MetricsConfig,
    /// Sync configuration
    #[serde(default)]
    #[validate(nested)]
    pub sync: SyncConfig,
    /// Daemon configuration
    #[serde(default)]
    #[validate(nested)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GlobalProviderConfig {
    #[validate(nested)]
    pub embedding: EmbeddingProviderConfig,
    #[validate(nested)]
    pub vector_store: VectorStoreProviderConfig,
}

/// Legacy provider config (maintained for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProviderConfig {
    #[validate(nested)]
    pub embedding: crate::domain::types::EmbeddingConfig,
    #[validate(nested)]
    pub vector_store: crate::domain::types::VectorStoreConfig,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            embedding: crate::domain::types::EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                api_key: None,
                base_url: Some("http://localhost:11434".to_string()),
                dimensions: Some(768),
                max_tokens: Some(8192),
            },
            vector_store: crate::domain::types::VectorStoreConfig {
                provider: "in-memory".to_string(),
                address: None,
                token: None,
                collection: None,
                dimensions: Some(768),
                timeout_secs: None, // Default: 10 seconds (set in provider)
            },
        }
    }
}

/// Main application configuration
///
/// Central configuration structure containing all settings for the MCP Context Browser.
/// Supports hierarchical configuration with validation and environment variable overrides.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Application name
    #[serde(default = "default_name")]
    pub name: String,
    /// Application version
    #[serde(default = "default_version")]
    pub version: String,
    /// Server configuration (host, port, etc.)
    #[serde(default)]
    #[validate(nested)]
    pub server: ServerConfig,
    /// AI and vector store provider configurations
    #[serde(default)]
    #[validate(nested)]
    pub providers: ProviderConfig,
    /// Metrics and monitoring configuration
    #[serde(default)]
    #[validate(nested)]
    pub metrics: MetricsConfig,
    /// Admin interface configuration
    #[serde(default)]
    #[validate(nested)]
    pub admin: crate::admin::AdminConfig,
    /// Authentication and authorization settings
    #[serde(default)]
    #[validate(nested)]
    pub auth: AuthConfig,

    /// Database configuration
    #[serde(default)]
    #[validate(nested)]
    pub database: DatabaseConfig,
    /// Sync coordination configuration
    #[serde(default)]
    #[validate(nested)]
    pub sync: SyncConfig,
    /// Background daemon configuration
    #[serde(default)]
    #[validate(nested)]
    pub daemon: DaemonConfig,

    /// Resource limits configuration
    #[serde(default)]
    #[validate(nested)]
    pub resource_limits: ResourceLimitsConfig,

    /// Advanced caching configuration
    #[serde(default)]
    #[validate(nested)]
    pub cache: CacheConfig,
    #[serde(default)]
    #[validate(nested)]
    pub hybrid_search: HybridSearchConfig,
}

fn default_name() -> String {
    "MCP Context Browser".to_string()
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
            metrics: MetricsConfig::default(),
            admin: crate::admin::AdminConfig::default(),
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

impl Config {
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
}
