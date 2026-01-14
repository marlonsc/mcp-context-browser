//! Persistent User Store for first-run admin initialization
//!
//! Stores users in `~/.local/share/mcp-context-browser/users.json` with
//! Argon2id password hashing and secure file permissions.

use super::claims::{HashVersion, User};
use super::password::hash_password;
use super::roles::UserRole;
use crate::domain::error::{Error, Result};
use crate::infrastructure::constants::{GENERATED_PASSWORD_LENGTH, JWT_SECRET_MIN_LENGTH_STRICT};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimum JWT secret length for security (alias for centralized constant)
pub const MIN_JWT_SECRET_LENGTH: usize = JWT_SECRET_MIN_LENGTH_STRICT;

/// Characters used for password generation (alphanumeric + special)
const PASSWORD_CHARSET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";

/// Characters used for JWT secret generation (alphanumeric only for base64 compat)
const SECRET_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Persistent user store
///
/// Stores users and JWT secret in a JSON file with proper serialization
/// of the User type including password hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStore {
    /// List of users with their credentials
    pub users: Vec<StoredUser>,
    /// JWT signing secret
    pub jwt_secret: String,
}

/// User data for storage (includes password_hash unlike the skip-serialized User)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredUser {
    /// Id
    pub id: String,
    /// Email
    pub email: String,
    /// Role
    pub role: UserRole,
    /// Password Hash
    pub password_hash: String,
    /// Hash Version
    pub hash_version: String,
    /// Created At
    pub created_at: u64,
    /// Last Active
    pub last_active: u64,
}

impl From<StoredUser> for User {
    fn from(stored: StoredUser) -> Self {
        let hash_version = match stored.hash_version.as_str() {
            "Bcrypt" => HashVersion::Bcrypt,
            _ => HashVersion::Argon2id,
        };

        User {
            id: stored.id,
            email: stored.email,
            role: stored.role,
            password_hash: stored.password_hash,
            hash_version,
            created_at: stored.created_at,
            last_active: stored.last_active,
        }
    }
}

impl From<&User> for StoredUser {
    fn from(user: &User) -> Self {
        let hash_version = match user.hash_version {
            HashVersion::Bcrypt => "Bcrypt".to_string(),
            HashVersion::Argon2id => "Argon2id".to_string(),
        };

        StoredUser {
            id: user.id.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            password_hash: user.password_hash.clone(),
            hash_version,
            created_at: user.created_at,
            last_active: user.last_active,
        }
    }
}

impl UserStore {
    /// Load user store from file
    ///
    /// Returns None if file doesn't exist, Error if file is invalid.
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::Internal {
                message: format!("Failed to read user store: {}", e),
            })?;

        let store: Self = serde_json::from_str(&content).map_err(|e| Error::Internal {
            message: format!("Failed to parse user store: {}", e),
        })?;

        Ok(Some(store))
    }

    /// Save user store to file with 0600 permissions
    pub async fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::Internal {
                    message: format!("Failed to create data directory: {}", e),
                })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| Error::Internal {
            message: format!("Failed to serialize user store: {}", e),
        })?;

        tokio::fs::write(path, &content)
            .await
            .map_err(|e| Error::Internal {
                message: format!("Failed to write user store: {}", e),
            })?;

        // Set permissions to 0600 (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(path, perms)
                .await
                .map_err(|e| Error::Internal {
                    message: format!("Failed to set file permissions: {}", e),
                })?;
        }

        Ok(())
    }

    /// Generate a new user store with random admin password
    ///
    /// Returns the store and the generated plaintext password (for display).
    /// Uses environment variables MCP_ADMIN_USERNAME if available, otherwise defaults to "admin".
    pub fn generate_new() -> Result<(Self, String)> {
        let username = std::env::var("MCP_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let password = generate_secure_string(GENERATED_PASSWORD_LENGTH, PASSWORD_CHARSET);
        let jwt_secret = generate_secure_string(MIN_JWT_SECRET_LENGTH, SECRET_CHARSET);

        let password_hash = hash_password(&password)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let admin_user = StoredUser {
            id: "admin".to_string(),
            email: username.clone(),
            role: UserRole::Admin,
            password_hash,
            hash_version: "Argon2id".to_string(),
            created_at: now,
            last_active: 0,
        };

        let store = Self {
            users: vec![admin_user],
            jwt_secret,
        };

        Ok((store, password))
    }

    /// Create a user store with provided credentials
    pub fn with_credentials(email: &str, password: &str) -> Result<Self> {
        let jwt_secret = generate_secure_string(MIN_JWT_SECRET_LENGTH, SECRET_CHARSET);
        let password_hash = hash_password(password)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let admin_user = StoredUser {
            id: "admin".to_string(),
            email: email.to_string(),
            role: UserRole::Admin,
            password_hash,
            hash_version: "Argon2id".to_string(),
            created_at: now,
            last_active: 0,
        };

        Ok(Self {
            users: vec![admin_user],
            jwt_secret,
        })
    }

    /// Convert to HashMap for AuthConfig
    pub fn to_user_map(&self) -> HashMap<String, User> {
        self.users
            .iter()
            .map(|u| (u.email.clone(), u.clone().into()))
            .collect()
    }

    /// Get the users.json file path within a data directory
    pub fn file_path(data_dir: &Path) -> std::path::PathBuf {
        data_dir.join("users.json")
    }
}

/// Generate a secure random string from the given charset
fn generate_secure_string(length: usize, charset: &[u8]) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

/// First-run status indicating how credentials were obtained
#[derive(Debug, Clone)]
pub enum FirstRunStatus {
    /// Loaded from environment variables
    FromEnv,
    /// Loaded from existing users.json file
    FromFile,
    /// First run - credentials were generated (includes plaintext password)
    Generated {
        /// Generated password (plaintext for first login)
        password: String,
        /// Generated email address
        email: String
    },
    /// First run - credentials were provided by user
    Provided {
        /// User-provided email address
        email: String
    },
}
