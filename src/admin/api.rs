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
    pub fn new(config: AdminConfig, mcp_server: crate::server::McpServer) -> Self {
        Self {
            config,
            mcp_server: Arc::new(mcp_server),
        }
    }

    /// Create the admin router
    pub fn create_router(&self) -> Router {
        let admin_api = Arc::new(AdminApi::new(self.config.clone()));
        let admin_service = self.mcp_server.admin_service();
        
        // Initialize web interface and templates
        let web_interface = crate::admin::web::WebInterface::new()
            .expect("Failed to initialize web interface templates");
        let templates = web_interface.templates();

        let state = AdminState {
            admin_api,
            admin_service,
            mcp_server: Arc::clone(&self.mcp_server),
            templates,
        };

        let api_router = create_admin_router(state.clone());
        let web_router = web_interface.routes(state);

        Router::new()
            .merge(api_router)
            .merge(web_router)
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
