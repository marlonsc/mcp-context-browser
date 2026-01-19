//! Admin API routes
//!
//! Route definitions for the admin API endpoints.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Authentication integration added in v0.1.2.

use rocket::{routes, Build, Rocket};
use std::sync::Arc;

use super::auth::AdminAuthConfig;
use super::config_handlers::{get_config, reload_config, update_config_section};
use super::handlers::{
    extended_health_check, get_cache_stats, get_indexing_status, get_metrics, health_check,
    liveness_check, readiness_check, shutdown, AdminState,
};
use super::lifecycle_handlers::{
    list_services, restart_service, services_health, start_service, stop_service,
};
use super::sse::events_stream;

/// Create the admin API rocket instance
///
/// Routes:
/// - GET /health - Health check with uptime and status
/// - GET /health/extended - Extended health check with dependency status
/// - GET /metrics - Performance metrics
/// - GET /indexing - Indexing operations status
/// - GET /ready - Kubernetes readiness probe (public)
/// - GET /live - Kubernetes liveness probe (public)
/// - POST /shutdown - Initiate graceful server shutdown (protected)
/// - GET /config - View current configuration (protected)
/// - POST /config/reload - Trigger configuration reload (protected)
/// - PATCH /config/:section - Update configuration section (protected)
/// - GET /events - SSE event stream for real-time updates
/// - GET /services - List registered services (protected)
/// - GET /services/health - Health check all services (protected)
/// - POST /services/:name/start - Start a service (protected)
/// - POST /services/:name/stop - Stop a service (protected)
/// - POST /services/:name/restart - Restart a service (protected)
/// - GET /cache/stats - Cache statistics (protected)
///
/// # Authentication
///
/// Protected endpoints require the `X-Admin-Key` header (or configured header name)
/// with a valid API key. Public endpoints (health probes) are exempt.
pub fn admin_rocket(state: AdminState, auth_config: Arc<AdminAuthConfig>) -> Rocket<Build> {
    rocket::build()
        .manage(state)
        .manage(auth_config)
        .mount(
            "/",
            routes![
                // Health and monitoring
                health_check,
                extended_health_check,
                get_metrics,
                get_indexing_status,
                readiness_check,
                liveness_check,
                // Service control
                shutdown,
                // Configuration management
                get_config,
                reload_config,
                update_config_section,
                // SSE event stream
                events_stream,
                // Service lifecycle management
                list_services,
                services_health,
                start_service,
                stop_service,
                restart_service,
                // Cache management
                get_cache_stats,
            ],
        )
}
