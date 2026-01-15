//! Admin Interface
//!
//! Administrative interfaces for system monitoring and management.
//! Uses domain ports to maintain Clean Architecture separation.

pub mod api;
pub mod auth;
pub mod config;
pub mod handlers;
pub mod models;
pub mod routes;

/// Admin API server for web-based administration
///
/// Provides REST endpoints for system monitoring, configuration management,
/// and administrative operations. Uses domain services through dependency injection.
pub struct AdminApiServer {
    // Will be implemented when admin ports are defined in domain layer
}

impl AdminApiServer {
    /// Create a new admin API server
    pub fn new() -> Self {
        Self {}
    }

    /// Create router with authentication
    pub fn create_router_with_auth(
        self,
        _auth_handler: crate::auth::AuthHandler,
    ) -> Result<axum::Router, Box<dyn std::error::Error>> {
        // Placeholder - will be implemented when admin domain ports are available
        Ok(axum::Router::new())
    }
}