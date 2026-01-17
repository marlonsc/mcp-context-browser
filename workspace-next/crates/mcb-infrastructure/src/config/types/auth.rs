//! Authentication configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Password hashing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordAlgorithm {
    Argon2, // Argon2id (recommended)
    Bcrypt, // bcrypt
    Pbkdf2, // PBKDF2
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,

    /// JWT expiration time in seconds
    pub expiration_secs: u64,

    /// JWT refresh token expiration in seconds
    pub refresh_expiration_secs: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: crate::crypto::TokenGenerator::generate_secure_token(32),
            expiration_secs: JWT_DEFAULT_EXPIRATION_SECS,
            refresh_expiration_secs: JWT_REFRESH_EXPIRATION_SECS,
        }
    }
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key authentication enabled
    pub enabled: bool,

    /// API key header name
    pub header: String,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header: API_KEY_HEADER.to_string(),
        }
    }
}

/// Admin API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiKeyConfig {
    /// Admin API key authentication enabled
    pub enabled: bool,

    /// Header name for admin API key
    #[serde(default = "default_admin_key_header")]
    pub header: String,

    /// The actual admin API key (can be set via config or MCB_ADMIN_API_KEY env var)
    #[serde(default)]
    pub key: Option<String>,
}

fn default_admin_key_header() -> String {
    "X-Admin-Key".to_string()
}

impl Default for AdminApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            header: default_admin_key_header(),
            key: std::env::var("MCB_ADMIN_API_KEY").ok(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,

    /// JWT configuration
    pub jwt: JwtConfig,

    /// API key configuration
    pub api_key: ApiKeyConfig,

    /// Admin API key configuration
    #[serde(default)]
    pub admin: AdminApiKeyConfig,

    /// User database path
    pub user_db_path: Option<PathBuf>,

    /// Password hashing algorithm
    pub password_algorithm: PasswordAlgorithm,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt: JwtConfig::default(),
            api_key: ApiKeyConfig::default(),
            admin: AdminApiKeyConfig::default(),
            user_db_path: None,
            password_algorithm: PasswordAlgorithm::Argon2,
        }
    }
}