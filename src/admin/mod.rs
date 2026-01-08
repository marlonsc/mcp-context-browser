//! Enterprise Administration & Monitoring Platform
//!
//! Provides a comprehensive web-based administration, configuration, and monitoring
//! interface for MCP Context Browser with advanced features for enterprise deployments.
//!
//! ## Key Capabilities
//!
//! - **Dynamic Configuration**: Hot-reload configuration changes without service restart
//! - **Real-time Monitoring**: Live performance metrics, health checks, and alerting
//! - **Advanced Logging**: Structured log investigation with filters and analytics
//! - **Maintenance Tools**: Cache management, provider lifecycle, data operations
//! - **Diagnostic Suite**: Connectivity testing, performance profiling, system health
//! - **Data Management**: Backup/restore, data cleanup, storage optimization
//! - **API Integration**: RESTful APIs for automation and external tooling

pub mod api;
pub mod auth;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod service;
pub mod web;

pub use routes::create_admin_router;

/// Admin API server configuration
#[derive(Debug, Clone)]
pub struct AdminConfig {
    /// Enable admin interface
    pub enabled: bool,
    /// Admin username
    pub username: String,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// JWT expiration time in seconds
    pub jwt_expiration: u64,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            username: "admin".to_string(),
            jwt_secret: "default-jwt-secret-change-in-production".to_string(),
            jwt_expiration: 3600, // 1 hour
        }
    }
}

/// Admin API instance
pub struct AdminApi {
    config: AdminConfig,
}

impl AdminApi {
    /// Create a new admin API instance
    pub fn new(config: AdminConfig) -> Self {
        Self { config }
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
