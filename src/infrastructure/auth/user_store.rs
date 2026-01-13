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
pub const MIN_JWT_SECRET_LENGTH: usize = JWT_SECRET_MIN_LENGTH_STRICT as usize;

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
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub password_hash: String,
    pub hash_version: String,
    pub created_at: u64,
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
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path).map_err(|e| Error::Internal {
            message: format!("Failed to read user store: {}", e),
        })?;

        let store: Self = serde_json::from_str(&content).map_err(|e| Error::Internal {
            message: format!("Failed to parse user store: {}", e),
        })?;

        Ok(Some(store))
    }

    /// Save user store to file with 0600 permissions
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Internal {
                message: format!("Failed to create data directory: {}", e),
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| Error::Internal {
            message: format!("Failed to serialize user store: {}", e),
        })?;

        std::fs::write(path, &content).map_err(|e| Error::Internal {
            message: format!("Failed to write user store: {}", e),
        })?;

        // Set permissions to 0600 (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, perms).map_err(|e| Error::Internal {
                message: format!("Failed to set file permissions: {}", e),
            })?;
        }

        Ok(())
    }

    /// Generate a new user store with random admin password
    ///
    /// Returns the store and the generated plaintext password (for display).
    pub fn generate_new() -> Result<(Self, String)> {
        let password = generate_secure_string(GENERATED_PASSWORD_LENGTH, PASSWORD_CHARSET);
        let jwt_secret = generate_secure_string(MIN_JWT_SECRET_LENGTH, SECRET_CHARSET);

        let password_hash = hash_password(&password)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let admin_user = StoredUser {
            id: "admin".to_string(),
            email: "admin@local".to_string(),
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
    Generated { password: String, email: String },
    /// First run - credentials were provided by user
    Provided { email: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_creates_valid_store() {
        let (store, password) = UserStore::generate_new().expect("should generate");

        assert_eq!(store.users.len(), 1);
        assert_eq!(store.users[0].email, "admin@local");
        assert_eq!(store.users[0].role, UserRole::Admin);
        assert!(!store.users[0].password_hash.is_empty());
        assert!(store.users[0].password_hash.starts_with("$argon2"));
        assert_eq!(password.len(), GENERATED_PASSWORD_LENGTH);
        assert_eq!(store.jwt_secret.len(), MIN_JWT_SECRET_LENGTH);
    }

    #[test]
    fn test_with_credentials_hashes_password() {
        let store = UserStore::with_credentials("test@example.com", "MyPassword123")
            .expect("should create");

        assert_eq!(store.users.len(), 1);
        assert_eq!(store.users[0].email, "test@example.com");
        assert!(store.users[0].password_hash.starts_with("$argon2"));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().expect("temp dir");
        let path = temp_dir.path().join("users.json");

        let (original, _password) = UserStore::generate_new().expect("should generate");
        original.save(&path).expect("should save");

        let loaded = UserStore::load(&path)
            .expect("should load")
            .expect("should exist");

        assert_eq!(loaded.users.len(), original.users.len());
        assert_eq!(loaded.users[0].email, original.users[0].email);
        assert_eq!(loaded.jwt_secret, original.jwt_secret);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let temp_dir = TempDir::new().expect("temp dir");
        let path = temp_dir.path().join("nonexistent.json");

        let result = UserStore::load(&path).expect("should not error");
        assert!(result.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_file_permissions_are_0600() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().expect("temp dir");
        let path = temp_dir.path().join("users.json");

        let (store, _) = UserStore::generate_new().expect("should generate");
        store.save(&path).expect("should save");

        let metadata = std::fs::metadata(&path).expect("metadata");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "File permissions should be 0600");
    }

    #[test]
    fn test_jwt_secret_length() {
        let (store, _) = UserStore::generate_new().expect("should generate");
        assert!(
            store.jwt_secret.len() >= MIN_JWT_SECRET_LENGTH,
            "JWT secret should be at least {} chars",
            MIN_JWT_SECRET_LENGTH
        );
    }

    #[test]
    fn test_to_user_map() {
        let (store, _) = UserStore::generate_new().expect("should generate");
        let map = store.to_user_map();

        assert_eq!(map.len(), 1);
        assert!(map.contains_key("admin@local"));

        let user = map.get("admin@local").unwrap();
        assert_eq!(user.role, UserRole::Admin);
        assert!(user.password_hash.starts_with("$argon2"));
    }

    #[test]
    fn test_stored_user_conversion() {
        let stored = StoredUser {
            id: "test".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::Developer,
            password_hash: "$argon2id$test".to_string(),
            hash_version: "Argon2id".to_string(),
            created_at: 12345,
            last_active: 0,
        };

        let user: User = stored.clone().into();
        assert_eq!(user.id, "test");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.hash_version, HashVersion::Argon2id);

        let back: StoredUser = (&user).into();
        assert_eq!(back.email, stored.email);
        assert_eq!(back.hash_version, "Argon2id");
    }
}
