//! Basic web interface for admin operations

use axum::{Router, routing::get};

/// Web interface manager
pub struct WebInterface;

impl WebInterface {
    /// Create a new web interface manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    /// Get the web routes
    pub fn routes(&self) -> Router {
        Router::new().route("/", get(|| async { "MCP Context Browser Admin Interface" }))
    }
}
