//! Admin request handlers
//!
//! HTTP handlers for admin API endpoints including health checks,
//! performance metrics, indexing status, and runtime configuration management.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mcb_application::ports::admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, PerformanceMetricsData, PerformanceMetricsInterface,
    ShutdownCoordinator,
};
use mcb_infrastructure::config::watcher::ConfigWatcher;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::info;

/// Admin handler state containing shared service references
#[derive(Clone)]
pub struct AdminState {
    /// Performance metrics tracker
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations tracker
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Configuration watcher for hot-reload support
    pub config_watcher: Option<Arc<ConfigWatcher>>,
    /// Configuration file path (for updates)
    pub config_path: Option<PathBuf>,
    /// Shutdown coordinator for graceful shutdown
    pub shutdown_coordinator: Option<Arc<dyn ShutdownCoordinator>>,
    /// Default shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

/// Health check response for admin API
#[derive(Serialize)]
pub struct AdminHealthResponse {
    /// Server status
    pub status: &'static str,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Number of active indexing operations
    pub active_indexing_operations: usize,
}

/// Health check endpoint
pub async fn health_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}

/// Get performance metrics endpoint
pub async fn get_metrics(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    Json(metrics)
}

/// Indexing status response
#[derive(Serialize)]
pub struct IndexingStatusResponse {
    /// Whether indexing is currently active
    pub is_indexing: bool,
    /// Number of active operations
    pub active_operations: usize,
    /// Details of each operation
    pub operations: Vec<IndexingOperationStatus>,
}

/// Individual indexing operation status
#[derive(Serialize)]
pub struct IndexingOperationStatus {
    /// Operation ID
    pub id: String,
    /// Collection being indexed
    pub collection: String,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Progress as percentage
    pub progress_percent: f32,
    /// Files processed
    pub processed_files: usize,
    /// Total files
    pub total_files: usize,
}

/// Get indexing status endpoint
pub async fn get_indexing_status(State(state): State<AdminState>) -> impl IntoResponse {
    let operations = state.indexing.get_operations();

    let operation_statuses: Vec<IndexingOperationStatus> = operations
        .values()
        .map(|op| {
            let progress = if op.total_files > 0 {
                (op.processed_files as f32 / op.total_files as f32) * 100.0
            } else {
                0.0
            };

            IndexingOperationStatus {
                id: op.id.clone(),
                collection: op.collection.clone(),
                current_file: op.current_file.clone(),
                progress_percent: progress,
                processed_files: op.processed_files,
                total_files: op.total_files,
            }
        })
        .collect();

    Json(IndexingStatusResponse {
        is_indexing: !operation_statuses.is_empty(),
        active_operations: operation_statuses.len(),
        operations: operation_statuses,
    })
}

/// Readiness check endpoint (for k8s/docker health checks)
pub async fn readiness_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();

    // Consider ready if server has been up for at least 1 second
    if metrics.uptime_seconds >= 1 {
        (StatusCode::OK, Json(serde_json::json!({ "ready": true })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "ready": false })),
        )
    }
}

/// Liveness check endpoint (for k8s/docker health checks)
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "alive": true })))
}

// ============================================================================
// Service Control Endpoints
// ============================================================================

/// Shutdown request body
#[derive(serde::Deserialize, Default)]
pub struct ShutdownRequest {
    /// Custom timeout in seconds (optional, uses default if not provided)
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Immediate shutdown without graceful period (default: false)
    #[serde(default)]
    pub immediate: bool,
}

/// Shutdown response
#[derive(Serialize)]
pub struct ShutdownResponse {
    /// Whether shutdown was initiated
    pub initiated: bool,
    /// Message describing the shutdown status
    pub message: String,
    /// Timeout being used for graceful shutdown
    pub timeout_secs: u64,
}

impl ShutdownResponse {
    fn error(message: impl Into<String>, timeout: u64) -> Self {
        Self {
            initiated: false,
            message: message.into(),
            timeout_secs: timeout,
        }
    }

    fn success(message: impl Into<String>, timeout: u64) -> Self {
        Self {
            initiated: true,
            message: message.into(),
            timeout_secs: timeout,
        }
    }
}

