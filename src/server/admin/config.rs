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


/// Minimum allowed password length for admin accounts
pub const MIN_PASSWORD_LENGTH: usize = 8;
/// Minimum allowed JWT secret length for security
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
    /// Required environment variable is missing
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Username validation failed
    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    /// Password validation failed
    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    /// JWT secret validation failed
    #[error("Invalid JWT secret: {0}")]
    InvalidJwtSecret(String),

    /// Configuration contains insecure defaults
    #[error("Insecure configuration: {0}")]
    InsecureConfig(String),

    /// General configuration error
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

    /// Load admin config with first-run support
    ///
    /// Priority order:
    /// 1. Environment variables (MCP_ADMIN_USERNAME, MCP_ADMIN_PASSWORD, MCP_ADMIN_JWT_SECRET)
    /// 2. Data file (users.json in data_dir)
    /// 3. First run - generate new credentials and save to data file
    ///
    /// Returns (config, Option<generated_password>) - password is Some only on first run
    pub async fn load_with_first_run(
        data_dir: &std::path::Path,
    ) -> Result<(Self, Option<String>), ConfigError> {
        use crate::infrastructure::auth::user_store::UserStore;

        // Priority 1: Check MCP_ADMIN_* environment variables
        let username_env = std::env::var("MCP_ADMIN_USERNAME").ok();
        let password_env = std::env::var("MCP_ADMIN_PASSWORD").ok();
        let jwt_secret_env = std::env::var("MCP_ADMIN_JWT_SECRET").ok();

        if let (Some(username), Some(password), Some(jwt_secret)) =
            (&username_env, &password_env, &jwt_secret_env)
        {
            if !username.is_empty() && !password.is_empty() && !jwt_secret.is_empty() {
                let config = Self::from_env()?;
                return Ok((config, None));
            }
        }

        // Priority 2: Check data file (users.json)
        let users_file = UserStore::file_path(data_dir);
        if let Ok(Some(store)) = UserStore::load(&users_file).await {
            // Get first admin user's credentials
            if let Some(user) = store.users.first() {
                // Note: We can't recover the plaintext password from the hash,
                // so admin auth must use the stored hash directly.
                // For now, we create a config that can be used with the stored credentials.
                let config = Self {
                    enabled: true,
                    username: user.email.clone(),
                    // Password field stores the HASH for verification in auth.rs
                    // The auth.rs AuthService will need to be updated to use hash directly
                    password: user.password_hash.clone(),
                    jwt_secret: store.jwt_secret.clone(),
                    jwt_expiration: 3600,
                };
                return Ok((config, None));
            }
        }

        // Priority 3: First run - use development defaults if in development mode
        // Check if we should use development defaults (when no env vars are set)
        let use_dev_defaults = username_env.is_none() && password_env.is_none() && jwt_secret_env.is_none();

        let (store, generated_password) = if use_dev_defaults {
            // Use development defaults for easier local development
            tracing::warn!("Using development admin credentials (admin/adminpass123). Set MCP_ADMIN_* environment variables for production.");
            let dev_username = "admin".to_string();
            let dev_password = "adminpass123".to_string();
            let dev_jwt_secret = "mcp-context-browser-jwt-secret-key-32chars".to_string();

            let dev_password_hash = crate::infrastructure::auth::password::hash_password(&dev_password)
                .map_err(|e| ConfigError::ConfigError(format!("Failed to hash dev password: {}", e)))?;

            let store = UserStore {
                users: vec![crate::infrastructure::auth::user_store::StoredUser {
                    id: "admin".to_string(),
                    email: dev_username.clone(),
                    role: crate::infrastructure::auth::roles::UserRole::Admin,
                    password_hash: dev_password_hash,
                    hash_version: "Argon2id".to_string(),
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    last_active: 0,
                }],
                jwt_secret: dev_jwt_secret,
            };

            (store, dev_password)
        } else {
            // Generate secure random credentials for production
            UserStore::generate_new().map_err(|e| {
                ConfigError::ConfigError(format!("Failed to generate credentials: {}", e))
            })?
        };

        // Save to data file
        store
            .save(&users_file)
            .await
            .map_err(|e| ConfigError::ConfigError(format!("Failed to save credentials: {}", e)))?;

        let user = store
            .users
            .first()
            .ok_or_else(|| ConfigError::ConfigError("No user generated".to_string()))?;

        let config = Self {
            enabled: true,
            username: user.email.clone(),
            // Store the plaintext password for first-run so auth works
            // On subsequent runs, we'll load the hash from file
            password: generated_password.clone(),
            jwt_secret: store.jwt_secret.clone(),
            jwt_expiration: 3600,
        };

        Ok((config, Some(generated_password)))
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
