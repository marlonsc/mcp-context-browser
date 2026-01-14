//! Data types for admin service operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::server::admin::models::{IndexingConfig, ProviderInfo, SecurityConfig};

/// Configuration data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationData {
    /// Collection of providers items
    pub providers: Vec<ProviderInfo>,
    /// Indexing
    pub indexing: IndexingConfig,
    /// Security
    pub security: SecurityConfig,
    /// Metrics
    pub metrics: MetricsConfigData,
    /// Cache
    pub cache: CacheConfigData,
    /// Database
    pub database: DatabaseConfigData,
}

/// Metrics configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfigData {
    /// Enabled
    pub enabled: bool,
    /// Collection Interval
    pub collection_interval: u64,
    /// Retention Days
    pub retention_days: u32,
}

/// Cache configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfigData {
    /// Enabled
    pub enabled: bool,
    /// Max Size
    pub max_size: u64,
    /// Ttl Seconds
    pub ttl_seconds: u64,
}

/// Database configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigData {
    /// Url
    pub url: String,
    /// Pool Size
    pub pool_size: u32,
    /// Connection Timeout
    pub connection_timeout: u64,
}

/// Configuration update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdateResult {
    /// Success
    pub success: bool,
    /// Collection of changes_applied items
    pub changes_applied: Vec<String>,
    /// Requires Restart
    pub requires_restart: bool,
    /// Collection of validation_warnings items
    pub validation_warnings: Vec<String>,
}

/// Configuration change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChange {
    /// Id
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// User
    pub user: String,
    /// Path
    pub path: String,
    /// Optional old_value value
    pub old_value: Option<serde_json::Value>,
    /// New Value
    pub new_value: serde_json::Value,
    /// Change Type
    pub change_type: String,
}

/// Log filter for querying logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    /// Optional level value
    pub level: Option<String>,
    /// Optional module value
    pub module: Option<String>,
    /// Optional message_contains value
    pub message_contains: Option<String>,
    /// Optional start_time value
    pub start_time: Option<DateTime<Utc>>,
    /// Optional end_time value
    pub end_time: Option<DateTime<Utc>>,
    /// Optional limit value
    pub limit: Option<usize>,
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Level
    pub level: String,
    /// Module
    pub module: String,
    /// Message
    pub message: String,
    /// Target
    pub target: String,
    /// Optional file value
    pub file: Option<String>,
    /// Optional line value
    pub line: Option<u32>,
}

/// Log entries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntries {
    /// Collection of entries items
    pub entries: Vec<LogEntry>,
    /// Total Count
    pub total_count: u64,
    /// Has More
    pub has_more: bool,
}

/// Log export format
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Supported log export formats
pub enum LogExportFormat {
    /// Export logs in JSON format
    Json,
    /// Export logs in CSV format
    Csv,
    /// Export logs in plain text format
    PlainText,
}

/// Log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    /// Total Entries
    pub total_entries: u64,
    /// Map of entries_by_level entries
    pub entries_by_level: HashMap<String, u64>,
    /// Map of entries_by_module entries
    pub entries_by_module: HashMap<String, u64>,
    /// Optional oldest_entry value
    pub oldest_entry: Option<DateTime<Utc>>,
    /// Optional newest_entry value
    pub newest_entry: Option<DateTime<Utc>>,
}

/// Cache types for maintenance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    /// Clear all cache types
    All,
    /// Clear cached query results
    QueryResults,
    /// Clear cached embeddings
    Embeddings,
    /// Clear cached indexes
    Indexes,
}

/// Maintenance operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceResult {
    /// Success
    pub success: bool,
    /// Operation
    pub operation: String,
    /// Message
    pub message: String,
    /// Affected Items
    pub affected_items: u64,
    /// Execution Time Ms
    pub execution_time_ms: u64,
}

/// Data cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Older Than Days
    pub older_than_days: u32,
    /// Optional max_items_to_keep value
    pub max_items_to_keep: Option<u64>,
    /// Collection of cleanup_types items
    pub cleanup_types: Vec<String>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall Status
    pub overall_status: String,
    /// Collection of checks items
    pub checks: Vec<HealthCheck>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Duration Ms
    pub duration_ms: u64,
}

/// Individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Name
    pub name: String,
    /// Status
    pub status: String,
    /// Message
    pub message: String,
    /// Duration Ms
    pub duration_ms: u64,
    /// Optional details value
    pub details: Option<serde_json::Value>,
}

/// Connectivity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityTestResult {
    /// Provider Id
    pub provider_id: String,
    /// Success
    pub success: bool,
    /// Optional response_time_ms value
    pub response_time_ms: Option<u64>,
    /// Optional error_message value
    pub error_message: Option<String>,
    /// Details
    pub details: serde_json::Value,
}

/// Performance test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestConfig {
    /// Test Type
    pub test_type: String,
    /// Duration Seconds
    pub duration_seconds: u32,
    /// Concurrency
    pub concurrency: u32,
    /// Collection of queries items
    pub queries: Vec<String>,
}

/// Performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// Test Id
    pub test_id: String,
    /// Test Type
    pub test_type: String,
    /// Duration Seconds
    pub duration_seconds: u32,
    /// Total Requests
    pub total_requests: u64,
    /// Successful Requests
    pub successful_requests: u64,
    /// Failed Requests
    pub failed_requests: u64,
    /// Average Response Time Ms
    pub average_response_time_ms: f64,
    /// P95 Response Time Ms
    pub p95_response_time_ms: f64,
    /// P99 Response Time Ms
    pub p99_response_time_ms: f64,
    /// Throughput Rps
    pub throughput_rps: f64,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Name
    pub name: String,
    /// Include Data
    pub include_data: bool,
    /// Include Config
    pub include_config: bool,
    /// Compression
    pub compression: bool,
}

