//! Data models for the admin API

use crate::infrastructure::utils::TimeUtils;
use serde::{Deserialize, Serialize};

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Token
    pub token: String,
    /// Expires At
    pub expires_at: u64,
    /// User
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    /// Username
    pub username: String,
    /// Role
    pub role: String,
}

/// Provider information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderInfo {
    /// Id
    pub id: String,
    /// Name
    pub name: String,
    /// Provider Type
    pub provider_type: String,
    /// Status
    pub status: String,
    /// Config
    pub config: serde_json::Value,
}

/// Provider configuration request
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfigRequest {
    /// Provider Type
    pub provider_type: String,
    /// Config
    pub config: serde_json::Value,
}

/// Index information
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Id
    pub id: String,
    /// Name
    pub name: String,
    /// Status
    pub status: String,
    /// Document Count
    pub document_count: u64,
    /// Created At
    pub created_at: u64,
    /// Updated At
    pub updated_at: u64,
}

/// Index operation request
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexOperationRequest {
    /// Operation
    pub operation: String, // "clear", "rebuild", "status"
}

/// System configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    /// Collection of providers items
    pub providers: Vec<ProviderInfo>,
    /// Indexing
    pub indexing: IndexingConfig,
    /// Security
    pub security: SecurityConfig,
    /// Metrics
    pub metrics: MetricsConfig,
}

/// Indexing configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexingConfig {
    /// Chunk Size
    pub chunk_size: usize,
    /// Chunk Overlap
    pub chunk_overlap: usize,
    /// Max File Size
    pub max_file_size: u64,
    /// Collection of supported_extensions items
    pub supported_extensions: Vec<String>,
    /// Collection of exclude_patterns items
    pub exclude_patterns: Vec<String>,
}

/// Security configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    /// Enable Auth
    pub enable_auth: bool,
    /// Rate Limiting
    pub rate_limiting: bool,
    /// Max Requests Per Minute
    pub max_requests_per_minute: u32,
}

/// Metrics configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricsConfig {
    /// Enabled
    pub enabled: bool,
    /// Collection Interval
    pub collection_interval: u64,
    /// Retention Days
    pub retention_days: u32,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Success
    pub success: bool,
    /// Optional data value
    pub data: Option<T>,
    /// Optional error value
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    /// Create a successful API response with data
    ///
    /// # Arguments
    /// * `data` - The response data to include
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: TimeUtils::now_unix_secs(),
        }
    }

    /// Create an error API response with message
    ///
    /// # Arguments
    /// * `error` - The error message to include
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: TimeUtils::now_unix_secs(),
        }
    }
}

/// Admin server state with proper dependency injection
#[derive(Clone)]
pub struct AdminState {
    /// Admin Api
    pub admin_api: std::sync::Arc<super::AdminApi>,
    /// Admin Service
    pub admin_service: std::sync::Arc<dyn super::service::AdminService>,
    /// Authentication Service from DI container
    pub auth_service: std::sync::Arc<dyn crate::infrastructure::auth::AuthServiceInterface>,
    /// Mcp Server
    pub mcp_server: std::sync::Arc<crate::server::McpServer>,
    /// Templates
    pub templates: std::sync::Arc<tera::Tera>,
    /// Recovery manager for automatic component restart
    pub recovery_manager: Option<crate::infrastructure::recovery::SharedRecoveryManager>,
    /// Event bus for system-wide event coordination
    pub event_bus: crate::infrastructure::events::SharedEventBusProvider,
    /// Activity logger for tracking system events
    pub activity_logger: std::sync::Arc<super::service::helpers::activity::ActivityLogger>,
}
