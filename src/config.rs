//! Configuration management for MCP Context Browser v0.0.3
//!
//! Centralized configuration with environment variable support for:
//! - Metrics API, sync coordination, background daemon, and all v0.0.3 features

use crate::core::error::{Error, Result};
use crate::daemon::DaemonConfig;
use crate::sync::SyncConfig;
use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub server: ServerConfig,
    pub providers: ProviderConfig,
    /// Metrics API configuration (v0.0.3)
    pub metrics: MetricsConfig,
    /// Sync coordination configuration (v0.0.3)
    pub sync: SyncConfig,
    /// Background daemon configuration (v0.0.3)
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub embedding: crate::core::types::EmbeddingConfig,
    pub vector_store: crate::core::types::VectorStoreConfig,
}

/// Metrics API configuration (v0.0.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Port for metrics HTTP API
    pub port: u16,
    /// Enable metrics collection
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
            metrics: MetricsConfig::default(),
            sync: SyncConfig::default(),
            daemon: DaemonConfig::default(),
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
                provider: "mock".to_string(),
                model: "mock".to_string(),
                api_key: None,
                base_url: None,
                dimensions: Some(128),
                max_tokens: Some(512),
            },
            vector_store: crate::core::types::VectorStoreConfig {
                provider: "in-memory".to_string(),
                address: None,
                token: None,
                collection: None,
                dimensions: Some(128),
            },
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            enabled: true,
        }
    }
}

impl Config {
    /// Load configuration from environment variables (v0.0.3)
    pub fn from_env() -> Result<Self> {
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
            providers: ProviderConfig::default(), // Keep defaults for now
            metrics: MetricsConfig {
                port: std::env::var("CONTEXT_METRICS_PORT")
                    .unwrap_or_else(|_| "3001".to_string())
                    .parse()
                    .unwrap_or(3001),
                enabled: std::env::var("CONTEXT_METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            sync: SyncConfig::from_env(),
            daemon: DaemonConfig::from_env(),
        })
    }

    /// Validate configuration (v0.0.3)
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

    /// Get metrics port (v0.0.3)
    pub fn metrics_port(&self) -> u16 {
        self.metrics.port
    }

    /// Check if metrics are enabled (v0.0.3)
    pub fn metrics_enabled(&self) -> bool {
        self.metrics.enabled
    }

    /// Get server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get metrics server address string (v0.0.3)
    pub fn metrics_addr(&self) -> String {
        format!("0.0.0.0:{}", self.metrics.port)
    }

    /// Print configuration summary (v0.0.3)
    pub fn print_summary(&self) {
        println!("🔧 Configuration Summary:");
        println!("  📡 MCP Server: {}:{}", self.server.host, self.server.port);
        println!(
            "  📊 Metrics API: {} (enabled: {})",
            self.metrics_addr(),
            self.metrics.enabled
        );
        println!(
            "  🔄 Sync Interval: {}ms (lockfile: {})",
            self.sync.interval_ms, self.sync.enable_lockfile
        );
        println!(
            "  🤖 Daemon Cleanup: {}s, Monitoring: {}s",
            self.daemon.cleanup_interval_secs, self.daemon.monitoring_interval_secs
        );
    }
}
