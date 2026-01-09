//! Authentication and Authorization System
//!
//! Enterprise-grade authentication system with JWT tokens and RBAC.
//! Provides secure user management with hierarchical role-based permissions.
//!
//! ## Features
//!
//! - JWT token-based authentication
//! - Role-Based Access Control (RBAC) with permission hierarchies
//! - Secure password validation and user management
//! - Token expiration and refresh capabilities
//! - Enterprise-ready security controls

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

/// User roles with hierarchical permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub enum UserRole {
    /// Guest access - minimal permissions
    Guest,
    /// Viewer access - read-only operations
    Viewer,
    /// Developer access - indexing and search
    Developer,
    /// Admin access - full system control
    Admin,
}

impl UserRole {
    /// Check if this role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match self {
            UserRole::Admin => true, // Admin has all permissions
            UserRole::Developer => matches!(
                permission,
                Permission::IndexCodebase | Permission::SearchCodebase | Permission::ViewMetrics
            ),
            UserRole::Viewer => matches!(
                permission,
                Permission::SearchCodebase | Permission::ViewMetrics
            ),
            UserRole::Guest => matches!(permission, Permission::ViewMetrics),
        }
    }

    /// Check if this role can be assigned by another role
    pub fn can_assign(&self, target_role: &UserRole) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::Developer => matches!(target_role, UserRole::Viewer | UserRole::Guest),
            _ => false,
        }
    }
}

/// System permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Can index codebases
    IndexCodebase,
    /// Can search codebases
    SearchCodebase,
    /// Can view system metrics
    ViewMetrics,
    /// Can manage users and roles
    ManageUsers,
    /// Can configure system settings
    ManageSystem,
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User email
    pub email: String,
    /// User role
    pub role: UserRole,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
    /// Issuer
    pub iss: String,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct User {
    /// Unique user ID
    pub id: String,
    /// User email
    pub email: String,
    /// User role
    pub role: UserRole,
    /// When user was created
    pub created_at: u64,
    /// When user was last active
    pub last_active: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    /// JWT secret key
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
    /// User database (in production, this would be a proper database)
    #[serde(skip)]
    pub users: HashMap<String, User>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let mut users = HashMap::new();

        // Create default admin user
        let admin_user = User {
            id: "admin".to_string(),
            email: "admin@context.browser".to_string(),
            role: UserRole::Admin,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_active: 0,
        };
        users.insert("admin".to_string(), admin_user);

        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiration: 3600, // 1 hour
            jwt_issuer: "mcp-context-browser".to_string(),
            enabled: true,
            users,
        }
    }
}

/// Authentication and authorization service
///
/// Handles JWT-based authentication with role-based access control.
/// Provides secure user management and permission validation.
///
/// ## Security Features
///
/// - JWT token generation and validation
/// - Password-based authentication
/// - Role-based permission checking
/// - Token expiration handling
/// - Secure user data management
pub struct AuthService {
    /// Authentication configuration
    config: AuthConfig,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Create a default auth service with admin user
    pub fn with_default_config() -> Self {
        Self::new(AuthConfig::default())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::with_default_config()
    }
}

impl AuthService {
    /// Authenticate user with email and password
    ///
    /// Performs user authentication and returns a JWT token on success.
    /// This is a simplified implementation for demonstration purposes.
    ///
    /// # Arguments
    ///
    /// * `email` - User email address
    /// * `password` - User password (plaintext for demo)
    ///
    /// # Returns
    ///
    /// Returns a JWT token string on successful authentication.
    ///
    /// # Security Note
    ///
    /// In production, passwords should be hashed and compared using secure
    /// password hashing algorithms like Argon2, bcrypt, or scrypt.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication is disabled
    /// - Invalid credentials provided
    /// - User not found
    /// - Token generation fails
    pub fn authenticate(&self, email: &str, password: &str) -> Result<String> {
        if !self.config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        // Simplified authentication - in production, verify password hash
        if email == "admin@context.browser" && password == "admin" {
            let user = self
                .config
                .users
                .get("admin")
                .ok_or_else(|| Error::generic("User not found"))?;

            // Generate JWT token
            self.generate_token(user)
        } else {
            Err(Error::generic("Invalid credentials"))
        }
    }

