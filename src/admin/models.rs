//! Data models for the admin API

use serde::{Deserialize, Serialize};

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: u64,
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}

/// Provider information
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub status: String,
    pub config: serde_json::Value,
}

/// Provider configuration request
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfigRequest {
    pub provider_type: String,
    pub config: serde_json::Value,
}

/// Index information
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub document_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Index operation request
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexOperationRequest {
    pub operation: String, // "clear", "rebuild", "status"
}

/// System configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub providers: Vec<ProviderInfo>,
    pub indexing: IndexingConfig,
    pub security: SecurityConfig,
    pub metrics: MetricsConfig,
}

/// Indexing configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_file_size: u64,
    pub supported_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// Security configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_auth: bool,
    pub rate_limiting: bool,
    pub max_requests_per_minute: u32,
}

/// Metrics configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval: u64,
    pub retention_days: u32,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Admin server state with proper dependency injection
#[derive(Clone)]
pub struct AdminState {
    pub admin_api: std::sync::Arc<super::AdminApi>,
    pub admin_service: std::sync::Arc<dyn super::service::AdminService>,
    pub mcp_server: std::sync::Arc<crate::server::McpServer>,
}