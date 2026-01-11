//! Admin API integration

use axum::Router;
use std::sync::Arc;

use crate::admin::{models::AdminState, routes::create_admin_router, AdminApi, AdminConfig};

/// Admin API server
pub struct AdminApiServer {
    config: AdminConfig,
    mcp_server: Arc<crate::server::McpServer>,
}

impl AdminApiServer {
    /// Create a new admin API server
    pub fn new(config: AdminConfig, mcp_server: Arc<crate::server::McpServer>) -> Self {
        Self { config, mcp_server }
    }

    /// Create the admin router
    pub fn create_router(&self) -> Router {
        let admin_api = Arc::new(AdminApi::new(self.config.clone()));
        let admin_service = self.mcp_server.admin_service();
        let state = AdminState {
            admin_api,
            admin_service,
            mcp_server: Arc::clone(&self.mcp_server),
        };

        create_admin_router(state)
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
