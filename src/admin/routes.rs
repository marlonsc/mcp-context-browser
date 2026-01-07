//! Admin API routes configuration

use axum::{middleware, routing::{get, post, put, delete}, Router};
use tower_http::cors::CorsLayer;

use crate::admin::{
    auth::{auth_middleware, login_handler},
    handlers::{
        add_provider_handler, clear_cache_handler, create_backup_handler, export_logs_handler,
        get_config_handler, get_configuration_handler, get_configuration_history_handler,
        get_log_stats_handler, get_logs_handler, get_status_handler, health_check_handler,
        index_operation_handler, list_backups_handler, list_indexes_handler,
        list_providers_handler, performance_test_handler, rebuild_index_handler,
        remove_provider_handler, restart_provider_handler, restore_backup_handler,
        search_handler, test_connectivity_handler, update_config_handler,
        update_configuration_handler, validate_configuration_handler,
        cleanup_data_handler,
    },
    models::AdminState,
};

/// Create the admin API router
pub fn create_admin_router(state: AdminState) -> Router {
    Router::new()
        // Public routes (no auth required)
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/status", get(get_status_handler))

        // Protected routes (auth required)
        .route("/admin/config", get(get_config_handler))
        .route("/admin/config", put(update_config_handler))

        // Dynamic Configuration Management
        .route("/admin/configuration", get(get_configuration_handler))
        .route("/admin/configuration", put(update_configuration_handler))
        .route("/admin/configuration/validate", post(validate_configuration_handler))
        .route("/admin/configuration/history", get(get_configuration_history_handler))

        // Logging System
        .route("/admin/logs", get(get_logs_handler))
        .route("/admin/logs/export", get(export_logs_handler))
        .route("/admin/logs/stats", get(get_log_stats_handler))

        // Maintenance Operations
        .route("/admin/maintenance/cache/:cache_type", post(clear_cache_handler))
        .route("/admin/maintenance/providers/:provider_id/restart", post(restart_provider_handler))
        .route("/admin/maintenance/indexes/:index_id/rebuild", post(rebuild_index_handler))
        .route("/admin/maintenance/cleanup", post(cleanup_data_handler))

        // Diagnostic Operations
        .route("/admin/diagnostic/health", get(health_check_handler))
        .route("/admin/diagnostic/connectivity/:provider_id", post(test_connectivity_handler))
        .route("/admin/diagnostic/performance", post(performance_test_handler))

        // Data Management
        .route("/admin/backup", post(create_backup_handler))
        .route("/admin/backup", get(list_backups_handler))
        .route("/admin/backup/:backup_id/restore", post(restore_backup_handler))

        // Legacy Provider Management (keeping for backward compatibility)
        .route("/admin/providers", get(list_providers_handler))
        .route("/admin/providers", post(add_provider_handler))
        .route("/admin/providers/:provider_id", delete(remove_provider_handler))
        .route("/admin/indexes", get(list_indexes_handler))
        .route("/admin/indexes/:index_id/operations", post(index_operation_handler))
        .route("/admin/search", get(search_handler))

        // Apply authentication middleware to protected routes
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))

        // CORS support
        .layer(CorsLayer::permissive())

        // Shared state
        .with_state(state)
}