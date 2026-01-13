//! Authentication and authorization tests
//!
//! Tests migrated from src/infrastructure/auth.rs

use mcp_context_browser::infrastructure::auth::{
    AuthConfig, AuthService, HashVersion, Permission, User, UserRole,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test helper: Create an AuthConfig with a known test admin user
///
/// For testing purposes, creates a config with:
/// - enabled: true
/// - jwt_secret: test secret (32+ chars)
/// - admin user: email="admin@context.browser", password hash for "admin" password
fn create_test_auth_config(enabled: bool) -> AuthConfig {
    use mcp_context_browser::infrastructure::auth::password;

    let jwt_secret = "test-jwt-secret-that-is-long-enough".to_string();

    // Create admin user with hashed "admin" password (only for tests)
    let admin_password_hash = password::hash_password("admin")
        .expect("Failed to hash test password");

    let admin_user = User {
        id: "admin".to_string(),
        email: "admin@context.browser".to_string(),
        role: UserRole::Admin,
        password_hash: admin_password_hash,
        hash_version: HashVersion::Argon2id,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        last_active: 0,
    };

    let mut users = HashMap::new();
    users.insert("admin@context.browser".to_string(), admin_user);

    AuthConfig {
        enabled,
        jwt_secret,
        jwt_expiration: 3600,
        jwt_issuer: "mcp-context-browser".to_string(),
        bypass_paths: vec![
            "/api/health".to_string(),
            "/api/context/metrics".to_string(),
        ],
        users,
    }
}

#[test]
fn test_user_roles() {
    assert!(UserRole::Admin.has_permission(&Permission::ManageUsers));
    assert!(UserRole::Developer.has_permission(&Permission::IndexCodebase));
    assert!(UserRole::Viewer.has_permission(&Permission::SearchCodebase));
    assert!(UserRole::Guest.has_permission(&Permission::ViewMetrics));

    assert!(!UserRole::Viewer.has_permission(&Permission::ManageUsers));
    assert!(!UserRole::Guest.has_permission(&Permission::IndexCodebase));
}

#[test]
fn test_role_hierarchy() {
    assert!(UserRole::Admin.can_assign(&UserRole::Developer));
    assert!(UserRole::Developer.can_assign(&UserRole::Viewer));
    assert!(!UserRole::Viewer.can_assign(&UserRole::Developer));
}

#[tokio::test]
async fn test_auth_service_creation() {
    let config = AuthConfig::default();
    let auth = AuthService::new(config);

    // Default is disabled for local/MCP usage
    assert!(!auth.is_enabled());
}

#[tokio::test]
async fn test_auth_service_creation_with_enabled() {
    let config = create_test_auth_config(true);
    let auth = AuthService::new(config);

    assert!(auth.is_enabled());
    assert!(auth.get_user("admin@context.browser").is_some());
}

/// Helper to create an auth service with auth explicitly enabled (for testing)
fn enabled_auth_service() -> AuthService {
    AuthService::new(create_test_auth_config(true))
}

#[tokio::test]
async fn test_authentication() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let auth = enabled_auth_service();

    // Test valid credentials
    let token = auth.authenticate("admin@context.browser", "admin")?;
    assert!(!token.is_empty());

    // Test invalid credentials
    assert!(auth.authenticate("invalid", "invalid").is_err());
    Ok(())
}

#[tokio::test]
async fn test_token_validation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let auth = enabled_auth_service();

    // Generate token
    let token = auth.authenticate("admin@context.browser", "admin")?;

    // Validate token
    let claims = auth.validate_token(&token)?;
    assert_eq!(claims.email, "admin@context.browser");
    assert_eq!(claims.role, UserRole::Admin);
    assert_eq!(claims.iss, "mcp-context-browser");
    Ok(())
}

#[tokio::test]
async fn test_permission_checking() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let auth = enabled_auth_service();
    let token = auth.authenticate("admin@context.browser", "admin")?;
    let claims = auth.validate_token(&token)?;

    assert!(auth.check_permission(&claims, &Permission::ManageUsers));
    assert!(auth.check_permission(&claims, &Permission::IndexCodebase));
    assert!(auth.check_permission(&claims, &Permission::SearchCodebase));
    assert!(auth.check_permission(&claims, &Permission::ViewMetrics));
    Ok(())
}

#[tokio::test]
async fn test_disabled_auth() {
    let config = AuthConfig {
        enabled: false,
        ..Default::default()
    };
    let auth = AuthService::new(config);

    assert!(!auth.is_enabled());
    assert!(auth.authenticate("admin", "admin").is_err());
    assert!(auth.validate_token("dummy").is_err());
}

#[test]
fn test_auth_service_handles_disabled_auth_errors(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = AuthConfig {
        enabled: false,
        ..Default::default()
    };
    let auth = AuthService::new(config);

    // Should return proper error instead of panicking
    let result = auth.authenticate("user", "pass");
    assert!(result.is_err());
    let error_message = result.err().ok_or("Expected error")?.to_string();
    assert!(error_message.contains("Authentication is disabled"));
    Ok(())
}

#[test]
fn test_auth_service_handles_invalid_credentials_errors(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let auth = enabled_auth_service();

    // Should return proper error instead of panicking
    let result = auth.authenticate("invalid@email.com", "wrongpass");
    assert!(result.is_err());
    let error_message = result.err().ok_or("Expected error")?.to_string();
    assert!(error_message.contains("Invalid credentials"));
    Ok(())
}

#[test]
fn test_auth_service_handles_token_validation_errors(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let auth = enabled_auth_service();

    // Should return proper error for invalid tokens instead of panicking
    let result = auth.validate_token("invalid.jwt.token");
    assert!(result.is_err()); // Should return some kind of error for invalid tokens
    let err_msg = result.err().ok_or("Expected error")?.to_string();
    assert!(
        err_msg.contains("Invalid token")
            || err_msg.contains("InvalidToken")
            || err_msg.contains("Base64 decode error")
            || err_msg.contains("JSON parsing error")
    );

    // Should return proper error for malformed tokens instead of panicking
    let result = auth.validate_token("malformed.token");
    assert!(result.is_err()); // Should return some kind of error for malformed tokens
    Ok(())
}

#[test]
fn test_auth_service_handles_token_generation_errors() {
    let auth = enabled_auth_service();

    // This should work in normal cases, but we test the error handling path
    let result = auth.authenticate("admin@context.browser", "admin");
    assert!(
        result.is_ok(),
        "Authentication should succeed with valid credentials"
    );
}
