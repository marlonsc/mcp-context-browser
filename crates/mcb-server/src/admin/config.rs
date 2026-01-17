//! Admin Configuration API
//!
//! Runtime configuration management for the admin API.
//! Provides endpoints for viewing, reloading, and updating configuration.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/config` | GET | View current configuration (sanitized) |
//! | `/config/reload` | POST | Trigger configuration reload |
//! | `/config/:section` | PATCH | Update configuration section |
//!
//! ## Security
//!
//! Configuration responses are sanitized to remove sensitive fields like:
//! - JWT secrets
//! - API keys
//! - Database passwords
//! - Encryption keys

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use mcb_infrastructure::config::data::AppConfig;
use mcb_infrastructure::config::types::{CacheConfig, LimitsConfig, MetricsConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration response (sanitized for API output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Configuration data (sanitized)
    pub config: SanitizedConfig,
    /// Configuration file path
    pub config_path: Option<String>,
    /// Last reload timestamp (RFC 3339)
    pub last_reload: Option<String>,
}

/// Sanitized configuration for API responses
///
/// Contains only non-sensitive configuration values suitable for
/// display in admin interfaces.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SanitizedConfig {
    /// Server configuration section
    pub server: ServerConfigView,
    /// Embedding provider configurations
    pub embedding: HashMap<String, EmbeddingConfigView>,
    /// Vector store configurations
    pub vector_store: HashMap<String, VectorStoreConfigView>,
    /// Logging configuration
    pub logging: LoggingConfigView,
    /// Cache configuration
    pub cache: CacheConfigView,
    /// Metrics configuration
    pub metrics: MetricsConfigView,
    /// Limits configuration
    pub limits: LimitsConfigView,
}

impl SanitizedConfig {
    /// Create a sanitized config from AppConfig, removing sensitive fields
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            server: Self::server_view(config),
            embedding: Self::embedding_views(&config.providers.embedding),
            vector_store: Self::vector_store_views(&config.providers.vector_store),
            logging: Self::logging_view(config),
            cache: Self::cache_view(&config.system.infrastructure.cache),
            metrics: Self::metrics_view(&config.system.infrastructure.metrics),
            limits: Self::limits_view(&config.system.infrastructure.limits),
        }
    }

    fn server_view(config: &AppConfig) -> ServerConfigView {
        ServerConfigView {
            host: config.server.network.host.clone(),
            port: config.server.network.port,
            transport_mode: format!("{:?}", config.server.transport_mode),
            admin_port: config.server.network.admin_port,
            https: config.server.ssl.https,
        }
    }

    fn embedding_views(
        cfg: &HashMap<String, EmbeddingConfig>,
    ) -> HashMap<String, EmbeddingConfigView> {
        cfg.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    EmbeddingConfigView {
                        provider: format!("{:?}", v.provider),
                        model: v.model.clone(),
                        dimensions: v.dimensions,
                        has_api_key: v.api_key.is_some(),
                    },
                )
            })
            .collect()
    }

    fn vector_store_views(
        cfg: &HashMap<String, VectorStoreConfig>,
    ) -> HashMap<String, VectorStoreConfigView> {
        cfg.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    VectorStoreConfigView {
                        provider: format!("{:?}", v.provider),
                        dimensions: v.dimensions,
                        collection: v.collection.clone(),
                        has_address: v.address.is_some(),
                    },
                )
            })
            .collect()
    }

    fn logging_view(config: &AppConfig) -> LoggingConfigView {
        LoggingConfigView {
            level: config.logging.level.clone(),
            json_format: config.logging.json_format,
            file_output: config
                .logging
                .file_output
                .as_ref()
                .map(|p| p.display().to_string()),
        }
    }

    fn cache_view(c: &CacheConfig) -> CacheConfigView {
        CacheConfigView {
            enabled: c.enabled,
            provider: format!("{:?}", c.provider),
            default_ttl_secs: c.default_ttl_secs,
            max_size: c.max_size,
        }
    }

    fn metrics_view(m: &MetricsConfig) -> MetricsConfigView {
        MetricsConfigView {
            enabled: m.enabled,
            collection_interval_secs: m.collection_interval_secs,
        }
    }

    fn limits_view(l: &LimitsConfig) -> LimitsConfigView {
        LimitsConfigView {
            memory_limit: l.memory_limit,
            cpu_limit: l.cpu_limit,
            max_connections: l.max_connections,
        }
    }
}

