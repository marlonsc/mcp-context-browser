//! Admin API routes configuration

use axum::{middleware, routing::{get, post, put, delete}, Router};
use tower_http::cors::CorsLayer;

use crate::admin::{
    auth::{auth_middleware, login_handler},
    handlers::{
        add_provider_handler, get_config_handler, get_status_handler, index_operation_handler,
        list_indexes_handler, list_providers_handler, remove_provider_handler,
        search_handler, update_config_handler,
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