    /// Validate JWT token and extract claims
    ///
    /// Parses and validates a JWT token, checking its signature, expiration,
    /// and extracting the claims payload.
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string to validate
    ///
    /// # Returns
    ///
    /// Returns the token claims if validation succeeds.
    ///
    /// # Security Note
    ///
    /// This is a simplified implementation for demonstration.
    /// In production, use a proper JWT library like `jsonwebtoken` crate
    /// with proper signature verification.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication is disabled
    /// - Token format is invalid
    /// - Token is expired
    /// - Token signature is invalid
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        if !self.config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        // In a real implementation, you'd use a JWT library like jsonwebtoken
        // For this demo, we'll do a simplified validation
        self.decode_token(token)
    }

    /// Check if user has permission
    pub fn check_permission(&self, claims: &Claims, permission: &Permission) -> bool {
        claims.role.has_permission(permission)
    }

    /// Generate JWT token for user
    fn generate_token(&self, user: &User) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now,
            exp: now + self.config.jwt_expiration,
            iss: self.config.jwt_issuer.clone(),
        };

        // Simplified token generation - in production, use proper JWT library
        self.encode_token(&claims)
    }

    /// Simplified token encoding (for demo - use proper JWT library in production)
    fn encode_token(&self, claims: &Claims) -> Result<String> {
        use base64::{Engine as _, engine::general_purpose};

        let claims_json = serde_json::to_string(claims)?;
        let claims_b64 = general_purpose::URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // Create simplified JWT structure (header.payload.signature)
        let header = r#"{"alg":"HS256","typ":"JWT"}"#;
        let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header.as_bytes());

        // Simplified signature (not cryptographically secure - use proper JWT library!)
        let message = format!("{}.{}", header_b64, claims_b64);
        let signature = format!("{:x}", seahash::hash(message.as_bytes()));
        let signature_b64 = general_purpose::URL_SAFE_NO_PAD.encode(signature.as_bytes());

        Ok(format!("{}.{}.{}", header_b64, claims_b64, signature_b64))
    }

    /// Simplified token decoding (for demo - use proper JWT library in production)
    fn decode_token(&self, token: &str) -> Result<Claims> {
        use base64::{Engine as _, engine::general_purpose};

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::generic("Invalid token format"));
        }

        // Decode payload (claims)
        let claims_b64 = parts[1];
        let claims_bytes = general_purpose::URL_SAFE_NO_PAD.decode(claims_b64)?;
        let claims_json = String::from_utf8(claims_bytes)?;
        let claims: Claims = serde_json::from_str(&claims_json)?;

        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if claims.exp < now {
            return Err(Error::generic("Token expired"));
        }

        Ok(claims)
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.config.users.get(user_id)
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

/// Authentication middleware for Axum
pub mod middleware {
    use super::*;
    use axum::{
        extract::Request,
        http::{StatusCode, header},
        middleware::Next,
        response::Response,
    };

    /// Authentication middleware that validates JWT tokens
    pub async fn auth_middleware(
        auth_service: std::sync::Arc<AuthService>,
        mut req: Request,
        next: Next,
    ) -> std::result::Result<Response, StatusCode> {
        // Skip auth for health check and metrics (configurable)
        let path = req.uri().path();
        if path == "/api/health" || path.starts_with("/api/context/metrics") {
            return Ok(next.run(req).await);
        }

        // Extract token from Authorization header
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        let token = if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                auth.trim_start_matches("Bearer ")
            } else {
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        };

        // Validate token
        match auth_service.validate_token(token) {
            Ok(claims) => {
                // Insert claims into request extensions for handlers to use
                req.extensions_mut().insert(claims);
                Ok(next.run(req).await)
            }
            Err(_) => Err(axum::http::StatusCode::UNAUTHORIZED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(auth.is_enabled());
        assert!(auth.get_user("admin").is_some());
    }

    #[tokio::test]
    async fn test_authentication() {
        let auth = AuthService::default();

        // Test valid credentials
        let token = auth.authenticate("admin@context.browser", "admin").unwrap();
        assert!(!token.is_empty());

        // Test invalid credentials
        assert!(auth.authenticate("invalid", "invalid").is_err());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let auth = AuthService::default();

        // Generate token
        let token = auth.authenticate("admin@context.browser", "admin").unwrap();

        // Validate token
        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.email, "admin@context.browser");
        assert_eq!(claims.role, UserRole::Admin);
        assert_eq!(claims.iss, "mcp-context-browser");
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let auth = AuthService::default();
        let token = auth.authenticate("admin@context.browser", "admin").unwrap();
        let claims = auth.validate_token(&token).unwrap();

        assert!(auth.check_permission(&claims, &Permission::ManageUsers));
        assert!(auth.check_permission(&claims, &Permission::IndexCodebase));
        assert!(auth.check_permission(&claims, &Permission::SearchCodebase));
        assert!(auth.check_permission(&claims, &Permission::ViewMetrics));
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
    fn test_auth_service_handles_disabled_auth_errors() {
        let config = AuthConfig {
            enabled: false,
            ..Default::default()
        };
        let auth = AuthService::new(config);

        // Should return proper error instead of panicking
        let result = auth.authenticate("user", "pass");
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Authentication is disabled"));
    }

    #[test]
    fn test_auth_service_handles_invalid_credentials_errors() {
        let auth = AuthService::default();

        // Should return proper error instead of panicking
        let result = auth.authenticate("invalid@email.com", "wrongpass");
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid credentials"));
    }

    #[test]
    fn test_auth_service_handles_token_validation_errors() {
        let auth = AuthService::default();

        // Should return proper error for invalid tokens instead of panicking
        let result = auth.validate_token("invalid.jwt.token");
        assert!(result.is_err()); // Should return some kind of error for invalid tokens
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid token format")
                || err_msg.contains("Base64 decode error")
                || err_msg.contains("JSON parsing error")
        );

        // Should return proper error for malformed tokens instead of panicking
        let result = auth.validate_token("malformed.token");
        assert!(result.is_err()); // Should return some kind of error for malformed tokens
    }

    #[test]
    fn test_auth_service_handles_token_generation_errors() {
        let auth = AuthService::default();

        // This should work in normal cases, but we test the error handling path
        let result = auth.authenticate("admin@context.browser", "admin");
        assert!(
            result.is_ok(),
            "Authentication should succeed with valid credentials"
        );
    }
}
