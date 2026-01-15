//! Configuration loader
//!
//! Handles loading configuration from various sources including
//! TOML files, environment variables, and default values.

use crate::config::data::AppConfig;
use crate::constants::*;
use crate::error_ext::ErrorContext;
use crate::logging::log_config_loaded;
use mcb_domain::error::{Error, Result};
use config::{Config, ConfigError, Environment, File};
use std::path::{Path, PathBuf};
use std::env;

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
    pub fn load(&self) -> Result<AppConfig> {
        let mut builder = Config::builder();

        // Start with default configuration
        builder = builder.add_source(config::File::from_str(
            &toml::to_string(&AppConfig::default())
                .context("Failed to serialize default config")?,
            config::FileFormat::Toml,
        ));

        // Add configuration file if specified
        if let Some(config_path) = &self.config_path {
            if config_path.exists() {
                builder = builder.add_source(File::from(config_path.as_path()));
                log_config_loaded(config_path, true);
            } else {
                log_config_loaded(config_path, false);
            }
        } else {
            // Try to find default config file
            if let Some(default_path) = Self::find_default_config_path() {
                if default_path.exists() {
                    builder = builder.add_source(File::from(default_path.as_path()));
                    log_config_loaded(&default_path, true);
                }
            }
        }

        // Add environment variables
        builder = builder.add_source(
            Environment::with_prefix(&self.env_prefix)
                .prefix_separator("_")
                .separator("_")
                .try_parsing(true)
                .list_separator(" ")
                .with_list_parse_key("server.cors_origins"),
        );

        // Build and deserialize configuration
        let config = builder
            .build()
            .context("Failed to build configuration")?;

        let app_config: AppConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

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
        let toml_string = toml::to_string_pretty(config)
            .context("Failed to serialize config to TOML")?;

        std::fs::write(path.as_ref(), toml_string)
            .context("Failed to write config file")?;

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
            current_dir.join(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_FILENAME),
            dirs::config_dir()
                .map(|d| d.join(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_FILENAME))
                .unwrap_or_default(),
            dirs::home_dir()
                .map(|d| d.join(format!(".{}", DEFAULT_CONFIG_DIR)).join(DEFAULT_CONFIG_FILENAME))
                .unwrap_or_default(),
        ];

        candidates.into_iter().find(|path| path.exists())
    }

    /// Validate configuration values
    fn validate_config(&self, config: &AppConfig) -> Result<()> {
        // Validate server configuration
        if config.server.port == 0 {
            return Err(Error::Configuration {
                message: "Server port cannot be 0".to_string(),
                source: None,
            });
        }

        if config.server.https {
            if config.server.ssl_cert_path.is_none() || config.server.ssl_key_path.is_none() {
                return Err(Error::Configuration {
                    message: "SSL certificate and key paths are required when HTTPS is enabled".to_string(),
                    source: None,
                });
            }
        }

        // Validate auth configuration
        if config.auth.enabled {
            if config.auth.jwt_secret.is_empty() {
                return Err(Error::Configuration {
                    message: "JWT secret cannot be empty when authentication is enabled".to_string(),
                    source: None,
                });
            }

            if config.auth.jwt_secret.len() < 32 {
                return Err(Error::Configuration {
                    message: "JWT secret should be at least 32 characters long".to_string(),
                    source: None,
                });
            }
        }

        // Validate cache configuration
        if config.cache.enabled && config.cache.default_ttl_secs == 0 {
            return Err(Error::Configuration {
                message: "Cache TTL cannot be 0 when cache is enabled".to_string(),
                source: None,
            });
        }

        // Validate limits
        if config.limits.memory_limit == 0 {
            return Err(Error::Configuration {
                message: "Memory limit cannot be 0".to_string(),
                source: None,
            });
        }

        if config.limits.cpu_limit == 0 {
            return Err(Error::Configuration {
                message: "CPU limit cannot be 0".to_string(),
                source: None,
            });
        }

        // Validate daemon configuration
        if config.daemon.enabled && config.daemon.max_restart_attempts == 0 {
            return Err(Error::Configuration {
                message: "Maximum restart attempts cannot be 0 when daemon is enabled".to_string(),
                source: None,
            });
        }

        // Validate backup configuration
        if config.backup.enabled && config.backup.interval_secs == 0 {
            return Err(Error::Configuration {
                message: "Backup interval cannot be 0 when backup is enabled".to_string(),
                source: None,
            });
        }

        // Validate operations configuration
        if config.operations.tracking_enabled {
            if config.operations.cleanup_interval_secs == 0 {
                return Err(Error::Configuration {
                    message: "Operations cleanup interval cannot be 0 when tracking is enabled".to_string(),
                    source: None,
                });
            }

            if config.operations.retention_secs == 0 {
                return Err(Error::Configuration {
                    message: "Operations retention period cannot be 0 when tracking is enabled".to_string(),
                    source: None,
                });
            }
        }

        Ok(())
    }
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
        self.config.embedding.insert(name, config);
        self
    }

    /// Add vector store provider configuration
    pub fn with_vector_store_provider(
        mut self,
        name: String,
        config: mcb_domain::value_objects::VectorStoreConfig,
    ) -> Self {
        self.config.vector_store.insert(name, config);
        self
    }

    /// Set authentication configuration
    pub fn with_auth(mut self, auth: crate::config::data::AuthConfig) -> Self {
        self.config.auth = auth;
        self
    }

    /// Set cache configuration
    pub fn with_cache(mut self, cache: crate::config::data::CacheConfig) -> Self {
        self.config.cache = cache;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::new();
        let config = loader.load().unwrap();

        assert_eq!(config.server.port, DEFAULT_HTTP_PORT);
        assert_eq!(config.logging.level, DEFAULT_LOG_LEVEL);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_server(crate::config::data::ServerConfig {
                port: 9090,
                ..Default::default()
            })
            .build();

        assert_eq!(config.server.port, 9090);
    }

    #[test]
    fn test_config_validation() {
        let loader = ConfigLoader::new();

        // Test invalid server port
        let invalid_config = AppConfig {
            server: crate::config::data::ServerConfig {
                port: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(loader.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let loader = ConfigLoader::new();
        let original_config = ConfigBuilder::new()
            .with_server(crate::config::data::ServerConfig {
                port: 9999,
                ..Default::default()
            })
            .build();

        // Save config
        loader.save_to_file(&original_config, &config_path).unwrap();

        // Load config
        let loaded_config = ConfigLoader::new()
            .with_config_path(&config_path)
            .load()
            .unwrap();

        assert_eq!(loaded_config.server.port, 9999);
    }
}