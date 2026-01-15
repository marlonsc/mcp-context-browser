//! Configuration types and utilities
//!
//! Additional types and utilities for configuration management.

use crate::config::data::*;
use mcb_domain::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration source priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigSource {
    /// Default values (lowest priority)
    Default = 0,
    /// Configuration file
    File = 1,
    /// Environment variables
    Environment = 2,
    /// Runtime overrides (highest priority)
    Runtime = 3,
}

/// Configuration update event
#[derive(Debug, Clone)]
pub enum ConfigUpdateEvent {
    /// Configuration reloaded from file
    Reloaded {
        /// Path to the configuration file
        path: std::path::PathBuf,
        /// Timestamp of the reload
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Configuration updated programmatically
    Updated {
        /// Keys that were updated
        keys: Vec<String>,
        /// Timestamp of the update
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add a validation error
    pub fn with_error<S: Into<String>>(mut self, error: S) -> Self {
        self.errors.push(error.into());
        self.is_valid = false;
        self
    }

    /// Add a validation warning
    pub fn with_warning<S: Into<String>>(mut self, warning: S) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Configuration key path utilities
pub struct ConfigKey;

impl ConfigKey {
    /// Join configuration keys with dots
    pub fn join(keys: &[&str]) -> String {
        keys.join(".")
    }

    /// Split a configuration key path into components
    pub fn split(key: &str) -> Vec<String> {
        key.split('.').map(|s| s.to_string()).collect()
    }

    /// Check if a key is under a specific prefix
    pub fn is_under_prefix(key: &str, prefix: &str) -> bool {
        key.starts_with(prefix)
    }

    /// Get the parent key path
    pub fn parent(key: &str) -> Option<String> {
        key.rsplit_once('.').map(|(parent, _)| parent.to_string())
    }

    /// Get the last component of a key path
    pub fn last_component(key: &str) -> String {
        key.rsplit('.').next().unwrap_or(key).to_string()
    }
}

/// Configuration environment utilities
pub struct ConfigEnv;

impl ConfigEnv {
    /// Get a configuration value from environment variables
    pub fn get_var(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Get a configuration value with a default
    pub fn get_var_or(key: &str, default: &str) -> String {
        Self::get_var(key).unwrap_or_else(|| default.to_string())
    }

    /// Get a configuration value as a boolean
    pub fn get_bool(key: &str) -> Option<bool> {
        Self::get_var(key)?.parse().ok()
    }

    /// Get a configuration value as an integer
    pub fn get_int<T: std::str::FromStr>(key: &str) -> Option<T> {
        Self::get_var(key)?.parse().ok()
    }

    /// Check if running in development mode
    pub fn is_development() -> bool {
        Self::get_var("RUST_ENV")
            .map(|env| env == "development")
            .unwrap_or(true)
    }

    /// Check if running in production mode
    pub fn is_production() -> bool {
        Self::get_var("RUST_ENV")
            .map(|env| env == "production")
            .unwrap_or(false)
    }

    /// Check if running in test mode
    pub fn is_test() -> bool {
        Self::get_var("RUST_ENV")
            .map(|env| env == "test")
            .unwrap_or_else(|| cfg!(test))
    }
}

/// Configuration profile utilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigProfile {
    /// Development profile
    Development,
    /// Production profile
    Production,
    /// Testing profile
    Testing,
    /// Custom profile
    Custom(&'static str),
}

impl ConfigProfile {
    /// Get the current profile from environment
    pub fn current() -> Self {
        if ConfigEnv::is_production() {
            Self::Production
        } else if ConfigEnv::is_test() {
            Self::Testing
        } else {
            Self::Development
        }
    }

    /// Get profile-specific configuration overrides
    pub fn overrides(&self) -> HashMap<String, serde_json::Value> {
        let mut overrides = HashMap::new();

        match self {
            Self::Development => {
                overrides.insert(
                    "logging.level".to_string(),
                    serde_json::Value::String("debug".to_string()),
                );
                overrides.insert(
                    "server.port".to_string(),
                    serde_json::Value::Number(8080.into()),
                );
            }
            Self::Production => {
                overrides.insert(
                    "logging.level".to_string(),
                    serde_json::Value::String("info".to_string()),
                );
                overrides.insert(
                    "logging.json_format".to_string(),
                    serde_json::Value::Bool(true),
                );
                overrides.insert(
                    "metrics.enabled".to_string(),
                    serde_json::Value::Bool(true),
                );
            }
            Self::Testing => {
                overrides.insert(
                    "logging.level".to_string(),
                    serde_json::Value::String("warn".to_string()),
                );
                overrides.insert(
                    "cache.enabled".to_string(),
                    serde_json::Value::Bool(false),
                );
            }
            Self::Custom(_) => {
                // No default overrides for custom profiles
            }
        }

        overrides
    }
}

/// Configuration migration utilities
pub struct ConfigMigration;

impl ConfigMigration {
    /// Migrate configuration from an older version
    pub fn migrate_from_version(config: &mut AppConfig, from_version: &str) -> Result<()> {
        match from_version {
            "0.1.0" => {
                // Add new fields that were introduced in 0.1.1
                // For now, we just ensure defaults are set
                let default_config = AppConfig::default();

                // Migrate server config
                if config.server.cors_enabled == false && config.server.cors_origins.is_empty() {
                    config.server.cors_origins = default_config.server.cors_origins;
                }
            }
            _ => {
                // No migration needed for unknown versions
            }
        }

        Ok(())
    }

    /// Check if configuration migration is needed
    pub fn needs_migration(_config: &AppConfig, _target_version: &str) -> bool {
        // For now, always return false as we don't have version-specific migrations
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_key_utilities() {
        assert_eq!(ConfigKey::join(&["server", "port"]), "server.port");
        assert_eq!(ConfigKey::split("server.port"), vec!["server", "port"]);
        assert!(ConfigKey::is_under_prefix("server.port", "server"));
        assert_eq!(ConfigKey::parent("server.port"), Some("server".to_string()));
        assert_eq!(ConfigKey::last_component("server.port"), "port");
    }

    #[test]
    fn test_validation_result() {
        let valid = ValidationResult::valid();
        assert!(valid.is_valid);
        assert!(!valid.has_errors());

        let invalid = ValidationResult::invalid(vec!["error1".to_string()])
            .with_warning("warning1");
        assert!(!invalid.is_valid);
        assert!(invalid.has_errors());
        assert!(invalid.has_warnings());
    }

    #[test]
    fn test_config_profile() {
        let dev_profile = ConfigProfile::Development;
        let overrides = dev_profile.overrides();
        assert!(overrides.contains_key("logging.level"));

        let prod_profile = ConfigProfile::Production;
        let prod_overrides = prod_profile.overrides();
        assert_eq!(
            prod_overrides.get("logging.level").unwrap(),
            &serde_json::Value::String("info".to_string())
        );
    }
}