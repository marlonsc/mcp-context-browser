//! Environment variable loading and management module
//!
//! This module provides comprehensive environment variable handling
//! with priority-based configuration merging and validation.

use crate::config::{Config, DaemonConfig, MetricsConfig, ServerConfig, SyncConfig};
use crate::core::error::{Error, Result};
use std::collections::HashMap;
use std::env;

/// Environment variable loader with caching and validation
pub struct EnvironmentLoader {
    cache: HashMap<String, String>,
    prefix: String,
}

impl EnvironmentLoader {
    /// Create a new environment loader
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            prefix: "MCP_".to_string(),
        }
    }

    /// Create a loader with custom prefix
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            cache: HashMap::new(),
            prefix: prefix.to_string(),
        }
    }

    /// Check if the loader is ready for operations
    pub fn is_ready(&self) -> bool {
        true // Always ready
    }

    /// Load server configuration from environment variables
    pub fn load_server_config(&self) -> Result<ServerConfig> {
        let host = self
            .get_var("HOST")
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port_str = self.get_var("PORT");
        let port = if let Some(port_str) = port_str {
            port_str.parse().map_err(|_| {
                Error::config(format!(
                    "Invalid PORT value '{}': must be a valid port number",
                    port_str
                ))
            })?
        } else {
            3000
        };

        Ok(ServerConfig { host, port })
    }

    /// Load metrics configuration from environment variables
    pub fn load_metrics_config(&self) -> Result<MetricsConfig> {
        Ok(MetricsConfig {
            port: self
                .get_var("METRICS_PORT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(3001),
            enabled: self
                .get_var("METRICS_ENABLED")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            rate_limiting: Default::default(), // Would be loaded separately if needed
        })
    }

    /// Load sync configuration from environment variables
    pub fn load_sync_config(&self) -> Result<SyncConfig> {
        Ok(SyncConfig {
            interval_ms: self
                .get_var("SYNC_INTERVAL_MS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(900000), // 15 minutes default
            enable_lockfile: self
                .get_var("SYNC_ENABLE_LOCKFILE")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            debounce_ms: self
                .get_var("SYNC_DEBOUNCE_MS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(60000), // 60 seconds default
        })
    }

    /// Load daemon configuration from environment variables
    pub fn load_daemon_config(&self) -> Result<DaemonConfig> {
        Ok(DaemonConfig {
            cleanup_interval_secs: self
                .get_var("DAEMON_CLEANUP_INTERVAL_SECS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(30), // 30 seconds default
            monitoring_interval_secs: self
                .get_var("DAEMON_MONITORING_INTERVAL_SECS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(30), // 30 seconds default
            max_lock_age_secs: self
                .get_var("DAEMON_MAX_LOCK_AGE_SECS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(300), // 5 minutes default
        })
    }

    /// Merge environment variables into existing configuration
    pub fn merge_with_environment(&self, mut config: Config) -> Result<Config> {
        // Override server config
        if let Some(host) = self.get_var("HOST") {
            config.server.host = host;
        }
        if let Some(port_str) = self.get_var("PORT")
            && let Ok(port) = port_str.parse::<u16>()
        {
            config.server.port = port;
        }

        // Override metrics config
        if let Some(port_str) = self.get_var("METRICS_PORT") {
            let _ = port_str.parse().map(|port| config.metrics.port = port);
        }
        if let Some(enabled_str) = self.get_var("METRICS_ENABLED") {
            let _ = enabled_str
                .parse()
                .map(|enabled| config.metrics.enabled = enabled);
        }

        // Override sync config
        if let Some(interval_str) = self.get_var("SYNC_INTERVAL_MS") {
            let _ = interval_str
                .parse()
                .map(|interval| config.sync.interval_ms = interval);
        }
        if let Some(enable_str) = self.get_var("SYNC_ENABLE_LOCKFILE") {
            let _ = enable_str
                .parse()
                .map(|enable| config.sync.enable_lockfile = enable);
        }
        if let Some(debounce_str) = self.get_var("SYNC_DEBOUNCE_MS")
            && let Ok(debounce) = debounce_str.parse()
        {
            config.sync.debounce_ms = debounce;
        }

        // Override daemon config
        if let Some(cleanup_str) = self.get_var("DAEMON_CLEANUP_INTERVAL_SECS")
            && let Ok(cleanup) = cleanup_str.parse()
        {
            config.daemon.cleanup_interval_secs = cleanup;
        }
        if let Some(monitoring_str) = self.get_var("DAEMON_MONITORING_INTERVAL_SECS")
            && let Ok(monitoring) = monitoring_str.parse()
        {
            config.daemon.monitoring_interval_secs = monitoring;
        }
        if let Some(max_age_str) = self.get_var("DAEMON_MAX_LOCK_AGE_SECS") {
            if let Ok(max_age) = max_age_str.parse() {
                config.daemon.max_lock_age_secs = max_age;
            }
        }

        Ok(config)
    }

    /// Get environment variable with prefix resolution
    ///
    /// Searches for environment variables in the following priority order:
    /// 1. Cache + prefixed version (e.g., MCP_HOST)
    /// 2. Cache + legacy CONTEXT_ prefix (e.g., CONTEXT_HOST) - for backward compatibility
    /// 3. Cache + unprefixed version (e.g., HOST)
    /// 4. Environment variables (same order as cache)
    ///
    /// # Arguments
    /// * `key` - The base key name without prefix
    ///
    /// # Returns
    /// The value of the first matching environment variable or cache entry, or None if not found
    pub fn get_var(&self, key: &str) -> Option<String> {
        // Priority order: MCP_ prefix > CONTEXT_ prefix > unprefixed
        // This matches the expected precedence in tests

        // Try prefixed version first (highest priority)
        let prefixed_key = format!("{}{}", self.prefix, key);
        if let Some(value) = self.cache.get(&prefixed_key) {
            return Some(value.clone());
        }
        if let Ok(value) = env::var(&prefixed_key) {
            return Some(value);
        }

        // Try legacy CONTEXT_ prefix for backward compatibility
        let legacy_key = format!("CONTEXT_{}", key);
        if let Some(value) = self.cache.get(&legacy_key) {
            return Some(value.clone());
        }
        if let Ok(value) = env::var(&legacy_key) {
            return Some(value);
        }

        // Try unprefixed version (lowest priority)
        if let Some(value) = self.cache.get(key) {
            return Some(value.clone());
        }
        env::var(key).ok()
    }

    /// Set environment variable in cache (for testing)
    pub fn set_var(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
        unsafe {
            env::set_var(key, value);
        }
    }

    /// Remove environment variable from cache (for testing)
    pub fn remove_var(&mut self, key: &str) {
        self.cache.remove(key);
        unsafe {
            env::remove_var(key);
        }
    }

    /// Get all environment variables with the configured prefix
    pub fn get_prefixed_vars(&self) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(&self.prefix))
            .map(|(key, value)| (key[self.prefix.len()..].to_string(), value))
            .collect()
    }

    /// Get detailed information about environment variable resolution for a key
    ///
    /// Useful for debugging configuration issues and understanding which
    /// environment variables are being used.
    ///
    /// # Arguments
    /// * `key` - The base key name without prefix
    ///
    /// # Returns
    /// A tuple containing (found_value, source_description)
    pub fn get_var_info(&self, key: &str) -> (Option<String>, String) {
        // Check cache first
        let prefixed_key = format!("{}{}", self.prefix, key);
        if let Some(value) = self.cache.get(&prefixed_key) {
            return (
                Some(value.clone()),
                format!("cache (prefixed: {})", prefixed_key),
            );
        }
        if let Ok(value) = env::var(&prefixed_key) {
            return (
                Some(value),
                format!("environment (prefixed: {})", prefixed_key),
            );
        }

        let legacy_key = format!("CONTEXT_{}", key);
        if let Some(value) = self.cache.get(&legacy_key) {
            return (
                Some(value.clone()),
                format!("cache (legacy: {})", legacy_key),
            );
        }
        if let Ok(value) = env::var(&legacy_key) {
            return (Some(value), format!("environment (legacy: {})", legacy_key));
        }

        if let Some(value) = self.cache.get(key) {
            return (Some(value.clone()), format!("cache (unprefixed: {})", key));
        }
        if let Ok(value) = env::var(key) {
            return (Some(value), format!("environment (unprefixed: {})", key));
        }

        (None, "not found".to_string())
    }

    /// Validate environment variable configuration
    pub fn validate_environment(&self) -> Result<()> {
        // Validate server port
        if let Some(port_str) = self.get_var("PORT") {
            let port: u16 = port_str.parse().map_err(|_| {
                Error::config(format!(
                    "Invalid PORT value '{}': must be a valid port number between 1024 and 65535",
                    port_str
                ))
            })?;

            if !(1024..=65535).contains(&port) {
                return Err(Error::config(format!(
                    "Server port {} is out of valid range (1024-65535)",
                    port
                )));
            }
        }

        // Validate metrics port
        if let Some(metrics_port_str) = self.get_var("METRICS_PORT") {
            let metrics_port: u16 = metrics_port_str.parse().map_err(|_| {
                Error::config(format!(
                    "Invalid METRICS_PORT value '{}': must be a valid port number between 1024 and 65535",
                    metrics_port_str
                ))
            })?;

            if !(1024..=65535).contains(&metrics_port) {
                return Err(Error::config(format!(
                    "Metrics port {} is out of valid range (1024-65535)",
                    metrics_port
                )));
            }

            // Check for port conflicts if both are set
            if let Some(port_str) = self.get_var("PORT")
                && let Ok(port) = port_str.parse::<u16>()
                && port == metrics_port
            {
                return Err(Error::config(format!(
                    "Server port ({}) and metrics port ({}) cannot be the same",
                    port, metrics_port
                )));
            }
        }

        Ok(())
    }
}

impl Default for EnvironmentLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_loader_creation() {
        let loader = EnvironmentLoader::new();
        assert!(loader.is_ready());
    }

    #[test]
    fn test_environment_variable_loading() {
        let mut loader = EnvironmentLoader::new();

        // Set test variables
        loader.set_var("MCP_HOST", "127.0.0.1");
        loader.set_var("MCP_PORT", "8080");

        // Test loading server config
        let server_config = loader.load_server_config().unwrap();
        assert_eq!(server_config.host, "127.0.0.1");
        assert_eq!(server_config.port, 8080);

        // Clean up
        loader.remove_var("MCP_HOST");
        loader.remove_var("MCP_PORT");
    }

    #[test]
    fn test_environment_precedence() {
        let mut loader = EnvironmentLoader::new();

        // Clean up any existing environment variables first
        loader.remove_var("MCP_PORT");
        loader.remove_var("CONTEXT_PORT");
        loader.remove_var("PORT");

        // Set variables with different prefixes
        loader.set_var("MCP_PORT", "3000");
        loader.set_var("CONTEXT_PORT", "4000"); // Legacy prefix
        loader.set_var("PORT", "5000"); // No prefix

        // MCP_ prefix should take precedence over unprefixed
        assert_eq!(loader.get_var("PORT"), Some("3000".to_string()));

        // Clean up
        loader.remove_var("MCP_PORT");
        loader.remove_var("CONTEXT_PORT");
        loader.remove_var("PORT");
    }

    #[test]
    fn test_config_merging() {
        let mut loader = EnvironmentLoader::new();

        // Create base config
        let base_config = Config::default();

        // Set environment overrides
        loader.set_var("MCP_HOST", "localhost");
        loader.set_var("MCP_PORT", "9000");

        // Merge with environment
        let merged_config = loader.merge_with_environment(base_config).unwrap();

        assert_eq!(merged_config.server.host, "localhost");
        assert_eq!(merged_config.server.port, 9000);

        // Clean up
        loader.remove_var("MCP_HOST");
        loader.remove_var("MCP_PORT");
    }

    #[test]
    fn test_environment_validation() {
        let mut loader = EnvironmentLoader::new();

        // Valid configuration
        loader.set_var("MCP_PORT", "3000");
        loader.set_var("MCP_METRICS_PORT", "3001");
        assert!(loader.validate_environment().is_ok());

        // Invalid configuration (same ports)
        loader.set_var("MCP_METRICS_PORT", "3000");
        assert!(loader.validate_environment().is_err());

        // Invalid port range
        loader.set_var("MCP_PORT", "80");
        assert!(loader.validate_environment().is_err());

        // Invalid port value (non-numeric)
        loader.set_var("MCP_PORT", "not-a-number");
        assert!(loader.validate_environment().is_err());

        // Clean up
        loader.remove_var("MCP_PORT");
        loader.remove_var("MCP_METRICS_PORT");
    }
}
