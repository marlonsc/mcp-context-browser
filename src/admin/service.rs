//! Admin service layer - SOLID principles implementation
//!
//! This service provides a clean interface to access system data
//! following SOLID principles and dependency injection.

use std::sync::Arc;
use async_trait::async_trait;

/// Core admin service trait
#[async_trait]
pub trait AdminService: Send + Sync {
    /// Get system information
    async fn get_system_info(&self) -> Result<SystemInfo, AdminError>;

    /// Get all registered providers
    async fn get_providers(&self) -> Result<Vec<ProviderInfo>, AdminError>;

    /// Get indexing status
    async fn get_indexing_status(&self) -> Result<IndexingStatus, AdminError>;

    /// Get performance metrics
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, AdminError>;

    /// Get dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData, AdminError>;
}

/// Concrete implementation of AdminService
pub struct AdminServiceImpl {
    mcp_server: Arc<crate::server::McpServer>,
}

impl AdminServiceImpl {
    /// Create new admin service with dependency injection
    pub fn new(mcp_server: Arc<crate::server::McpServer>) -> Self {
        Self { mcp_server }
    }
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn get_system_info(&self) -> Result<SystemInfo, AdminError> {
        let info = self.mcp_server.get_system_info();
        Ok(SystemInfo {
            version: info.version,
            uptime: info.uptime,
            pid: info.pid,
        })
    }

    async fn get_providers(&self) -> Result<Vec<ProviderInfo>, AdminError> {
        let providers = self.mcp_server.get_registered_providers();
        Ok(providers
            .into_iter()
            .map(|p| ProviderInfo {
                id: p.id,
                name: p.name,
                provider_type: p.provider_type,
                status: p.status,
                config: p.config,
            })
            .collect())
    }

    async fn get_indexing_status(&self) -> Result<IndexingStatus, AdminError> {
        let status = self.mcp_server.get_indexing_status();
        Ok(IndexingStatus {
            is_indexing: status.is_indexing,
            total_documents: status.total_documents,
            indexed_documents: status.indexed_documents,
            failed_documents: status.failed_documents,
            current_file: status.current_file,
            start_time: status.start_time,
            estimated_completion: status.estimated_completion,
        })
    }

    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, AdminError> {
        let metrics = self.mcp_server.get_performance_metrics();
        Ok(PerformanceMetrics {
            total_queries: metrics.total_queries,
            successful_queries: metrics.successful_queries,
            failed_queries: metrics.failed_queries,
            average_response_time_ms: metrics.average_response_time_ms,
            cache_hit_rate: metrics.cache_hit_rate,
            active_connections: metrics.active_connections,
            uptime_seconds: metrics.uptime_seconds,
        })
    }

    async fn get_dashboard_data(&self) -> Result<DashboardData, AdminError> {
        let system_info = self.get_system_info().await?;
        let providers = self.get_providers().await?;
        let indexing = self.get_indexing_status().await?;
        let performance = self.get_performance_metrics().await?;

        let active_providers = providers.iter().filter(|p| p.status == "active").count();
        let active_indexes = if indexing.is_indexing { 0 } else { 1 };

        Ok(DashboardData {
            system_info,
            active_providers,
            total_providers: providers.len(),
            active_indexes,
            total_documents: indexing.indexed_documents,
            cpu_usage: 15.0, // TODO: Get from system metrics
            memory_usage: 45.0, // TODO: Get from system metrics
            performance,
        })
    }
}

/// Data structures for admin service

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub uptime: u64,
    pub pid: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub status: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub total_documents: u64,
    pub indexed_documents: u64,
    pub failed_documents: u64,
    pub current_file: Option<String>,
    pub start_time: Option<u64>,
    pub estimated_completion: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct DashboardData {
    pub system_info: SystemInfo,
    pub active_providers: usize,
    pub total_providers: usize,
    pub active_indexes: usize,
    pub total_documents: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub performance: PerformanceMetrics,
}

/// Admin service errors
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("MCP server error: {0}")]
    McpServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<crate::core::error::Error> for AdminError {
    fn from(err: crate::core::error::Error) -> Self {
        AdminError::McpServerError(err.to_string())
    }
}