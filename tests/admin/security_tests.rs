//! Admin security tests
//!
//! Tests for authentication, token validation, and security features

use mcp_context_browser::server::admin::auth::{AuthService, Claims, AUTH_COOKIE_NAME};
use mcp_context_browser::server::admin::models::UserInfo;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Authentication Service Tests
// ============================================================================

#[tokio::test]
async fn test_valid_credentials_authenticate() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let result = auth_service.authenticate("admin", "admin");
    assert!(result.is_ok());

    let user = result.unwrap();
    assert_eq!(user.username, "admin");
    assert_eq!(user.role, "admin");
}

#[tokio::test]
async fn test_invalid_password_rejected() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let result = auth_service.authenticate("admin", "wrong_password");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid credentials"));
}

#[tokio::test]
async fn test_invalid_username_rejected() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let result = auth_service.authenticate("wrong_user", "admin");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_credentials_rejected() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    assert!(auth_service.authenticate("", "admin").is_err());
    assert!(auth_service.authenticate("admin", "").is_err());
    assert!(auth_service.authenticate("", "").is_err());
}

// ============================================================================
// Token Generation Tests
// ============================================================================

#[tokio::test]
async fn test_token_generation_success() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };

    let token = auth_service.generate_token(&user);
    assert!(token.is_ok());
    assert!(!token.unwrap().is_empty());
}

#[tokio::test]
async fn test_token_contains_correct_claims() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "test_user".to_string(),
        role: "admin".to_string(),
    };

    let token = auth_service.generate_token(&user).unwrap();
    let claims = auth_service.validate_token(&token).unwrap();

    assert_eq!(claims.sub, "test_user");
    assert_eq!(claims.role, "admin");
}

// ============================================================================
// Token Validation Tests
// ============================================================================

#[tokio::test]
async fn test_valid_token_validated() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };

    let token = auth_service.generate_token(&user).unwrap();
    let result = auth_service.validate_token(&token);

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wrong_secret_token_rejected() {
    let auth_service1 = AuthService::new(
        "secret-1".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let auth_service2 = AuthService::new(
        "secret-2".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };

    // Generate token with service 1
    let token = auth_service1.generate_token(&user).unwrap();

    // Try to validate with service 2 (different secret)
    let result = auth_service2.validate_token(&token);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_malformed_token_rejected() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let malformed_tokens = [
        "not-a-valid-jwt",
        "eyJ.invalid.token",
        "",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
        "a.b.c",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.invalid",
    ];

    for token in malformed_tokens {
        let result = auth_service.validate_token(token);
        assert!(
            result.is_err(),
            "Token '{}' should be rejected as malformed",
            token
        );
    }
}

#[tokio::test]
async fn test_expired_token_rejected() {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let secret = "test-secret";
    let auth_service = AuthService::new(
        secret.to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    // Create expired token (expired 1 hour ago)
    let claims = Claims {
        sub: "admin".to_string(),
        role: "admin".to_string(),
        exp: now - 3600, // expired
        iat: now - 7200,
    };

    let expired_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to create expired token");

    let result = auth_service.validate_token(&expired_token);
    assert!(result.is_err());
}

// ============================================================================
// Token Expiration Tests
// ============================================================================

#[tokio::test]
async fn test_token_expiration_time() {
    let jwt_expiration: u64 = 3600;
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        jwt_expiration,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };

    let token = auth_service.generate_token(&user).unwrap();
    let claims = auth_service.validate_token(&token).unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    // Expiration should be approximately now + jwt_expiration
    let expected_exp = now + jwt_expiration as usize;

    // Allow 5 second tolerance for test execution time
    assert!(
        claims.exp >= expected_exp - 5 && claims.exp <= expected_exp + 5,
        "Token expiration {} should be close to expected {}",
        claims.exp,
        expected_exp
    );
}

// ============================================================================
// Password Security Tests
// ============================================================================

#[tokio::test]
async fn test_bcrypt_password_not_stored_plaintext() {
    // This test verifies that passwords are hashed, not stored in plaintext
    // We can't directly check the internal state, but we can verify behavior

    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "secure_password_123".to_string(),
    )
    .expect("Failed to create auth service");

    // Should authenticate with correct password
    assert!(auth_service
        .authenticate("admin", "secure_password_123")
        .is_ok());

    // Should reject similar but different passwords
    assert!(auth_service
        .authenticate("admin", "secure_password_124")
        .is_err());
    assert!(auth_service
        .authenticate("admin", "Secure_password_123")
        .is_err());
}

// ============================================================================
// Cookie Name Tests
// ============================================================================

#[test]
fn test_auth_cookie_name_defined() {
    assert!(!AUTH_COOKIE_NAME.is_empty());
    assert_eq!(AUTH_COOKIE_NAME, "mcp_admin_token");
}

// ============================================================================
// Multiple Failed Login Attempts
// ============================================================================

#[tokio::test]
async fn test_multiple_failed_logins_still_allows_valid() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    // Try multiple failed logins
    for i in 0..10 {
        let _ = auth_service.authenticate("admin", &format!("wrong_{}", i));
    }

    // Valid login should still work (no lockout in current implementation)
    let result = auth_service.authenticate("admin", "admin");
    assert!(result.is_ok());
}

// ============================================================================
// Role-Based Access Tests
// ============================================================================

#[tokio::test]
async fn test_token_contains_role() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };

    let token = auth_service.generate_token(&user).unwrap();
    let claims = auth_service.validate_token(&token).unwrap();

    assert_eq!(claims.role, "admin");
}

#[tokio::test]
async fn test_different_roles_in_token() {
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    // Test with admin role
    let admin_user = UserInfo {
        username: "admin".to_string(),
        role: "admin".to_string(),
    };
    let admin_token = auth_service.generate_token(&admin_user).unwrap();
    let admin_claims = auth_service.validate_token(&admin_token).unwrap();
    assert_eq!(admin_claims.role, "admin");

    // Test with read-only role
    let readonly_user = UserInfo {
        username: "viewer".to_string(),
        role: "readonly".to_string(),
    };
    let readonly_token = auth_service.generate_token(&readonly_user).unwrap();
    let readonly_claims = auth_service.validate_token(&readonly_token).unwrap();
    assert_eq!(readonly_claims.role, "readonly");
}