/// Initiate graceful server shutdown
///
/// Signals all components to begin shutdown. The server will attempt
/// to complete in-flight requests before terminating.
///
/// # Request Body
///
/// - `timeout_secs`: Optional custom timeout (default: 30s)
/// - `immediate`: Skip graceful shutdown period (default: false)
pub async fn shutdown(
    State(state): State<AdminState>,
    Json(request): Json<ShutdownRequest>,
) -> impl IntoResponse {
    let Some(coordinator) = &state.shutdown_coordinator else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ShutdownResponse::error(
                "Shutdown coordinator not available",
                0,
            )),
        );
    };

    if coordinator.is_shutting_down() {
        return (
            StatusCode::CONFLICT,
            Json(ShutdownResponse::error(
                "Shutdown already in progress",
                state.shutdown_timeout_secs,
            )),
        );
    }

    let timeout_secs = request.timeout_secs.unwrap_or(state.shutdown_timeout_secs);

    if request.immediate {
        info!("Immediate shutdown requested");
        coordinator.signal_shutdown();
        return (
            StatusCode::OK,
            Json(ShutdownResponse::success("Immediate shutdown initiated", 0)),
        );
    }

    info!(timeout_secs = timeout_secs, "Graceful shutdown requested");
    spawn_graceful_shutdown(Arc::clone(coordinator), timeout_secs);

    let msg = format!(
        "Graceful shutdown initiated, server will stop in {} seconds",
        timeout_secs
    );
    (
        StatusCode::OK,
        Json(ShutdownResponse::success(msg, timeout_secs)),
    )
}

fn spawn_graceful_shutdown(coord: Arc<dyn ShutdownCoordinator>, timeout: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        coord.signal_shutdown();
    });
}

/// Extended health check with dependency status
///
/// Returns detailed health information including the status of
/// all service dependencies (embedding provider, vector store, cache).
pub async fn extended_health_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();
    let now = current_timestamp();

    let dependencies = build_dependency_checks(&metrics, &operations, now);
    let dependencies_status = calculate_overall_health(&dependencies);

    let response = ExtendedHealthResponse {
        status: if dependencies_status == DependencyHealth::Unhealthy {
            "degraded"
        } else {
            "healthy"
        },
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
        dependencies,
        dependencies_status,
    };

    Json(response)
}

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Build dependency health checks from metrics and operations
fn build_dependency_checks(
    metrics: &PerformanceMetricsData,
    operations: &std::collections::HashMap<String, IndexingOperation>,
    now: u64,
) -> Vec<DependencyHealthCheck> {
    vec![
        build_embedding_health(metrics, now),
        build_vector_store_health(operations, now),
        build_cache_health(metrics, now),
    ]
}

/// Build embedding provider health check
fn build_embedding_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "embedding_provider".to_string(),
        status: match (metrics.total_queries, metrics.failed_queries) {
            (total, 0) if total > 0 => DependencyHealth::Healthy,
            (_, failed) if failed > 0 => DependencyHealth::Degraded,
            _ => DependencyHealth::Unknown,
        },
        message: Some(format!(
            "Total queries: {}, Failed: {}",
            metrics.total_queries, metrics.failed_queries
        )),
        latency_ms: Some(metrics.average_response_time_ms as u64),
        last_check: now,
    }
}

/// Build vector store health check
fn build_vector_store_health(
    operations: &std::collections::HashMap<String, IndexingOperation>,
    now: u64,
) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "vector_store".to_string(),
        status: DependencyHealth::Healthy,
        message: Some(format!("Active indexing operations: {}", operations.len())),
        latency_ms: None,
        last_check: now,
    }
}

/// Build cache health check
fn build_cache_health(metrics: &PerformanceMetricsData, now: u64) -> DependencyHealthCheck {
    DependencyHealthCheck {
        name: "cache".to_string(),
        status: if metrics.cache_hit_rate > 0.0 {
            DependencyHealth::Healthy
        } else {
            DependencyHealth::Unknown
        },
        message: Some(format!(
            "Cache hit rate: {:.1}%",
            metrics.cache_hit_rate * 100.0
        )),
        latency_ms: None,
        last_check: now,
    }
}

/// Calculate overall health status from individual dependency checks
fn calculate_overall_health(dependencies: &[DependencyHealthCheck]) -> DependencyHealth {
    let mut unhealthy_count = 0;
    let mut degraded_count = 0;

    for dep in dependencies {
        match dep.status {
            DependencyHealth::Unhealthy => unhealthy_count += 1,
            DependencyHealth::Degraded => degraded_count += 1,
            DependencyHealth::Healthy | DependencyHealth::Unknown => {
                // Healthy/Unknown dependencies don't need counting
            }
        }
    }

    if unhealthy_count > 0 {
        DependencyHealth::Unhealthy
    } else if degraded_count > 0 {
        DependencyHealth::Degraded
    } else {
        DependencyHealth::Healthy
    }
}
