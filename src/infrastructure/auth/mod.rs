//! Authentication and Authorization System
//!
//! Enterprise-grade authentication system with JWT tokens, RBAC, and rate limiting.
//! Provides secure user management with hierarchical role-based permissions.
//!
//! # Features
//!
//! - JWT token-based authentication
//! - Argon2id password hashing (with bcrypt migration support)
//! - Role-Based Access Control (RBAC) with permission hierarchies
//! - API key authentication (alternative to JWT)
//! - Rate limiting and brute-force protection
//! - Configurable auth bypass for health/metrics endpoints
//!
//! # Module Structure
//!
//! - `roles` - User roles and permissions definitions
//! - `claims` - JWT claims and user structures
//! - `config` - Authentication configuration with validation
//! - `password` - Password hashing (Argon2id/bcrypt)
//! - `service` - Main authentication service
//! - `middleware` - Axum middleware and extractors
//! - `api_keys` - API key management
//! - `rate_limit` - Rate limiting and DDoS protection
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_context_browser::infrastructure::auth::{AuthService, AuthConfig};
//!
//! # fn example() -> anyhow::Result<()> {
//! // Create auth service with default config
//! let auth_service = AuthService::with_default_config();
//!
//! // Authenticate a user (would need user to be created first)
//! // let token = auth_service.authenticate("user@example.com", "password")?;
//!
//! // Validate a token
//! // let claims = auth_service.validate_token(&token)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Checking Permissions
//!
//! ```rust
//! use mcp_context_browser::infrastructure::auth::{UserRole, Permission};
//!
//! let role = UserRole::Admin;
//! assert!(role.has_permission(&Permission::ManageUsers));
//! assert!(role.has_permission(&Permission::SearchCodebase));
//! ```

pub mod api_keys;
pub mod claims;
pub mod config;
pub mod middleware;
pub mod password;
pub mod rate_limit;
pub mod roles;
pub mod service;
pub mod user_store;

// Re-export main types for convenience
pub use claims::{Claims, HashVersion, User};
pub use config::{AuthConfig, SecurityWarning, WarningSeverity, MIN_JWT_SECRET_LENGTH};
pub use middleware::{
    auth_middleware, auth_middleware_simple, ClaimsExtractor, OptionalClaimsExtractor,
    RequirePermission,
};
pub use roles::{Permission, UserRole};
pub use service::{AuthService, AuthServiceInterface};

// Re-export API key types
pub use api_keys::{ApiKey, ApiKeyStore, API_KEY_PREFIX};

// Re-export rate limiting types
pub use rate_limit::{
    AuthRateLimiter, RateLimitConfig, RateLimitError, RateLimitStatus, RateLimiterState,
};

// Re-export user store types
pub use user_store::{FirstRunStatus, StoredUser, UserStore};
