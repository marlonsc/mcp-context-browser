//! Admin request handlers
//!
//! HTTP handlers for admin API endpoints including health checks,
//! performance metrics, indexing status, and runtime configuration management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mcb_domain::ports::admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse,
    IndexingOperationsInterface, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_infrastructure::config::watcher::ConfigWatcher;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::info;

use super::config::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};

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
            Json(ShutdownResponse {
                initiated: false,
                message: "Shutdown coordinator not available".to_string(),
                timeout_secs: 0,
            }),
        );
    };

    // Check if already shutting down
    if coordinator.is_shutting_down() {
        return (
            StatusCode::CONFLICT,
            Json(ShutdownResponse {
                initiated: false,
                message: "Shutdown already in progress".to_string(),
                timeout_secs: state.shutdown_timeout_secs,
            }),
        );
    }

    let timeout_secs = request.timeout_secs.unwrap_or(state.shutdown_timeout_secs);

    if request.immediate {
        info!("Immediate shutdown requested");
        coordinator.signal_shutdown();
        return (
            StatusCode::OK,
            Json(ShutdownResponse {
                initiated: true,
                message: "Immediate shutdown initiated".to_string(),
                timeout_secs: 0,
            }),
        );
    }

    info!(timeout_secs = timeout_secs, "Graceful shutdown requested");

    // Spawn shutdown task to handle the graceful shutdown process
    let coord = Arc::clone(coordinator);
    tokio::spawn(async move {
        // Wait for graceful period
        tokio::time::sleep(Duration::from_secs(timeout_secs)).await;
        coord.signal_shutdown();
    });

    (
        StatusCode::OK,
        Json(ShutdownResponse {
            initiated: true,
            message: format!(
                "Graceful shutdown initiated, server will stop in {} seconds",
                timeout_secs
            ),
            timeout_secs,
        }),
    )
}

/// Extended health check with dependency status
///
/// Returns detailed health information including the status of
/// all service dependencies (embedding provider, vector store, cache).
pub async fn extended_health_check(State(state): State<AdminState>) -> impl IntoResponse {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Build dependency health checks
    // Note: In a full implementation, these would actually ping the services
    // For now, we report based on whether the system is functioning
    let mut dependencies = Vec::new();

    // Embedding provider health (inferred from metrics)
    dependencies.push(DependencyHealthCheck {
        name: "embedding_provider".to_string(),
        status: if metrics.total_queries > 0 && metrics.failed_queries == 0 {
            DependencyHealth::Healthy
        } else if metrics.failed_queries > 0 {
            DependencyHealth::Degraded
        } else {
            DependencyHealth::Unknown
        },
        message: Some(format!(
            "Total queries: {}, Failed: {}",
            metrics.total_queries, metrics.failed_queries
        )),
        latency_ms: Some(metrics.average_response_time_ms as u64),
        last_check: now,
    });

    // Vector store health (inferred from operations)
    dependencies.push(DependencyHealthCheck {
        name: "vector_store".to_string(),
        status: DependencyHealth::Healthy, // Assume healthy if server is running
        message: Some(format!("Active indexing operations: {}", operations.len())),
        latency_ms: None,
        last_check: now,
    });

    // Cache health (inferred from cache hit rate)
    dependencies.push(DependencyHealthCheck {
        name: "cache".to_string(),
        status: if metrics.cache_hit_rate > 0.0 {
            DependencyHealth::Healthy
        } else {
            DependencyHealth::Unknown
        },
        message: Some(format!("Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0)),
        latency_ms: None,
        last_check: now,
    });

    // Calculate overall dependencies status
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

/// Calculate overall health status from individual dependency checks
fn calculate_overall_health(dependencies: &[DependencyHealthCheck]) -> DependencyHealth {
    let mut unhealthy_count = 0;
    let mut degraded_count = 0;

    for dep in dependencies {
        match dep.status {
            DependencyHealth::Unhealthy => unhealthy_count += 1,
            DependencyHealth::Degraded => degraded_count += 1,
            _ => {}
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

// ============================================================================
// Configuration Management Endpoints
// ============================================================================

/// Get current configuration (sanitized)
///
/// Returns the current configuration with sensitive fields removed.
/// API keys, secrets, and passwords are not exposed.
pub async fn get_config(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigResponse {
                success: false,
                config: SanitizedConfig::default(),
                config_path: None,
                last_reload: None,
            }),
        );
    };

    let config = watcher.get_config().await;
    let sanitized = SanitizedConfig::from_app_config(&config);

    (
        StatusCode::OK,
        Json(ConfigResponse {
            success: true,
            config: sanitized,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            last_reload: Some(chrono::Utc::now().to_rfc3339()),
        }),
    )
}

/// Reload configuration from file
///
/// Triggers a manual configuration reload. The new configuration
/// will be validated before being applied.
pub async fn reload_config(State(state): State<AdminState>) -> impl IntoResponse {
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigReloadResponse::watcher_unavailable()),
        );
    };

    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (StatusCode::OK, Json(ConfigReloadResponse::success(sanitized)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigReloadResponse::failure(format!(
                "Failed to reload configuration: {}",
                e
            ))),
        ),
    }
}

