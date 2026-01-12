//! Authentication configuration
//!
//! Centralized configuration for authentication settings with validation.

use super::claims::{HashVersion, User};
use super::roles::UserRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

/// Minimum JWT secret length for security
pub const MIN_JWT_SECRET_LENGTH: usize = 32;

/// Default JWT expiration in seconds (24 hours)
pub const DEFAULT_JWT_EXPIRATION: u64 = 86400;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct AuthConfig {
    /// JWT secret key (minimum 32 bytes recommended)
    #[validate(length(min = 1))]
    pub jwt_secret: String,
    /// JWT expiration time in seconds
    #[validate(range(min = 1))]
    pub jwt_expiration: u64,
    /// Issuer claim for JWT
    #[validate(length(min = 1))]
    pub jwt_issuer: String,
    /// Whether authentication is enabled
    pub enabled: bool,
    /// Paths that bypass authentication (e.g., health checks)
    #[serde(default)]
    pub bypass_paths: Vec<String>,
    /// User database (in production, this would be a proper database)
    #[serde(skip)]
    pub users: HashMap<String, User>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let mut users = HashMap::new();

        // Create default admin user
        // In production, ADMIN_PASSWORD environment variable should be set
        let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
            // Always log warning when using default password
            tracing::warn!(
                "ADMIN_PASSWORD not set. Using default password 'admin'. \
                 Set ADMIN_PASSWORD environment variable for production use."
            );
            "admin".to_string()
        });

        // Hash password - fail loudly on error, no silent fallbacks
        let password_hash = match super::password::hash_password(&admin_password) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!(
                    "Failed to hash admin password: {}. Authentication will be disabled.",
                    e
                );
                // Return empty hash - auth will fail if enabled
                String::new()
            }
        };

        let admin_user = User {
            id: "admin".to_string(),
            email: "admin@context.browser".to_string(),
            role: UserRole::Admin,
            password_hash,
            hash_version: HashVersion::Argon2id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_active: 0,
        };
        users.insert("admin@context.browser".to_string(), admin_user);

        // JWT secret - required for secure token generation
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            // Always log warning for default JWT secret
            tracing::warn!(
                "JWT_SECRET not set. Using insecure default. \
                 Set JWT_SECRET environment variable for production use."
            );
            "local-development-secret-change-this-in-production".to_string()
        });

        Self {
            jwt_secret,
            jwt_expiration: DEFAULT_JWT_EXPIRATION,
            jwt_issuer: "mcp-context-browser".to_string(),
            // Default: disabled for local/MCP stdio usage
            // Enable explicitly in config for production/HTTP deployments
            enabled: false,
            bypass_paths: vec![
                "/api/health".to_string(),
                "/api/context/metrics".to_string(),
            ],
            users,
        }
    }
}

impl AuthConfig {
    /// Create a new auth config with explicit values
    pub fn new(jwt_secret: String, jwt_expiration: u64, enabled: bool) -> Self {
        Self {
            jwt_secret,
            jwt_expiration,
            jwt_issuer: "mcp-context-browser".to_string(),
            enabled,
            bypass_paths: vec![
                "/api/health".to_string(),
                "/api/context/metrics".to_string(),
            ],
            users: HashMap::new(),
        }
    }

    /// Validate configuration for production use
    ///
    /// Returns warnings if the configuration uses insecure defaults.
    /// Should be called at startup to alert operators of security issues.
    pub fn validate_for_production(&self) -> Vec<SecurityWarning> {
        let mut warnings = Vec::new();

        if !self.enabled {
            // Auth disabled - no security validation needed
            return warnings;
        }

        // Check JWT secret length
        if self.jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            warnings.push(SecurityWarning {
                code: "JWT_SECRET_TOO_SHORT",
                message: format!(
                    "JWT_SECRET is {} bytes, minimum {} recommended",
                    self.jwt_secret.len(),
                    MIN_JWT_SECRET_LENGTH
                ),
                severity: WarningSeverity::High,
            });
        }

