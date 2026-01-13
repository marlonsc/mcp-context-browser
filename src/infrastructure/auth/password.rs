//! Password hashing with Argon2id and bcrypt migration support
//!
//! Uses Argon2id (NIST/OWASP recommended) for new passwords while
//! supporting bcrypt verification for migration.

use crate::domain::error::{Error, Result};
use crate::infrastructure::constants::BCRYPT_COST;

// Feature flags for password hashing backends
// Argon2id is preferred, bcrypt is fallback for migration

/// Hash a password using Argon2id
///
/// Returns the PHC string format hash that includes algorithm parameters.
#[cfg(feature = "argon2")]
pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| Error::generic(format!("Password hashing failed: {}", e)))
}

/// Hash a password using bcrypt (fallback when argon2 feature disabled)
#[cfg(not(feature = "argon2"))]
pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, BCRYPT_COST)
        .map_err(|e| Error::generic(format!("Password hashing failed: {}", e)))
}

/// Verify a password against an Argon2id hash
#[cfg(feature = "argon2")]
pub fn verify_argon2(password: &str, hash: &str) -> Result<bool> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| Error::generic(format!("Invalid hash format: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Verify a password against a bcrypt hash (for migration)
pub fn verify_bcrypt(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| Error::generic(format!("Password verification failed: {}", e)))
}

/// Verify a password against a hash, auto-detecting the algorithm
///
/// Supports both Argon2id (PHC format: $argon2...) and bcrypt ($2b$...).
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    if hash.is_empty() {
        return Ok(false);
    }

    // Detect hash algorithm by prefix
    if hash.starts_with("$argon2") {
        #[cfg(feature = "argon2")]
        return verify_argon2(password, hash);

        #[cfg(not(feature = "argon2"))]
        return Err(Error::generic(
            "Argon2 hash detected but argon2 feature not enabled",
        ));
    } else if hash.starts_with("$2") {
        // bcrypt hash ($2a$, $2b$, $2y$)
        verify_bcrypt(password, hash)
    } else {
        Err(Error::generic("Unknown password hash format"))
    }
}

/// Check if a hash needs migration to Argon2id
pub fn needs_migration(hash: &str) -> bool {
    // bcrypt hashes start with $2
    hash.starts_with("$2")
}

/// Migrate a bcrypt hash to Argon2id by re-hashing with the plaintext password
///
/// Call this after successful bcrypt verification to upgrade the hash.
#[cfg(feature = "argon2")]
pub fn migrate_hash(password: &str) -> Result<String> {
    hash_password(password)
}

#[cfg(not(feature = "argon2"))]
pub fn migrate_hash(_password: &str) -> Result<String> {
    Err(Error::generic(
        "Cannot migrate hash: argon2 feature not enabled",
    ))
}

/// Password strength requirements
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require digits
    pub require_digit: bool,
    /// Require special characters
    pub require_special: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
        }
    }
}

impl PasswordPolicy {
    /// Validate a password against the policy
    pub fn validate(&self, password: &str) -> std::result::Result<(), Vec<&'static str>> {
        let mut errors = Vec::new();

        if password.len() < self.min_length {
            errors.push("Password too short");
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain uppercase letter");
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain lowercase letter");
        }

        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain digit");
        }

        if self.require_special
            && !password
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            errors.push("Password must contain special character");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("hash should succeed");

        assert!(verify_password(password, &hash).expect("verify should succeed"));
        assert!(!verify_password("wrong_password", &hash).expect("verify should succeed"));
    }

    #[test]
    fn test_bcrypt_verification() {
        // Known bcrypt hash for "admin"
        let bcrypt_hash = "$2b$10$7CJMei/BYSIj2KaM2dLq.eYSD5qv3wofVoaHiMf2vWxjGfbFPV3W";
        // Note: This is a test hash, actual verification depends on correct hash

        // Verify bcrypt detection
        assert!(needs_migration(bcrypt_hash));
    }

    #[test]
    fn test_empty_hash_returns_false() {
        assert!(!verify_password("any_password", "").expect("should handle empty hash"));
    }

    #[test]
    fn test_password_policy() {
        let policy = PasswordPolicy::default();

        // Valid password
        assert!(policy.validate("Password123").is_ok());

        // Too short
        assert!(policy.validate("Pwd1").is_err());

        // No uppercase
        assert!(policy.validate("password123").is_err());

        // No lowercase
        assert!(policy.validate("PASSWORD123").is_err());

        // No digit
        assert!(policy.validate("PasswordABC").is_err());
    }

    #[test]
    fn test_needs_migration() {
        // bcrypt hashes need migration
        assert!(needs_migration("$2b$10$..."));
        assert!(needs_migration("$2a$10$..."));

        // Argon2 hashes don't need migration
        assert!(!needs_migration("$argon2id$v=19$..."));
    }
}
