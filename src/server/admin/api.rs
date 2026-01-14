//! Admin API integration

use axum::Router;
use std::sync::Arc;

use super::{models::AdminState, routes::create_admin_router, AdminApi, AdminConfig};

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

    /// Create the admin router with auth handler access
    pub fn create_router_with_auth(&self, auth_handler: Arc<crate::server::auth::AuthHandler>) -> Result<Router, Box<dyn std::error::Error>> {
        let admin_api = Arc::new(AdminApi::new(self.config.clone()));
        let admin_service = self.mcp_server.admin_service();

        // Get AuthService from auth handler
        let auth_service: Arc<dyn crate::infrastructure::auth::AuthServiceInterface> = Arc::new((*auth_handler.auth_service()).clone());

        // Initialize web interface and templates
        let web_interface = super::web::WebInterface::new()?;
        let templates = web_interface.templates();

        // Create activity logger for tracking system events
        let activity_logger = Arc::new(super::service::helpers::activity::ActivityLogger::new());
        // Start listening to system events
        activity_logger.start_listening(self.mcp_server.event_bus.clone());

        let state = AdminState {
            admin_api,
            admin_service,
            auth_service,
            mcp_server: Arc::clone(&self.mcp_server),
            templates,
            recovery_manager: None, // Will be set during Phase 8 integration
            event_bus: self.mcp_server.event_bus.clone(),
            activity_logger,
        };

        let api_router = create_admin_router(state.clone());
        let web_router = web_interface.routes(state);

        Ok(Router::new().merge(api_router).merge(web_router))
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
