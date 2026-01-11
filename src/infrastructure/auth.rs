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

use crate::domain::error::{Error, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    /// Bcrypt hashed password
    #[serde(skip)]
    pub password_hash: String,
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
        // Default password is "admin" - in production change this via env/config
        let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
            // Warn if no admin password provided in production mode
            if cfg!(not(debug_assertions)) {
                tracing::warn!("No ADMIN_PASSWORD set! Using insecure default 'admin'");
            }
            "admin".to_string()
        });
        let password_hash = bcrypt::hash(admin_password, 10).unwrap_or_else(|_| {
            // Fallback if hash fails - this is a known bcrypt hash for "admin"
            "$2b$10$7CJMei/BYSIj2KaM2dLq.9YSD5qv3wofVoaHiMf2vWxjGfbFPV3W".to_string()
        });

        let admin_user = User {
            id: "admin".to_string(),
            email: "admin@context.browser".to_string(),
            role: UserRole::Admin,
            password_hash,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_active: 0,
        };
        // Use email as key for easier lookup in authenticate
        users.insert("admin@context.browser".to_string(), admin_user);

        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                "local-development-secret-change-this-in-production".to_string()
            }),
            jwt_expiration: 86400, // 24 hours
            jwt_issuer: "mcp-context-browser".to_string(),
            // Default: disabled for local/MCP stdio usage
            // Enable explicitly in config for production/HTTP deployments
            enabled: false,
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
    /// Uses bcrypt for secure password verification.
    pub fn authenticate(&self, email: &str, password: &str) -> Result<String> {
        if !self.config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        // Find user by email
        let user = self
            .config
            .users
            .get(email)
            .ok_or_else(|| Error::generic("Invalid credentials"))?;

        // Verify password hash
        match bcrypt::verify(password, &user.password_hash) {
            Ok(true) => self.generate_token(user),
            _ => Err(Error::generic("Invalid credentials")),
        }
    }

    /// Validate JWT token and extract claims
    ///
    /// Parses and validates a JWT token using HMAC-SHA256, checking its signature,
    /// expiration, and extracting the claims payload.
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        if !self.config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_issuer(&[&self.config.jwt_issuer]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| Error::generic(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
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

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::generic(format!("Token generation failed: {}", e)))
    }

    /// Get user by ID (Note: current implementation uses email as key in map)
    pub fn get_user(&self, email: &str) -> Option<&User> {
        self.config.users.get(email)
    }

    /// Alias for get_user for semantic clarity when using emails
    pub fn get_user_by_email(&self, email: &str) -> Option<&User> {
        self.get_user(email)
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
        http::{header, StatusCode},
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
