//! Server configuration utilities
//!
//! Utilities for managing HTTP server configuration and settings.

use crate::config::data::*;
use crate::constants::*;
use mcb_domain::error::{Error, Result};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Server configuration utilities
pub struct ServerConfigUtils;

impl ServerConfigUtils {
    /// Parse server address from configuration
    pub fn parse_address(config: &ServerConfig) -> Result<SocketAddr> {
        let ip: IpAddr = config.network.host.parse().map_err(|_| Error::Configuration {
            message: format!("Invalid server host: {}", config.network.host),
            source: None,
        })?;

        Ok(SocketAddr::new(ip, config.network.port))
    }

    /// Get the server URL for the given configuration
    pub fn get_server_url(config: &ServerConfig) -> String {
        let protocol = if config.ssl.https { "https" } else { "http" };
        // Use host:port directly - works for both IP addresses and domain names
        format!("{}://{}:{}", protocol, config.network.host, config.network.port)
    }

    /// Validate SSL configuration
    pub fn validate_ssl_config(config: &ServerConfig) -> Result<()> {
        if !config.ssl.https {
            return Ok(());
        }

        if config.ssl.ssl_cert_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL certificate path is required when HTTPS is enabled".to_string(),
                source: None,
            });
        }

        if config.ssl.ssl_key_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL key path is required when HTTPS is enabled".to_string(),
                source: None,
            });
        }

        // Both paths validated above - use pattern match to avoid unwrap
        if let (Some(cert_path), Some(key_path)) = (&config.ssl.ssl_cert_path, &config.ssl.ssl_key_path) {
            if !cert_path.exists() {
                return Err(Error::Configuration {
                    message: format!(
                        "SSL certificate file does not exist: {}",
                        cert_path.display()
                    ),
                    source: None,
                });
            }

            if !key_path.exists() {
                return Err(Error::Configuration {
                    message: format!("SSL key file does not exist: {}", key_path.display()),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Get request timeout duration
    pub fn request_timeout(config: &ServerConfig) -> Duration {
        Duration::from_secs(config.timeouts.request_timeout_secs)
    }

    /// Get connection timeout duration
    pub fn connection_timeout(config: &ServerConfig) -> Duration {
        Duration::from_secs(config.timeouts.connection_timeout_secs)
    }

    /// Check if CORS is enabled and get allowed origins
    pub fn cors_settings(config: &ServerConfig) -> (bool, Vec<String>) {
        (config.cors.cors_enabled, config.cors.cors_origins.clone())
    }

    /// Get the maximum request body size in bytes
    pub fn max_request_body_size(config: &ServerConfig) -> usize {
        config.timeouts.max_request_body_size
    }
}

/// Server configuration builder
#[derive(Clone)]
pub struct ServerConfigBuilder {
    config: ServerConfig,
}

impl ServerConfigBuilder {
    /// Create a new server config builder with defaults
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    /// Set the server host
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.config.network.host = host.into();
        self
    }

    /// Set the server port
    pub fn port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Enable HTTPS
    pub fn https(mut self, enabled: bool) -> Self {
        self.config.ssl.https = enabled;
        self
    }

    /// Set SSL certificate and key paths
    pub fn ssl_paths<P: Into<std::path::PathBuf>>(mut self, cert_path: P, key_path: P) -> Self {
        self.config.ssl.ssl_cert_path = Some(cert_path.into());
        self.config.ssl.ssl_key_path = Some(key_path.into());
        self
    }

    /// Set request timeout in seconds
    pub fn request_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.request_timeout_secs = seconds;
        self
    }

    /// Set connection timeout in seconds
    pub fn connection_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.connection_timeout_secs = seconds;
        self
    }

    /// Set maximum request body size
    pub fn max_request_body_size(mut self, size: usize) -> Self {
        self.config.timeouts.max_request_body_size = size;
        self
    }

    /// Configure CORS
    pub fn cors(mut self, enabled: bool, origins: Vec<String>) -> Self {
        self.config.cors.cors_enabled = enabled;
        self.config.cors.cors_origins = origins;
        self
    }

    /// Build the server configuration
    pub fn build(self) -> ServerConfig {
        self.config
    }
}

impl Default for ServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Server configuration presets
pub struct ServerConfigPresets;

impl ServerConfigPresets {
    /// Development server configuration
    pub fn development() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("127.0.0.1")
            .port(8080)
            .https(false)
            .request_timeout(60)
            .connection_timeout(10)
            .cors(
                true,
                vec!["http://localhost:3000".to_string(), "*".to_string()],
            )
            .build()
    }

    /// Production server configuration
    pub fn production() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("0.0.0.0")
            .port(DEFAULT_HTTPS_PORT)
            .https(true)
            .request_timeout(30)
            .connection_timeout(5)
            .cors(true, vec!["https://yourdomain.com".to_string()])
            .build()
    }

    /// Testing server configuration
    pub fn testing() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("127.0.0.1")
            .port(0) // Use random available port
            .https(false)
            .request_timeout(5)
            .connection_timeout(2)
            .cors(false, vec![])
            .build()
    }
}
