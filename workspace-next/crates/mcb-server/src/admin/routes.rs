//! Admin API routes
//!
//! Route definitions for the admin API endpoints.

use axum::{
    routing::{get, patch, post},
    Router,
};

use super::handlers::{
    extended_health_check, get_config, get_indexing_status, get_metrics, health_check,
    liveness_check, readiness_check, reload_config, shutdown, update_config_section, AdminState,
};

/// Create the admin API router
///
/// Routes:
/// - GET /health - Health check with uptime and status
/// - GET /health/extended - Extended health check with dependency status
/// - GET /metrics - Performance metrics
/// - GET /indexing - Indexing operations status
/// - GET /ready - Kubernetes readiness probe
/// - GET /live - Kubernetes liveness probe
/// - POST /shutdown - Initiate graceful server shutdown
/// - GET /config - View current configuration (sanitized)
/// - POST /config/reload - Trigger configuration reload
/// - PATCH /config/:section - Update configuration section
pub fn admin_router(state: AdminState) -> Router {
    Router::new()
        // Health and monitoring
        .route("/health", get(health_check))
        .route("/health/extended", get(extended_health_check))
        .route("/metrics", get(get_metrics))
        .route("/indexing", get(get_indexing_status))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        // Service control
        .route("/shutdown", post(shutdown))
        // Configuration management
        .route("/config", get(get_config))
        .route("/config/reload", post(reload_config))
        .route("/config/{section}", patch(update_config_section))
        .with_state(state)
}

/// Create admin router with a prefix
///
/// This creates a nested router under the given prefix, e.g., `/admin`.
pub fn admin_router_with_prefix(prefix: &str, state: AdminState) -> Router {
    Router::new().nest(prefix, admin_router(state))
}

/// Create admin router with authentication enabled
///
/// Wraps the admin router with API key authentication middleware.
/// Routes `/live` and `/ready` are exempt from authentication for K8s probes.
///
/// # Arguments
///
/// * `state` - Admin state with metrics, indexing, and shutdown coordination
/// * `auth_config` - Authentication configuration (enabled, header name, API key)
///
/// # Example
///
/// ```ignore
/// use mcb_server::admin::{AdminState, AdminAuthConfig, admin_router_with_auth};
///
/// let auth_config = AdminAuthConfig::new(true, "X-Admin-Key".to_string(), Some("secret".to_string()));
/// let router = admin_router_with_auth(state, auth_config);
/// ```
pub fn admin_router_with_auth(state: AdminState, auth_config: super::auth::AdminAuthConfig) -> Router {
    super::auth::with_admin_auth(auth_config, admin_router(state))
}

/// Create a full admin router with both API and Web UI
///
/// This combines the REST API endpoints with the HTMX web interface.
/// The web UI is served at `/` and `/ui/*` paths.
///
/// # Arguments
///
/// * `state` - Admin state with metrics, indexing, and shutdown coordination
///
/// # Routes
///
/// API endpoints (require auth if configured):
/// - `/health`, `/metrics`, `/indexing`, etc.
///
/// Web UI (no auth required):
/// - `/` - Dashboard
/// - `/ui/config` - Configuration page
/// - `/ui/health` - Health status page
/// - `/ui/indexing` - Indexing status page
pub fn admin_router_full(state: AdminState) -> Router {
    admin_router(state).merge(super::web::web_router())
}

/// Create a full admin router with authentication
///
/// Same as `admin_router_full` but with API key authentication enabled.
/// Note: Web UI pages are NOT authenticated, only API endpoints are.
pub fn admin_router_full_with_auth(state: AdminState, auth_config: super::auth::AdminAuthConfig) -> Router {
    // Web UI routes don't need auth (they call authenticated API endpoints)
    let web_routes = super::web::web_router();
    // API routes with auth
    let api_routes = super::auth::with_admin_auth(auth_config, admin_router(state));
    // Merge them - web routes first so they take precedence for root path
    web_routes.merge(api_routes)
}
