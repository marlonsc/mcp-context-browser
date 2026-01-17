//! Admin API Authentication
//!
//! Provides API key-based authentication for admin endpoints.
//! Uses the `X-Admin-Key` header by default (configurable).
//!
//! # Configuration
//!
//! Authentication can be configured via:
//! - Config file: `auth.admin.enabled = true` and `auth.admin.key = "your-key"`
//! - Environment variable: `MCB_ADMIN_API_KEY=your-key`
//!
//! # Unauthenticated Routes
//!
//! The following routes are exempt from authentication:
//! - `/live` - Kubernetes liveness probe
//! - `/ready` - Kubernetes readiness probe

use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
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
    fn not_configured() -> Response {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Self {
                error: "auth_not_configured",
                message: "Admin authentication is enabled but no API key is configured. \
                         Set MCB_ADMIN_API_KEY environment variable or auth.admin.key in config."
                    .to_string(),
            }),
        )
            .into_response()
    }

    fn invalid_key() -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(Self {
                error: "invalid_api_key",
                message: "Invalid admin API key".to_string(),
            }),
        )
            .into_response()
    }

    fn missing_key(header_name: &str) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            [(
                "WWW-Authenticate",
                HeaderValue::from_static("ApiKey realm=\"admin\""),
            )],
            Json(Self {
                error: "missing_api_key",
                message: format!(
                    "Admin API key required. Provide it in the '{}' header.",
                    header_name
                ),
            }),
        )
            .into_response()
    }
}

/// Admin authentication middleware
///
/// Verifies the API key in the configured header.
/// Returns 401 Unauthorized if authentication fails.
/// Returns 503 Service Unavailable if auth is enabled but not properly configured.
pub async fn admin_auth_middleware(
    State(auth_config): State<Arc<AdminAuthConfig>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if is_unauthenticated_route(path) || !auth_config.enabled {
        return next.run(request).await;
    }

    if !auth_config.is_configured() {
        return AuthErrorResponse::not_configured();
    }

    let api_key = request
        .headers()
        .get(&auth_config.header_name)
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if auth_config.validate_key(key) => next.run(request).await,
        Some(_) => AuthErrorResponse::invalid_key(),
        None => AuthErrorResponse::missing_key(&auth_config.header_name),
    }
}

/// Check if a route should bypass authentication
pub fn is_unauthenticated_route(path: &str) -> bool {
    matches!(path, "/live" | "/ready")
}

/// Create a router with authentication middleware applied
///
/// Returns a function that wraps a router with the authentication layer.
pub fn with_admin_auth(auth_config: AdminAuthConfig, router: axum::Router) -> axum::Router {
    router.layer(axum::middleware::from_fn_with_state(
        Arc::new(auth_config),
        admin_auth_middleware,
    ))
}