        // Check for default JWT secret
        if self.jwt_secret == "local-development-secret-change-this-in-production" {
            warnings.push(SecurityWarning {
                code: "DEFAULT_JWT_SECRET",
                message: "Using default JWT_SECRET. Set JWT_SECRET environment variable."
                    .to_string(),
                severity: WarningSeverity::Critical,
            });
        }

        // Check for default admin password
        if std::env::var("ADMIN_PASSWORD").is_err() {
            warnings.push(SecurityWarning {
                code: "DEFAULT_ADMIN_PASSWORD",
                message: "Using default admin password. Set ADMIN_PASSWORD environment variable."
                    .to_string(),
                severity: WarningSeverity::Critical,
            });
        }

        // Check for empty password hashes (hash failure)
        for (email, user) in &self.users {
            if user.password_hash.is_empty() {
                warnings.push(SecurityWarning {
                    code: "EMPTY_PASSWORD_HASH",
                    message: format!(
                        "User '{}' has empty password hash. Authentication will fail.",
                        email
                    ),
                    severity: WarningSeverity::Critical,
                });
            }
        }

        warnings
    }

    /// Log all security warnings at startup
    pub fn log_security_warnings(&self) {
        for warning in self.validate_for_production() {
            match warning.severity {
                WarningSeverity::Critical => {
                    tracing::error!("[SECURITY] {}: {}", warning.code, warning.message);
                }
                WarningSeverity::High => {
                    tracing::warn!("[SECURITY] {}: {}", warning.code, warning.message);
                }
                WarningSeverity::Medium => {
                    tracing::info!("[SECURITY] {}: {}", warning.code, warning.message);
                }
            }
        }
    }

    /// Check if a path should bypass authentication
    pub fn should_bypass(&self, path: &str) -> bool {
        self.bypass_paths.iter().any(|p| {
            if p.ends_with('*') {
                path.starts_with(p.trim_end_matches('*'))
            } else {
                path == p
            }
        })
    }

    /// Add a user to the config
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.email.clone(), user);
    }

    /// Get a user by email
    pub fn get_user(&self, email: &str) -> Option<&User> {
        self.users.get(email)
    }
}

/// Security warning from configuration validation
#[derive(Debug, Clone)]
pub struct SecurityWarning {
    /// Warning code for programmatic handling
    pub code: &'static str,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: WarningSeverity,
}

/// Warning severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    /// Critical - should not run in production
    Critical,
    /// High - significant security risk
    High,
    /// Medium - potential security concern
    Medium,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_warnings_when_enabled() {
        let config = AuthConfig {
            enabled: true,
            ..Default::default()
        };

        let warnings = config.validate_for_production();

        // Should have warnings for default credentials
        assert!(
            !warnings.is_empty(),
            "Expected security warnings for default config"
        );

        // Should include JWT secret warning
        assert!(
            warnings
                .iter()
                .any(|w| w.code == "DEFAULT_JWT_SECRET" || w.code == "JWT_SECRET_TOO_SHORT"),
            "Expected JWT secret warning"
        );
    }

    #[test]
    fn test_disabled_config_has_no_warnings() {
        let config = AuthConfig::default();
        assert!(!config.enabled);

        let warnings = config.validate_for_production();
        assert!(warnings.is_empty(), "Disabled auth should have no warnings");
    }

    #[test]
    fn test_bypass_paths() {
        let config = AuthConfig::default();

        assert!(config.should_bypass("/api/health"));
        assert!(config.should_bypass("/api/context/metrics"));
        assert!(!config.should_bypass("/api/search"));
    }

    #[test]
    fn test_bypass_wildcard() {
        let mut config = AuthConfig::default();
        config.bypass_paths.push("/public/*".to_string());

        assert!(config.should_bypass("/public/docs"));
        assert!(config.should_bypass("/public/images/logo.png"));
        assert!(!config.should_bypass("/private/data"));
    }
}
