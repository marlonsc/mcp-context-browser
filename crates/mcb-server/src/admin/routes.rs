//! Admin API routes
//!
//! Route definitions for the admin API endpoints.

use axum::{
    routing::{get, patch, post},
    Router,
};

use super::config_handlers::{get_config, reload_config, update_config_section};
use super::handlers::{
    extended_health_check, get_indexing_status, get_metrics, health_check, liveness_check,
    readiness_check, shutdown, AdminState,
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
