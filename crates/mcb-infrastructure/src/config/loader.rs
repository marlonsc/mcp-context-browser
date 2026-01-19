//! Configuration loader
//!
//! Handles loading configuration from various sources including
//! TOML files, environment variables, and default values.
//!
//! Uses Figment for configuration management (migrated from config crate in v0.1.2).

use crate::config::AppConfig;
use crate::constants::*;
use crate::error_ext::ErrorContext;
use crate::logging::log_config_loaded;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use mcb_domain::error::{Error, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Configuration loader service
#[derive(Clone)]
pub struct ConfigLoader {
    /// Configuration file path
    config_path: Option<PathBuf>,

    /// Environment prefix
    env_prefix: String,
}

impl ConfigLoader {
    /// Create a new configuration loader with default settings
    pub fn new() -> Self {
        Self {
            config_path: None,
            env_prefix: CONFIG_ENV_PREFIX.to_string(),
        }
    }

    /// Set the configuration file path
    pub fn with_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Load configuration from all sources
    ///
    /// Configuration sources are merged in this order (later sources override earlier):
    /// 1. Default values from `AppConfig::default()`
    /// 2. TOML configuration file (if exists)
    /// 3. Environment variables with prefix (e.g., `MCB_SERVER_PORT`)
    pub fn load(&self) -> Result<AppConfig> {
        // Start with default configuration
        let mut figment = Figment::new().merge(Serialized::defaults(AppConfig::default()));

        // Add configuration file if specified
        if let Some(config_path) = &self.config_path {
            if config_path.exists() {
                figment = figment.merge(Toml::file(config_path));
                log_config_loaded(config_path, true);
            } else {
                log_config_loaded(config_path, false);
            }
        } else {
            // Try to find default config file
            if let Some(default_path) = Self::find_default_config_path() {
                if default_path.exists() {
                    figment = figment.merge(Toml::file(&default_path));
                    log_config_loaded(&default_path, true);
                }
            }
        }

        // Add environment variables
        // Uses underscore as separator for nested keys (e.g., MCB_SERVER_PORT)
        figment = figment.merge(Env::prefixed(&format!("{}_", self.env_prefix)).split("_"));

        // Extract and deserialize configuration
        let app_config: AppConfig = figment
            .extract()
            .context("Failed to extract configuration")?;

        // Validate configuration
        self.validate_config(&app_config)?;

        Ok(app_config)
    }

    /// Reload configuration (useful for hot-reloading)
    pub fn reload(&self) -> Result<AppConfig> {
        self.load()
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, config: &AppConfig, path: P) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

        std::fs::write(path.as_ref(), toml_string).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the current configuration file path
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Find default configuration file paths to try
    fn find_default_config_path() -> Option<PathBuf> {
        let current_dir = env::current_dir().ok()?;

        // Try various common config file locations
        let candidates = vec![
            current_dir.join(DEFAULT_CONFIG_FILENAME),
            current_dir
                .join(DEFAULT_CONFIG_DIR)
                .join(DEFAULT_CONFIG_FILENAME),
            dirs::config_dir()
                .map(|d| d.join(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_FILENAME))
                .unwrap_or_default(),
            dirs::home_dir()
                .map(|d| {
                    d.join(format!(".{}", DEFAULT_CONFIG_DIR))
                        .join(DEFAULT_CONFIG_FILENAME)
                })
                .unwrap_or_default(),
        ];

        candidates.into_iter().find(|path| path.exists())
    }

    /// Validate configuration values
    fn validate_config(&self, config: &AppConfig) -> Result<()> {
        validate_app_config(config)
    }
}

/// Validate application configuration
///
/// Performs comprehensive validation of all configuration sections.
fn validate_app_config(config: &AppConfig) -> Result<()> {
    validate_server_config(config)?;
    validate_auth_config(config)?;
    validate_cache_config(config)?;
    validate_limits_config(config)?;
    validate_daemon_config(config)?;
    validate_backup_config(config)?;
    validate_operations_config(config)?;
    Ok(())
}

fn validate_server_config(config: &AppConfig) -> Result<()> {
    if config.server.network.port == 0 {
        return Err(Error::Configuration {
            message: "Server port cannot be 0".to_string(),
            source: None,
        });
    }
    if config.server.ssl.https
        && (config.server.ssl.ssl_cert_path.is_none() || config.server.ssl.ssl_key_path.is_none())
    {
        return Err(Error::Configuration {
            message: "SSL certificate and key paths are required when HTTPS is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_auth_config(config: &AppConfig) -> Result<()> {
    if config.auth.enabled {
        if config.auth.jwt.secret.is_empty() {
            return Err(Error::Configuration {
                message: "JWT secret cannot be empty when authentication is enabled".to_string(),
                source: None,
            });
        }
        if config.auth.jwt.secret.len() < 32 {
            return Err(Error::Configuration {
                message: "JWT secret should be at least 32 characters long".to_string(),
                source: None,
            });
        }
    }
    Ok(())
}

fn validate_cache_config(config: &AppConfig) -> Result<()> {
    if config.system.infrastructure.cache.enabled
        && config.system.infrastructure.cache.default_ttl_secs == 0
    {
        return Err(Error::Configuration {
            message: "Cache TTL cannot be 0 when cache is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_limits_config(config: &AppConfig) -> Result<()> {
    if config.system.infrastructure.limits.memory_limit == 0 {
        return Err(Error::Configuration {
            message: "Memory limit cannot be 0".to_string(),
            source: None,
        });
    }
    if config.system.infrastructure.limits.cpu_limit == 0 {
        return Err(Error::Configuration {
            message: "CPU limit cannot be 0".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_daemon_config(config: &AppConfig) -> Result<()> {
    if config.operations_daemon.daemon.enabled
        && config.operations_daemon.daemon.max_restart_attempts == 0
    {
        return Err(Error::Configuration {
            message: "Maximum restart attempts cannot be 0 when daemon is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_backup_config(config: &AppConfig) -> Result<()> {
    if config.system.data.backup.enabled && config.system.data.backup.interval_secs == 0 {
        return Err(Error::Configuration {
            message: "Backup interval cannot be 0 when backup is enabled".to_string(),
            source: None,
        });
    }
    Ok(())
}

fn validate_operations_config(config: &AppConfig) -> Result<()> {
    if config.operations_daemon.operations.tracking_enabled {
        if config.operations_daemon.operations.cleanup_interval_secs == 0 {
            return Err(Error::Configuration {
                message: "Operations cleanup interval cannot be 0 when tracking is enabled"
                    .to_string(),
                source: None,
            });
        }
        if config.operations_daemon.operations.retention_secs == 0 {
            return Err(Error::Configuration {
                message: "Operations retention period cannot be 0 when tracking is enabled"
                    .to_string(),
                source: None,
            });
        }
    }
    Ok(())
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration builder for programmatic configuration
pub struct ConfigBuilder {
    config: AppConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    /// Set server configuration
    pub fn with_server(mut self, server: crate::config::data::ServerConfig) -> Self {
        self.config.server = server;
        self
    }

    /// Set logging configuration
    pub fn with_logging(mut self, logging: crate::config::data::LoggingConfig) -> Self {
        self.config.logging = logging;
        self
    }

    /// Add embedding provider configuration
    pub fn with_embedding_provider(
        mut self,
        name: String,
        config: mcb_domain::value_objects::EmbeddingConfig,
    ) -> Self {
        self.config.providers.embedding.insert(name, config);
        self
    }

    /// Add vector store provider configuration
    pub fn with_vector_store_provider(
        mut self,
        name: String,
        config: mcb_domain::value_objects::VectorStoreConfig,
    ) -> Self {
        self.config.providers.vector_store.insert(name, config);
        self
    }

    /// Set authentication configuration
    pub fn with_auth(mut self, auth: crate::config::data::AuthConfig) -> Self {
        self.config.auth = auth;
        self
    }

    /// Set cache configuration
    pub fn with_cache(mut self, cache: crate::config::data::CacheConfig) -> Self {
        self.config.system.infrastructure.cache = cache;
        self
    }

    /// Build the configuration
    pub fn build(self) -> AppConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