/// Backup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    /// Backup Id
    pub backup_id: String,
    /// Name
    pub name: String,
    /// Size Bytes
    pub size_bytes: u64,
    /// Created At
    pub created_at: DateTime<Utc>,
    /// Path
    pub path: String,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Id
    pub id: String,
    /// Name
    pub name: String,
    /// Created At
    pub created_at: DateTime<Utc>,
    /// Size Bytes
    pub size_bytes: u64,
    /// Status
    pub status: String,
}

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Success
    pub success: bool,
    /// Backup Id
    pub backup_id: String,
    /// Restored At
    pub restored_at: DateTime<Utc>,
    /// Items Restored
    pub items_restored: u64,
    /// Optional rollback_id value
    pub rollback_id: Option<String>,
    /// Message
    pub message: String,
    /// Execution Time Ms
    pub execution_time_ms: u64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Version
    pub version: String,
    /// Uptime
    pub uptime: u64,
    /// Pid
    pub pid: u32,
}

/// Indexing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    /// Is Indexing
    pub is_indexing: bool,
    /// Total Documents
    pub total_documents: u64,
    /// Indexed Documents
    pub indexed_documents: u64,
    /// Failed Documents
    pub failed_documents: u64,
    /// Optional current_file value
    pub current_file: Option<String>,
    /// Optional start_time value
    pub start_time: Option<u64>,
    /// Optional estimated_completion value
    pub estimated_completion: Option<u64>,
}

// Re-export PerformanceMetricsData from domain port for backward compatibility
pub use crate::domain::ports::admin::PerformanceMetricsData;

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// System Info
    pub system_info: SystemInfo,
    /// Active Providers
    pub active_providers: usize,
    /// Total Providers
    pub total_providers: usize,
    /// Active Indexes
    pub active_indexes: usize,
    /// Total Documents
    pub total_documents: u64,
    /// Cpu Usage
    pub cpu_usage: f64,
    /// Memory Usage
    pub memory_usage: f64,
    /// Performance
    pub performance: PerformanceMetricsData,
}

/// Search results returned from admin search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Query
    pub query: String,
    /// Collection of results items
    pub results: Vec<SearchResultItem>,
    /// Total
    pub total: usize,
    /// Took Ms
    pub took_ms: u64,
}

/// Individual search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// Id
    pub id: String,
    /// Content
    pub content: String,
    /// File Path
    pub file_path: String,
    /// Score
    pub score: f64,
}

// === Subsystem Control Types (ADR-007) ===

/// Type of subsystem in the MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemType {
    /// Embedding provider (OpenAI, Ollama, etc.)
    Embedding,
    /// Vector database provider (Milvus, EdgeVec, etc.)
    VectorStore,
    /// Search service
    Search,
    /// Indexing service
    Indexing,
    /// Cache manager
    Cache,
    /// Metrics collector
    Metrics,
    /// Background daemon
    Daemon,
    /// HTTP transport
    HttpTransport,
}

/// Current status of a subsystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemStatus {
    /// Subsystem is running normally
    Running,
    /// Subsystem is stopped
    Stopped,
    /// Subsystem encountered an error
    Error,
    /// Subsystem is starting up
    Starting,
    /// Subsystem is paused
    Paused,
    /// Subsystem status is unknown
    Unknown,
}

/// Runtime metrics for a subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in megabytes
    pub memory_mb: u64,
    /// Requests processed per second
    pub requests_per_sec: f64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
}

/// Comprehensive information about a subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemInfo {
    /// Unique identifier for this subsystem instance
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of subsystem
    pub subsystem_type: SubsystemType,
    /// Current operational status
    pub status: SubsystemStatus,
    /// Health status from last check
    pub health: HealthCheck,
    /// Current configuration
    pub config: serde_json::Value,
    /// Runtime metrics
    pub metrics: SubsystemMetrics,
}

/// Signal types that can be sent to subsystems
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemSignal {
    /// Restart the subsystem
    Restart,
    /// Reload configuration without restart
    Reload,
    /// Pause the subsystem
    Pause,
    /// Resume a paused subsystem
    Resume,
    /// Apply new configuration
    Configure(serde_json::Value),
}

/// Result of sending a signal to a subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    /// Whether the signal was successfully sent
    pub success: bool,
    /// ID of the subsystem that received the signal
    pub subsystem_id: String,
    /// The signal that was sent
    pub signal: String,
    /// Human-readable message about the result
    pub message: String,
}

/// Information about a registered HTTP route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// Unique identifier for this route
    pub id: String,
    /// URL path pattern (e.g., "/api/health")
    pub path: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Handler name or description
    pub handler: String,
    /// Whether authentication is required
    pub auth_required: bool,
    /// Rate limit in requests per minute (None = no limit)
    pub rate_limit: Option<u32>,
}

/// Result of configuration persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPersistResult {
    /// Whether the save was successful
    pub success: bool,
    /// Path where config was saved
    pub path: String,
    /// Any warnings during save
    pub warnings: Vec<String>,
}

/// Difference between runtime and file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    /// Whether there are any differences
    pub has_changes: bool,
    /// Changes in runtime but not in file
    pub runtime_only: HashMap<String, serde_json::Value>,
    /// Changes in file but not in runtime
    pub file_only: HashMap<String, serde_json::Value>,
}

/// Admin service errors
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    /// Error from MCP server operations
    #[error("MCP server error: {0}")]
    McpServerError(String),

    /// Configuration-related error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Database operation error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<crate::domain::error::Error> for AdminError {
    fn from(err: crate::domain::error::Error) -> Self {
        AdminError::McpServerError(err.to_string())
    }
}
