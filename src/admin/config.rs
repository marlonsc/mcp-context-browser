//! Admin configuration and API types
//!
//! ## Security-First Design
//!
//! AdminConfig REQUIRES explicit configuration - NO hardcoded defaults.
//! Must be loaded from environment variables or config file.
//!
//! Required environment variables:
//! - `MCP_ADMIN_USERNAME` - Admin account username
//! - `MCP_ADMIN_PASSWORD` - Admin account password (min 8 chars)
//! - `MCP_ADMIN_JWT_SECRET` - JWT signing secret (min 32 chars)
//!
//! Optional:
//! - `MCP_ADMIN_JWT_EXPIRATION` - Expiration in seconds (default: 3600)
//! - `MCP_ADMIN_ENABLED` - Enable/disable (default: true if credentials provided)

use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;

pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MIN_JWT_SECRET_LENGTH: usize = 32;

/// Insecure default values that MUST NOT be used in production
const INSECURE_PASSWORDS: &[&str] = &["admin", "password", "123456", ""];
const INSECURE_JWT_SECRETS: &[&str] = &[
    "default-jwt-secret-change-in-production",
    "secret",
    "jwt-secret",
    "",
];

/// Admin API server configuration
///
/// Can be loaded from:
/// 1. Environment variables (preferred for production)
/// 2. TOML config file [admin] section (for local dev with env vars)
/// 3. Explicit constructor (for testing)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdminConfig {
    /// Enable admin interface (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Admin username
    #[serde(default)]
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    /// Admin password (min 8 chars in production)
    #[serde(default)]
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
    /// JWT secret for authentication (min 32 chars in production)
    #[serde(default)]
    #[validate(length(min = 1, message = "JWT secret cannot be empty"))]
    pub jwt_secret: String,
    /// JWT expiration time in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_jwt_expiration")]
    #[validate(range(min = 1))]
    pub jwt_expiration: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_jwt_expiration() -> u64 {
    3600 // 1 hour
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    #[error("Invalid JWT secret: {0}")]
    InvalidJwtSecret(String),

    #[error("Insecure configuration: {0}")]
    InsecureConfig(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl AdminConfig {
    /// Create with explicit values (validates immediately)
    pub fn new(
        enabled: bool,
        username: String,
        password: String,
        jwt_secret: String,
        jwt_expiration: u64,
    ) -> Result<Self, ConfigError> {
        let config = Self {
            enabled,
            username,
            password,
            jwt_secret,
            jwt_expiration,
        };
        config
            .validate()
            .map_err(|e| ConfigError::ConfigError(format!("Validation failed: {}", e)))?;
        Ok(config)
    }

    /// Load configuration from environment variables
    ///
    /// Required when admin is enabled:
    /// - `MCP_ADMIN_USERNAME`
    /// - `MCP_ADMIN_PASSWORD`
    /// - `MCP_ADMIN_JWT_SECRET`
    ///
    /// Optional:
    /// - `MCP_ADMIN_ENABLED` (default: true)
    /// - `MCP_ADMIN_JWT_EXPIRATION` (default: 3600)
    pub fn from_env() -> Result<Self, ConfigError> {
        let enabled = std::env::var("MCP_ADMIN_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        if !enabled {
            // Admin disabled, return minimal config
            return Ok(Self {
                enabled: false,
                username: String::new(),
                password: String::new(),
                jwt_secret: String::new(),
                jwt_expiration: 3600,
            });
        }

        // Admin enabled - credentials required
        let username = std::env::var("MCP_ADMIN_USERNAME")
            .map_err(|_| ConfigError::MissingEnvVar("MCP_ADMIN_USERNAME".to_string()))?;

        let password = std::env::var("MCP_ADMIN_PASSWORD")
            .map_err(|_| ConfigError::MissingEnvVar("MCP_ADMIN_PASSWORD".to_string()))?;

        let jwt_secret = std::env::var("MCP_ADMIN_JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("MCP_ADMIN_JWT_SECRET".to_string()))?;

        let jwt_expiration = std::env::var("MCP_ADMIN_JWT_EXPIRATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        let config = Self {
            enabled,
            username,
            password,
            jwt_secret,
            jwt_expiration,
        };

        // Validate for production use
        config.validate_for_production()?;

        Ok(config)
    }

    /// Merge environment variables over TOML config
    ///
    /// Environment variables take precedence over TOML values.
    /// Use this when loading from config file but want env override.
    pub fn with_env_override(mut self) -> Result<Self, ConfigError> {
        // Override with env vars if present
        if let Ok(enabled) = std::env::var("MCP_ADMIN_ENABLED") {
            self.enabled = enabled.to_lowercase() != "false" && enabled != "0";
        }

        if let Ok(username) = std::env::var("MCP_ADMIN_USERNAME") {
            self.username = username;
        }

        if let Ok(password) = std::env::var("MCP_ADMIN_PASSWORD") {
            self.password = password;
        }

        if let Ok(jwt_secret) = std::env::var("MCP_ADMIN_JWT_SECRET") {
            self.jwt_secret = jwt_secret;
        }

        if let Ok(jwt_expiration) = std::env::var("MCP_ADMIN_JWT_EXPIRATION") {
            if let Ok(exp) = jwt_expiration.parse() {
                self.jwt_expiration = exp;
            }
        }

        // Validate if admin is enabled
        if self.enabled {
            self.validate_for_production()?;
        }

        Ok(self)
    }

    /// Validate configuration for production use
    ///
    /// Returns error if:
    /// - Using insecure default password
    /// - Using insecure default JWT secret
    /// - Password too short
    /// - JWT secret too short
    pub fn validate_for_production(&self) -> Result<(), ConfigError> {
        if !self.enabled {
            return Ok(());
        }

        // Check for insecure passwords
        if INSECURE_PASSWORDS.contains(&self.password.as_str()) {
            return Err(ConfigError::InsecureConfig(
                "Password is insecure. Set MCP_ADMIN_PASSWORD environment variable with a strong password (min 8 chars)".to_string()
            ));
        }

        // Check password length
        if self.password.len() < MIN_PASSWORD_LENGTH {
            return Err(ConfigError::InvalidPassword(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
        }

        // Check for insecure JWT secrets
        if INSECURE_JWT_SECRETS.contains(&self.jwt_secret.as_str()) {
            return Err(ConfigError::InsecureConfig(
                "JWT secret is insecure. Set MCP_ADMIN_JWT_SECRET environment variable with a secure secret (min 32 chars)".to_string()
            ));
        }

        // Check JWT secret length
        if self.jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(ConfigError::InvalidJwtSecret(format!(
                "JWT secret must be at least {} characters",
                MIN_JWT_SECRET_LENGTH
            )));
        }

        // Check username not empty
        if self.username.is_empty() {
            return Err(ConfigError::InvalidUsername(
                "Username cannot be empty. Set MCP_ADMIN_USERNAME environment variable".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if configuration uses insecure defaults
    ///
    /// Returns list of security warnings (does not fail)
    pub fn security_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if !self.enabled {
            return warnings;
        }

        if INSECURE_PASSWORDS.contains(&self.password.as_str()) {
            warnings.push(
                "CRITICAL: Using insecure default password. Set MCP_ADMIN_PASSWORD env var."
                    .to_string(),
            );
        } else if self.password.len() < MIN_PASSWORD_LENGTH {
            warnings.push(format!(
                "WARNING: Password is short ({} chars). Recommended: {} chars minimum.",
                self.password.len(),
                MIN_PASSWORD_LENGTH
            ));
        }

        if INSECURE_JWT_SECRETS.contains(&self.jwt_secret.as_str()) {
            warnings.push(
                "CRITICAL: Using insecure default JWT secret. Set MCP_ADMIN_JWT_SECRET env var."
                    .to_string(),
            );
        } else if self.jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            warnings.push(format!(
                "WARNING: JWT secret is short ({} chars). Recommended: {} chars minimum.",
                self.jwt_secret.len(),
                MIN_JWT_SECRET_LENGTH
            ));
        }

        warnings
    }

    /// Log security warnings at startup
    pub fn log_security_warnings(&self) {
        for warning in self.security_warnings() {
            if warning.starts_with("CRITICAL") {
                tracing::error!("[SECURITY] {}", warning);
            } else {
                tracing::warn!("[SECURITY] {}", warning);
            }
        }
    }

    /// Create for testing ONLY - bypasses production validation
    #[cfg(test)]
    pub fn for_testing(
        username: &str,
        password: &str,
        jwt_secret: &str,
    ) -> Result<Self, ConfigError> {
        let config = Self {
            enabled: true,
            username: username.to_string(),
            password: password.to_string(),
            jwt_secret: jwt_secret.to_string(),
            jwt_expiration: 3600,
        };
        config
            .validate()
            .map_err(|e| ConfigError::ConfigError(format!("Validation failed: {}", e)))?;
        Ok(config)
    }

    /// Create disabled config (for when admin is turned off)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            username: String::new(),
            password: String::new(),
            jwt_secret: String::new(),
            jwt_expiration: 3600,
        }
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        // Default is disabled - credentials must be provided explicitly
        Self::disabled()
    }
}

/// Admin API instance
pub struct AdminApi {
    config: AdminConfig,
}

impl AdminApi {
    /// Create a new admin API instance
    pub fn new(config: AdminConfig) -> Self {
        Self { config }
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insecure_password_detection() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "admin".to_string(),
            jwt_secret: "a".repeat(32),
            jwt_expiration: 3600,
        };

        let result = config.validate_for_production();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Password is insecure"));
    }

    #[test]
    fn test_insecure_jwt_secret_detection() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "securepassword123".to_string(),
            jwt_secret: "default-jwt-secret-change-in-production".to_string(),
            jwt_expiration: 3600,
        };

        let result = config.validate_for_production();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JWT secret is insecure"));
    }

    #[test]
    fn test_short_password_rejected() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "short".to_string(),
            jwt_secret: "a".repeat(32),
            jwt_expiration: 3600,
        };

        let result = config.validate_for_production();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));
    }

    #[test]
    fn test_short_jwt_secret_rejected() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "securepassword123".to_string(),
            jwt_secret: "tooshort".to_string(),
            jwt_expiration: 3600,
        };

        let result = config.validate_for_production();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 32 characters"));
    }

    #[test]
    fn test_valid_config_accepted() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "securepassword123".to_string(),
            jwt_secret: "a".repeat(32),
            jwt_expiration: 3600,
        };

        let result = config.validate_for_production();
        assert!(result.is_ok());
    }

    #[test]
    fn test_disabled_config_skips_validation() {
        let config = AdminConfig::disabled();
        let result = config.validate_for_production();
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_is_disabled() {
        let config = AdminConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_security_warnings() {
        let config = AdminConfig {
            enabled: true,
            username: "admin".to_string(),
            password: "admin".to_string(),
            jwt_secret: "default-jwt-secret-change-in-production".to_string(),
            jwt_expiration: 3600,
        };

        let warnings = config.security_warnings();
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("CRITICAL"));
        assert!(warnings[1].contains("CRITICAL"));
    }
}
