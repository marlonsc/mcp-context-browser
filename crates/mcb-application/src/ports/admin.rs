//! Admin Service Domain Ports
//!
//! Defines the port interfaces for admin and monitoring services.
//! These traits break the circular dependency where infrastructure/di
//! previously imported from server layer.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Performance Metrics Types
// ============================================================================

/// Performance metrics data
///
/// This type is defined in domain to allow the trait to be used
/// without circular dependencies on server layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsData {
    /// Total Queries
    pub total_queries: u64,
    /// Successful Queries
    pub successful_queries: u64,
    /// Failed Queries
    pub failed_queries: u64,
    /// Average Response Time Ms
    pub average_response_time_ms: f64,
    /// Cache Hit Rate
    pub cache_hit_rate: f64,
    /// Active Connections
    pub active_connections: u32,
    /// Uptime Seconds
    pub uptime_seconds: u64,
}

// ============================================================================
// Performance Metrics Interface
// ============================================================================

/// Real-time performance metrics tracking interface
///
/// Domain port for tracking server performance metrics including
/// queries, response times, cache hits, and active connections.
///
/// # Example
///
/// ```no_run
/// use mcb_application::ports::admin::PerformanceMetricsInterface;
/// use std::sync::Arc;
///
/// fn record_metrics(metrics: Arc<dyn PerformanceMetricsInterface>) {
///     // Record a successful query with 50ms response time (cache miss)
///     metrics.record_query(50, true, false);
///
///     // Track active connections
///     metrics.update_active_connections(1);  // connection opened
///     metrics.update_active_connections(-1); // connection closed
///
///     // Get current metrics snapshot
///     let stats = metrics.get_performance_metrics();
///     println!("Uptime: {}s, Queries: {}", stats.uptime_seconds, stats.total_queries);
/// }
/// ```
pub trait PerformanceMetricsInterface: Send + Sync {
    /// Get server uptime in seconds
    fn uptime_secs(&self) -> u64;

    /// Record a query with its metrics
    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool);

    /// Update active connection count (positive to add, negative to remove)
    fn update_active_connections(&self, delta: i64);

    /// Get current performance metrics snapshot
    fn get_performance_metrics(&self) -> PerformanceMetricsData;
}

// ============================================================================
// Indexing Operations Types
// ============================================================================

/// Tracks ongoing indexing operations
#[derive(Debug, Clone)]
pub struct IndexingOperation {
    /// Operation ID
    pub id: String,
    /// Collection being indexed
    pub collection: String,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Operation start timestamp (Unix timestamp)
    pub start_timestamp: u64,
}

// ============================================================================
// Indexing Operations Interface
// ============================================================================

/// Interface for indexing operations tracking
///
/// Domain port for tracking ongoing indexing operations in the MCP server.
///
/// # Example
///
/// ```no_run
/// use mcb_application::ports::admin::IndexingOperationsInterface;
/// use std::sync::Arc;
///
/// fn show_operations(tracker: Arc<dyn IndexingOperationsInterface>) {
///     // Get all active indexing operations
///     let operations = tracker.get_operations();
///     for (id, op) in operations {
///         println!("Operation {}: {}/{} files in {}",
///             id, op.processed_files, op.total_files, op.collection);
///     }
/// }
/// ```
pub trait IndexingOperationsInterface: Send + Sync {
    /// Get the map of ongoing indexing operations
    fn get_operations(&self) -> HashMap<String, IndexingOperation>;
}

// ============================================================================
// Service Lifecycle Management
// ============================================================================

/// Health status for a service dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DependencyHealth {
    /// Service is healthy and responsive
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy or unresponsive
    Unhealthy,
    /// Health status is unknown (not checked)
    #[default]
    Unknown,
}

/// Detailed health check result for a service dependency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyHealthCheck {
    /// Name of the dependency
    pub name: String,
    /// Health status
    pub status: DependencyHealth,
    /// Optional message providing more details
    pub message: Option<String>,
    /// Latency in milliseconds (if applicable)
    pub latency_ms: Option<u64>,
    /// Last check timestamp (Unix timestamp)
    pub last_check: u64,
}

/// Port service lifecycle state (simplified, Copy-able version)
///
/// This is a simplified version of ServiceState for port interfaces.
/// For domain events with failure reasons, use `mcb_domain::events::ServiceState`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PortServiceState {
    /// Service is starting up
    Starting,
    /// Service is running normally
    Running,
    /// Service is shutting down
    Stopping,
    /// Service is stopped
    #[default]
    Stopped,
}

/// Lifecycle management interface for services
///
/// Domain port for managing service lifecycle including start, stop,
/// restart, and health checks.
///
/// # Example
///
/// ```no_run
/// use mcb_application::ports::admin::{LifecycleManaged, PortServiceState};
/// use std::sync::Arc;
///
/// async fn check_service(service: Arc<dyn LifecycleManaged>) -> mcb_domain::Result<()> {
///     // Check service state
///     if service.state() == PortServiceState::Running {
///         // Perform health check
///         let health = service.health_check().await;
///         println!("Service health: {:?}", health.status);
///     }
///
///     // Graceful shutdown
///     service.stop().await?;
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    /// Get the service name
    fn name(&self) -> &str;

    /// Get the current service state
    fn state(&self) -> PortServiceState;

    /// Start the service
    async fn start(&self) -> mcb_domain::error::Result<()>;

    /// Stop the service gracefully
    async fn stop(&self) -> mcb_domain::error::Result<()>;

    /// Restart the service
    async fn restart(&self) -> mcb_domain::error::Result<()> {
        self.stop().await?;
        self.start().await
    }

    /// Perform a health check on this service
    async fn health_check(&self) -> DependencyHealthCheck;
}

// ============================================================================
// Shutdown Coordination
// ============================================================================

/// Shutdown coordinator for managing graceful server shutdown
///
/// This interface allows components to signal and check shutdown status.
/// The actual signaling mechanism is implementation-specific (e.g., broadcast channels).
///
/// # Example
///
/// ```no_run
/// use mcb_application::ports::admin::ShutdownCoordinator;
/// use std::sync::Arc;
///
/// fn handle_shutdown(coordinator: Arc<dyn ShutdownCoordinator>) {
///     // Check if shutdown has been requested
///     if coordinator.is_shutting_down() {
///         println!("Shutdown in progress, stopping work");
///     }
///
///     // To trigger shutdown (e.g., from admin API)
///     coordinator.signal_shutdown();
/// }
/// ```
pub trait ShutdownCoordinator: Send + Sync {
    /// Signal all components to begin shutdown
    fn signal_shutdown(&self);

    /// Check if shutdown has been signaled
    fn is_shutting_down(&self) -> bool;
}

/// Extended health check response including dependency status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    /// Overall server status
    pub status: &'static str,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Number of active indexing operations
    pub active_indexing_operations: usize,
    /// Health checks for dependencies
    pub dependencies: Vec<DependencyHealthCheck>,
    /// Overall dependencies health status
    pub dependencies_status: DependencyHealth,
}
