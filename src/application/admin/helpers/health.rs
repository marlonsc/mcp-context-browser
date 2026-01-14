//! Health monitoring helper module
//!
//! Provides functions for health checks, connectivity tests, and performance testing.
//!
//! # Helper Functions
//!
//! - [`determine_health_status`] - Determine status from usage and thresholds
//! - [`create_metric_health_check`] - Create HealthCheck from metric values

use super::runtime_config::{RuntimeConfig, RuntimeConfigDependencies};
use crate::application::admin::types::{
    AdminError, ConnectivityTestResult, HealthCheck, HealthCheckResult, ProviderInfo,
};
use crate::domain::ports::IndexingOperationsInterface;
use crate::infrastructure::cache::SharedCacheProvider;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::infrastructure::service_helpers::{SafeMetrics, TimedOperation};
use crate::infrastructure::utils::status;
use std::sync::Arc;

/// Determine health status from usage value and thresholds
///
/// Returns CRITICAL if value > unhealthy_threshold, DEGRADED if value > degraded_threshold,
/// otherwise HEALTHY.
#[inline]
fn determine_health_status(value: f32, degraded_threshold: f32, unhealthy_threshold: f32) -> String {
    if value > unhealthy_threshold {
        status::CRITICAL.to_string()
    } else if value > degraded_threshold {
        status::DEGRADED.to_string()
    } else {
        status::HEALTHY.to_string()
    }
}

/// Map provider status to health status
#[inline]
fn provider_status_to_health(provider_status: &str) -> String {
    if provider_status == status::ACTIVE {
        status::HEALTHY.to_string()
    } else if provider_status == status::DEGRADED {
        status::DEGRADED.to_string()
    } else {
        status::CRITICAL.to_string()
    }
}

/// Service dependencies for health checks
pub struct HealthCheckDependencies {
    /// System metrics collector
    pub system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    /// Provider list
    pub providers: Vec<ProviderInfo>,
    /// Indexing operations for real pending operation count
    pub indexing_operations: Option<Arc<dyn IndexingOperationsInterface>>,
    /// Cache provider for real cache statistics
    pub cache_provider: Option<SharedCacheProvider>,
}

/// Run comprehensive health check with injected services
pub async fn run_health_check(
    system_collector: &Arc<dyn SystemMetricsCollectorInterface>,
    providers: Vec<ProviderInfo>,
) -> Result<HealthCheckResult, AdminError> {
    run_health_check_with_services(HealthCheckDependencies {
        system_collector: Arc::clone(system_collector),
        providers,
        indexing_operations: None,
        cache_provider: None,
    })
    .await
}