/// Update a specific configuration section
///
/// Updates a configuration section by merging the provided values
/// with the existing configuration, then writing to the config file
/// and triggering a reload.
///
/// Valid sections: server, logging, cache, metrics, limits, resilience
pub async fn update_config_section(
    State(state): State<AdminState>,
    Path(section): Path<String>,
    Json(request): Json<ConfigSectionUpdateRequest>,
) -> impl IntoResponse {
    use super::config::is_valid_section;

    // Check if section is valid
    if !is_valid_section(&section) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ConfigSectionUpdateResponse::invalid_section(&section)),
        );
    }

    // Check if watcher is available
    let Some(watcher) = &state.config_watcher else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigSectionUpdateResponse::watcher_unavailable(&section)),
        );
    };

    // Get config path
    let Some(config_path) = &state.config_path else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                "Configuration file path not available",
            )),
        );
    };

    // Read current config file
    let config_content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to read configuration file: {}", e),
                )),
            );
        }
    };

    // Parse as TOML
    let mut config_value: toml::Value = match toml::from_str(&config_content) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to parse configuration file: {}", e),
                )),
            );
        }
    };

    // Update the section
    if let Some(table) = config_value.as_table_mut() {
        // Convert JSON value to TOML value
        let toml_value = match json_to_toml(&request.values) {
            Some(v) => v,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ConfigSectionUpdateResponse::failure(
                        &section,
                        "Invalid configuration value format",
                    )),
                );
            }
        };

        // Merge with existing section or create new
        if let Some(existing) = table.get_mut(&section) {
            if let (Some(existing_table), Some(new_table)) =
                (existing.as_table_mut(), toml_value.as_table())
            {
                for (key, value) in new_table {
                    existing_table.insert(key.clone(), value.clone());
                }
            } else {
                table.insert(section.clone(), toml_value);
            }
        } else {
            table.insert(section.clone(), toml_value);
        }
    }

    // Write back to file
    let updated_content = match toml::to_string_pretty(&config_value) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConfigSectionUpdateResponse::failure(
                    &section,
                    format!("Failed to serialize configuration: {}", e),
                )),
            );
        }
    };

    if let Err(e) = std::fs::write(config_path, updated_content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                format!("Failed to write configuration file: {}", e),
            )),
        );
    }

    // Reload configuration
    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (
                StatusCode::OK,
                Json(ConfigSectionUpdateResponse::success(&section, sanitized)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConfigSectionUpdateResponse::failure(
                &section,
                format!("Configuration updated but reload failed: {}", e),
            )),
        ),
    }
}

/// Convert a JSON value to a TOML value
fn json_to_toml(json: &serde_json::Value) -> Option<toml::Value> {
    match json {
        serde_json::Value::Null => Some(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(toml::Value::Integer)
            .or_else(|| n.as_f64().map(toml::Value::Float)),
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Option<Vec<toml::Value>> = arr.iter().map(json_to_toml).collect();
            toml_arr.map(toml::Value::Array)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Some(toml::Value::Table(table))
        }
    }
}
