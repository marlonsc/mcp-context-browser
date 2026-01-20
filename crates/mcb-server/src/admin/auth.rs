//! Admin API Authentication
//!
//! Provides API key-based authentication for admin endpoints.
//! Uses the `X-Admin-Key` header by default (configurable).
//!
//! # Configuration
//!
//! Authentication can be configured via:
//! - Config file: `auth.admin.enabled = true` and `auth.admin.key = "your-key"`
//! - Environment variable: `MCP__AUTH__ADMIN__KEY=your-key`
//!
//! # Unauthenticated Routes
//!
//! The following routes are exempt from authentication:
//! - `/live` - Kubernetes liveness probe
//! - `/ready` - Kubernetes readiness probe
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use serde::Serialize;
use std::sync::Arc;

/// Admin authentication configuration for the middleware
#[derive(Clone)]
pub struct AdminAuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,
    /// The header name to check for the API key
    pub header_name: String,
    /// The expected API key value
    pub api_key: Option<String>,
}

impl AdminAuthConfig {
    /// Create a new admin auth config
    pub fn new(enabled: bool, header_name: String, api_key: Option<String>) -> Self {
        Self {
            enabled,
            header_name,
            api_key,
        }
    }

    /// Create from infrastructure config
    pub fn from_app_config(config: &mcb_infrastructure::config::AppConfig) -> Self {
        Self {
            enabled: config.auth.admin.enabled,
            header_name: config.auth.admin.header.clone(),
            api_key: config.auth.admin.key.clone(),
        }
    }

    /// Check if the provided key matches the configured key
    pub fn validate_key(&self, provided_key: &str) -> bool {
        match &self.api_key {
            Some(expected) => expected == provided_key,
            None => false, // If no key is configured, reject all requests
        }
    }

    /// Check if authentication is properly configured
    pub fn is_configured(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}

impl Default for AdminAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            header_name: "X-Admin-Key".to_string(),
            api_key: None,
        }
    }
}

/// Authentication error response
#[derive(Serialize)]
pub struct AuthErrorResponse {
    /// Error type
    pub error: &'static str,
    /// Error message
    pub message: String,
}

impl AuthErrorResponse {
    /// Create error for not configured auth
    pub fn not_configured() -> (Status, Json<Self>) {
        (
            Status::ServiceUnavailable,
            Json(Self {
                error: "auth_not_configured",
                message: "Admin authentication is enabled but no API key is configured. \
                         Set MCP__AUTH__ADMIN__KEY environment variable or auth.admin.key in config."
                    .to_string(),
            }),
        )
    }

    /// Create error for invalid key
    pub fn invalid_key() -> (Status, Json<Self>) {
        (
            Status::Unauthorized,
            Json(Self {
                error: "invalid_api_key",
                message: "Invalid admin API key".to_string(),
            }),
        )
    }

    /// Create error for missing key
    pub fn missing_key(header_name: &str) -> (Status, Json<Self>) {
        (
            Status::Unauthorized,
            Json(Self {
                error: "missing_api_key",
                message: format!(
                    "Admin API key required. Provide it in the '{}' header.",
                    header_name
                ),
            }),
        )
    }
}

/// Request guard for admin authentication
///
/// Add this guard to route handlers that require authentication:
///
/// ```rust,ignore
/// #[get("/protected")]
/// fn protected(_auth: AdminAuth) -> &'static str {
///     "Protected content"
/// }
/// ```
///
/// Routes that should bypass authentication (like health checks)
/// should not include this guard.
pub struct AdminAuth;

/// Error type for admin authentication failures
#[derive(Debug)]
pub enum AdminAuthError {
    /// Authentication not configured
    NotConfigured,
    /// Invalid API key
    InvalidKey,
    /// Missing API key
    MissingKey(String),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuth {
    type Error = AdminAuthError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Get auth config from managed state
        let auth_config = match request.rocket().state::<Arc<AdminAuthConfig>>() {
            Some(config) => config,
            None => {
                // No auth config means auth is disabled
                return Outcome::Success(AdminAuth);
            }
        };

        // If authentication is disabled, allow all requests
        if !auth_config.enabled {
            return Outcome::Success(AdminAuth);
        }

        // Check if auth is properly configured
        if !auth_config.is_configured() {
            return Outcome::Error((Status::ServiceUnavailable, AdminAuthError::NotConfigured));
        }

        // Get the API key from headers
        let api_key = request.headers().get_one(&auth_config.header_name);

        match api_key {
            Some(key) if auth_config.validate_key(key) => Outcome::Success(AdminAuth),
            Some(_) => Outcome::Error((Status::Unauthorized, AdminAuthError::InvalidKey)),
            None => Outcome::Error((
                Status::Unauthorized,
                AdminAuthError::MissingKey(auth_config.header_name.clone()),
            )),
        }
    }
}

/// Check if a route should bypass authentication
pub fn is_unauthenticated_route(path: &str) -> bool {
    matches!(path, "/live" | "/ready")
}

/// Wrapper function for backwards compatibility
///
/// In Rocket, authentication is handled via Request Guards rather than
/// middleware. This function is kept for API compatibility but is a no-op.
/// Use the `AdminAuth` request guard directly in route handlers.
///
/// # Migration
///
/// Instead of:
/// ```rust,ignore
/// let router = with_admin_auth(auth_config, router);
/// ```
///
/// Use request guards:
/// ```rust,ignore
/// #[get("/protected")]
/// fn protected(_auth: AdminAuth) -> &'static str {
///     "Protected"
/// }
/// ```
pub fn with_admin_auth(
    auth_config: AdminAuthConfig,
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    rocket.manage(Arc::new(auth_config))
}