/// Run comprehensive health check with full service dependencies
///
/// This is the preferred method - it uses real service data.
pub async fn run_health_check_with_services(
    deps: HealthCheckDependencies,
) -> Result<HealthCheckResult, AdminError> {
    let timer = TimedOperation::start();
    let mut checks = Vec::new();

    // Load runtime configuration with real service data
    let runtime_cfg = RuntimeConfig::load_with_services(RuntimeConfigDependencies {
        indexing_operations: deps.indexing_operations,
        cache_provider: deps.cache_provider,
    })
    .await
    .unwrap_or_else(|_| RuntimeConfig {
        indexing: Default::default(),
        cache: Default::default(),
        database: Default::default(),
        thresholds: Default::default(),
    });

    let system_collector = &deps.system_collector;
    let providers = deps.providers;

    // Get system thresholds from runtime config
    let thresholds = &runtime_cfg.thresholds;

    // Collect system metrics with safe defaults
    let cpu_metrics =
        SafeMetrics::collect(system_collector.collect_cpu_metrics(), Default::default()).await;
    let memory_metrics = SafeMetrics::collect(
        system_collector.collect_memory_metrics(),
        Default::default(),
    )
    .await;
    let disk_metrics =
        SafeMetrics::collect(system_collector.collect_disk_metrics(), Default::default()).await;
    let process_metrics = SafeMetrics::collect(
        system_collector.collect_process_metrics(),
        Default::default(),
    )
    .await;

    // Determine CPU health status based on dynamic thresholds
    let cpu_status = determine_health_status(
        cpu_metrics.usage,
        thresholds.cpu_degraded_percent as f32,
        thresholds.cpu_unhealthy_percent as f32,
    );

    checks.push(HealthCheck {
        name: "cpu".to_string(),
        status: cpu_status,
        message: format!("CPU usage at {:.1}%", cpu_metrics.usage),
        duration_ms: 5,
        details: Some(serde_json::json!({
            "usage_percent": cpu_metrics.usage,
            "cores": cpu_metrics.cores,
            "model": cpu_metrics.model,
            "speed_mhz": cpu_metrics.speed
        })),
    });

    // Determine memory health status based on dynamic thresholds
    let memory_status = determine_health_status(
        memory_metrics.usage_percent,
        thresholds.memory_degraded_percent as f32,
        thresholds.memory_unhealthy_percent as f32,
    );

    checks.push(HealthCheck {
        name: "memory".to_string(),
        status: memory_status,
        message: format!(
            "Memory usage at {:.1}% ({} MB / {} MB)",
            memory_metrics.usage_percent,
            memory_metrics.used / 1024 / 1024,
            memory_metrics.total / 1024 / 1024
        ),
        duration_ms: 5,
        details: Some(serde_json::json!({
            "usage_percent": memory_metrics.usage_percent,
            "used_bytes": memory_metrics.used,
            "total_bytes": memory_metrics.total,
            "free_bytes": memory_metrics.free
        })),
    });

    // Determine disk health status based on dynamic thresholds
    let disk_status = determine_health_status(
        disk_metrics.usage_percent,
        thresholds.disk_degraded_percent as f32,
        thresholds.disk_unhealthy_percent as f32,
    );

    checks.push(HealthCheck {
        name: "disk".to_string(),
        status: disk_status,
        message: format!("Disk usage at {:.1}%", disk_metrics.usage_percent),
        duration_ms: 5,
        details: Some(serde_json::json!({
            "usage_percent": disk_metrics.usage_percent,
            "used_bytes": disk_metrics.used,
            "total_bytes": disk_metrics.total,
            "available_bytes": disk_metrics.available
        })),
    });

    // Process metrics
    let memory_mb = process_metrics.memory / 1024 / 1024;
    checks.push(HealthCheck {
        name: "process".to_string(),
        status: "healthy".to_string(),
        message: format!(
            "Process using {:.1}% CPU and {} MB memory",
            process_metrics.cpu_percent, memory_mb
        ),
        duration_ms: 5,
        details: Some(serde_json::json!({
            "pid": process_metrics.pid,
            "cpu_usage_percent": process_metrics.cpu_percent,
            "memory_usage_mb": memory_mb,
            "memory_percent": process_metrics.memory_percent,
            "uptime_seconds": process_metrics.uptime
        })),
    });

    // Provider health checks
    for provider in providers {
        let provider_health = provider_status_to_health(&provider.status);

        checks.push(HealthCheck {
            name: format!("provider_{}", provider.id),
            status: provider_health,
            message: format!("Provider {} is {}", provider.name, provider.status),
            duration_ms: 10,
            details: Some(serde_json::json!({
                "provider_id": provider.id,
                "provider_name": provider.name,
                "provider_type": provider.provider_type,
                "status": provider.status,
                "config": provider.config
            })),
        });
    }

    // Subsystem health checks with real runtime values
    let indexing_timer = TimedOperation::start();
    checks.push(HealthCheck {
        name: "indexing".to_string(),
        status: if runtime_cfg.indexing.enabled {
            status::HEALTHY.to_string()
        } else {
            status::DEGRADED.to_string()
        },
        message: format!(
            "Indexing subsystem {} with {} pending operations",
            if runtime_cfg.indexing.enabled {
                "operational"
            } else {
                "disabled"
            },
            runtime_cfg.indexing.pending_operations
        ),
        duration_ms: indexing_timer.elapsed_ms(),
        details: Some(serde_json::json!({
            "status": if runtime_cfg.indexing.enabled { "operational" } else { "disabled" },
            "pending_operations": runtime_cfg.indexing.pending_operations,
            "last_index_time": runtime_cfg.indexing.last_index_time.to_rfc3339()
        })),
    });

    let cache_timer = TimedOperation::start();
    let cache_status = if runtime_cfg.cache.enabled {
        if runtime_cfg.cache.hit_rate >= thresholds.cache_hit_rate_degraded as f64 {
            status::HEALTHY.to_string()
        } else {
            status::DEGRADED.to_string()
        }
    } else {
        status::DEGRADED.to_string()
    };

    checks.push(HealthCheck {
        name: "cache".to_string(),
        status: cache_status,
        message: format!(
            "Cache subsystem operational: {} entries, {:.1}% hit rate",
            runtime_cfg.cache.entries_count,
            runtime_cfg.cache.hit_rate * 100.0
        ),
        duration_ms: cache_timer.elapsed_ms(),
        details: Some(serde_json::json!({
            "status": if runtime_cfg.cache.enabled { "operational" } else { "disabled" },
            "entries": runtime_cfg.cache.entries_count,
            "hit_rate": runtime_cfg.cache.hit_rate,
            "size_bytes": runtime_cfg.cache.size_bytes,
            "max_size_bytes": runtime_cfg.cache.max_size_bytes
        })),
    });

    let database_timer = TimedOperation::start();
    let db_utilization = (runtime_cfg.database.active_connections as f64
        / runtime_cfg.database.total_pool_size as f64)
        * 100.0;

    let db_status = if !runtime_cfg.database.connected
        || db_utilization > thresholds.db_pool_unhealthy_percent as f64
    {
        status::CRITICAL.to_string()
    } else if db_utilization > thresholds.db_pool_degraded_percent as f64 {
        status::DEGRADED.to_string()
    } else {
        status::HEALTHY.to_string()
    };

    checks.push(HealthCheck {
        name: "database".to_string(),
        status: db_status,
        message: format!(
            "Database connection pool: {} active, {} idle of {} total (utilization: {:.1}%)",
            runtime_cfg.database.active_connections,
            runtime_cfg.database.idle_connections,
            runtime_cfg.database.total_pool_size,
            db_utilization
        ),
        duration_ms: database_timer.elapsed_ms(),
        details: Some(serde_json::json!({
            "status": if runtime_cfg.database.connected { "connected" } else { "disconnected" },
            "active_connections": runtime_cfg.database.active_connections,
            "idle_connections": runtime_cfg.database.idle_connections,
            "total_pool_size": runtime_cfg.database.total_pool_size,
            "utilization_percent": db_utilization
        })),
    });

    // Determine overall status
    let unhealthy_count = checks.iter().filter(|c| c.status == "unhealthy").count();
    let degraded_count = checks.iter().filter(|c| c.status == "degraded").count();

    let overall_status = if unhealthy_count > 0 {
        "unhealthy"
    } else if degraded_count > 0 {
        "degraded"
    } else {
        "healthy"
    }
    .to_string();

    Ok(HealthCheckResult {
        overall_status,
        checks,
        timestamp: chrono::Utc::now(),
        duration_ms: timer.elapsed_ms(),
    })
}

