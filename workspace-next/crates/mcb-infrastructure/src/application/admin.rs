//! Admin Service
//!
//! Application-level admin service that coordinates admin operations.

use async_trait::async_trait;
use shaku::Interface;
use mcb_domain::error::Result;

/// Admin service interface for application-level admin operations
#[async_trait]
pub trait AdminService: Interface + Send + Sync {
    /// Get server health status
    async fn health_check(&self) -> Result<AdminHealthResult>;

    /// Get server metrics
    async fn get_metrics(&self) -> Result<ServerMetrics>;
}

/// Admin health check result (domain service return type)
///
/// Note: Distinct from `AdminHealthResponse` in mcb-server which is an HTTP DTO.
#[derive(Debug, Clone)]
pub struct AdminHealthResult {
    pub status: String,
    pub uptime_seconds: u64,
}

/// Server metrics response
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    pub total_requests: u64,
    pub active_connections: u32,
}

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = AdminService)]
pub struct NullAdminService;

impl NullAdminService {
    pub fn new() -> Self { Self }
}

impl Default for NullAdminService {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl AdminService for NullAdminService {
    async fn health_check(&self) -> Result<AdminHealthResult> {
        Ok(AdminHealthResult {
            status: "healthy".to_string(),
            uptime_seconds: 0,
        })
    }

    async fn get_metrics(&self) -> Result<ServerMetrics> {
        Ok(ServerMetrics {
            total_requests: 0,
            active_connections: 0,
        })
    }
}