/// Server configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfigView {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Transport mode (Stdio, Http, Hybrid)
    pub transport_mode: String,
    /// Admin API port
    pub admin_port: u16,
    /// HTTPS enabled
    pub https: bool,
}

/// Embedding provider configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfigView {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Embedding dimensions (if configured)
    pub dimensions: Option<usize>,
    /// Whether an API key is configured (not the key itself)
    pub has_api_key: bool,
}

/// Vector store configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfigView {
    /// Provider name
    pub provider: String,
    /// Vector dimensions (if configured)
    pub dimensions: Option<usize>,
    /// Collection name
    pub collection: Option<String>,
    /// Whether address is configured (for remote providers)
    pub has_address: bool,
}

/// Logging configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfigView {
    /// Log level
    pub level: String,
    /// JSON format enabled
    pub json_format: bool,
    /// File output path (if configured)
    pub file_output: Option<String>,
}

/// Cache configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfigView {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache provider name
    pub provider: String,
    /// Default TTL in seconds
    pub default_ttl_secs: u64,
    /// Maximum cache size
    pub max_size: usize,
}

/// Metrics configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsConfigView {
    /// Whether metrics are enabled
    pub enabled: bool,
    /// Collection interval in seconds
    pub collection_interval_secs: u64,
}

/// Limits configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitsConfigView {
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// CPU limit (number of cores)
    pub cpu_limit: usize,
    /// Maximum concurrent connections
    pub max_connections: u32,
}

/// Configuration reload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReloadResponse {
    /// Whether the reload was successful
    pub success: bool,
    /// Reload result message
    pub message: String,
    /// New configuration (sanitized, if reload succeeded)
    pub config: Option<SanitizedConfig>,
    /// Reload timestamp (RFC 3339)
    pub reloaded_at: Option<String>,
}

impl ConfigReloadResponse {
    /// Create a success response
    pub fn success(config: SanitizedConfig) -> Self {
        Self {
            success: true,
            message: "Configuration reloaded successfully".to_string(),
            config: Some(config),
            reloaded_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a failure response
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            config: None,
            reloaded_at: None,
        }
    }

    /// Create a response indicating the watcher is not available
    pub fn watcher_unavailable() -> Self {
        Self {
            success: false,
            message: "Configuration watcher is not enabled".to_string(),
            config: None,
            reloaded_at: None,
        }
    }
}

/// Configuration section update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSectionUpdateRequest {
    /// Section-specific configuration values to update
    pub values: serde_json::Value,
}

/// Configuration section update response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSectionUpdateResponse {
    /// Whether the update was successful
    pub success: bool,
    /// Update result message
    pub message: String,
    /// Section name that was updated
    pub section: String,
    /// Updated configuration (sanitized)
    pub config: Option<SanitizedConfig>,
    /// Update timestamp (RFC 3339)
    pub updated_at: Option<String>,
}

impl ConfigSectionUpdateResponse {
    /// Create a success response
    pub fn success(section: impl Into<String>, config: SanitizedConfig) -> Self {
        Self {
            success: true,
            message: "Configuration section updated successfully".to_string(),
            section: section.into(),
            config: Some(config),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a failure response
    pub fn failure(section: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            section: section.into(),
            config: None,
            updated_at: None,
        }
    }

    /// Create a response for invalid section
    pub fn invalid_section(section: impl Into<String>) -> Self {
        let section_name = section.into();
        Self {
            success: false,
            message: format!("Unknown configuration section: {}", section_name),
            section: section_name,
            config: None,
            updated_at: None,
        }
    }

    /// Create a response indicating the watcher is not available
    pub fn watcher_unavailable(section: impl Into<String>) -> Self {
        Self {
            success: false,
            message: "Configuration watcher is not enabled".to_string(),
            section: section.into(),
            config: None,
            updated_at: None,
        }
    }
}

/// Valid configuration sections for updates
pub const VALID_SECTIONS: &[&str] = &[
    "server",
    "logging",
    "cache",
    "metrics",
    "limits",
    "resilience",
];

/// Check if a section name is valid for updates
pub fn is_valid_section(section: &str) -> bool {
    VALID_SECTIONS.contains(&section)
}