/// Test connectivity to a specific provider
pub fn test_provider_connectivity(
    service_provider: &Arc<dyn ServiceProviderInterface>,
    provider_id: &str,
) -> Result<ConnectivityTestResult, AdminError> {
    let timer = TimedOperation::start();
    let (embedding_providers, vector_store_providers) = service_provider.list_providers();

    let is_embedding = embedding_providers.iter().any(|p| p == provider_id);
    let is_vector_store = vector_store_providers.iter().any(|p| p == provider_id);

    if !is_embedding && !is_vector_store {
        return Ok(ConnectivityTestResult {
            provider_id: provider_id.to_string(),
            success: false,
            response_time_ms: Some(timer.elapsed_ms()),
            error_message: Some(format!("Provider '{}' not found in registry", provider_id)),
            details: serde_json::json!({
                "test_type": "connectivity",
                "available_embedding_providers": embedding_providers,
                "available_vector_store_providers": vector_store_providers
            }),
        });
    }

    let provider_type = if is_embedding {
        "embedding"
    } else {
        "vector_store"
    };
    let response_time = timer.elapsed_ms();

    Ok(ConnectivityTestResult {
        provider_id: provider_id.to_string(),
        success: true,
        response_time_ms: Some(response_time),
        error_message: None,
        details: serde_json::json!({
            "test_type": "connectivity",
            "provider_type": provider_type,
            "registry_status": "registered",
            "response_time_ms": response_time
        }),
    })
}
