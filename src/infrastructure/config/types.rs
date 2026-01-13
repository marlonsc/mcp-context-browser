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

use super::data::DataConfig;
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct ProviderConfig {
    #[validate(nested)]
    pub embedding: crate::domain::types::EmbeddingConfig,
    #[validate(nested)]
    pub vector_store: crate::domain::types::VectorStoreConfig,
}

/// Main application configuration
///
/// Central configuration structure containing all settings for the MCP Context Browser.
/// Supports hierarchical configuration with validation and environment variable overrides.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Server configuration (host, port, etc.)
    #[validate(nested)]
    pub server: ServerConfig,
    /// AI and vector store provider configurations
    #[validate(nested)]
    pub providers: ProviderConfig,
    /// Metrics and monitoring configuration
    #[validate(nested)]
    pub metrics: MetricsConfig,
    /// Admin interface configuration (optional - only if credentials provided)
    pub admin: Option<crate::admin::AdminConfig>,
    /// Authentication and authorization settings
    #[validate(nested)]
    pub auth: AuthConfig,

    /// Database configuration
    #[validate(nested)]
    pub database: DatabaseConfig,
    /// Sync coordination configuration
    #[validate(nested)]
    pub sync: SyncConfig,
    /// Background daemon configuration
    #[validate(nested)]
    pub daemon: DaemonConfig,

    /// Resource limits configuration
    #[validate(nested)]
    pub resource_limits: ResourceLimitsConfig,

    /// Advanced caching configuration
    pub cache: CacheConfig,
    #[validate(nested)]
    pub hybrid_search: HybridSearchConfig,

    /// Data directory configuration (XDG standard locations)
    #[validate(nested)]
    pub data: DataConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
            metrics: MetricsConfig::default(),
            admin: None,
            auth: AuthConfig::default(),
            database: DatabaseConfig::default(),
            sync: SyncConfig::default(),
            daemon: DaemonConfig::default(),
            resource_limits: ResourceLimitsConfig::default(),
            cache: CacheConfig::default(),
            hybrid_search: HybridSearchConfig::default(),
            data: DataConfig::default(),
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
