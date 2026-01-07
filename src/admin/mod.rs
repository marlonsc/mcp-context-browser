//! Web Administration Interface
//!
//! Provides a web-based administration interface for MCP Context Browser
//! running on the same port as the metrics server (3001).

pub mod api;
pub mod auth;
pub mod handlers;
pub mod models;
pub mod routes;

pub use api::AdminApi;
pub use routes::create_admin_router;

/// Admin API server configuration
#[derive(Debug, Clone)]
pub struct AdminConfig {
    /// Enable admin interface
    pub enabled: bool,
    /// Admin username
    pub username: String,
    /// Admin password hash
    pub password_hash: String,
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
            password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/JhJzGfjQwRrRrLrO".to_string(), // "admin"
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