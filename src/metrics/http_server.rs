//! HTTP REST API server for metrics endpoints
//!
//! Provides REST API endpoints on port 3001 for:
//! - Health checks
//! - System metrics (CPU, memory, disk, network)
//! - Performance metrics (queries, cache)
//! - Status information

use crate::core::error::{Error, Result};
use crate::metrics::{global_metrics_collector, SystemMetricsCollector};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

/// Comprehensive metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub timestamp: u64,
    pub cpu: crate::metrics::CpuMetrics,
    pub memory: crate::metrics::MemoryMetrics,
    pub disk: crate::metrics::DiskMetrics,
    pub network: crate::metrics::NetworkMetrics,
    pub process: crate::metrics::ProcessMetrics,
    pub query_performance: crate::metrics::QueryPerformanceMetrics,
    pub cache: crate::metrics::CacheMetrics,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub timestamp: u64,
    pub service: String,
    pub version: String,
    pub uptime: u64,
    pub pid: u32,
    pub status: String,
}

/// Status response with health thresholds
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub timestamp: u64,
    pub service: String,
    pub version: String,
    pub uptime: u64,
    pub pid: u32,
    pub health: HealthStatus,
    pub metrics: StatusMetrics,
}

/// Health status for each component
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub cpu: String,      // "healthy", "warning", "critical"
    pub memory: String,   // "healthy", "warning", "critical"
    pub disk: String,     // "healthy", "warning", "critical"
}

/// Key metrics for status overview
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusMetrics {
    pub cpu: f32,
    pub memory: f32,
    pub disk: f32,
    pub queries: u64,
    pub avg_latency: f64,
    pub cache_hit_rate: f64,
}

/// HTTP API server state
pub struct MetricsApiServer {
    port: u16,
    system_collector: Arc<Mutex<SystemMetricsCollector>>,
    start_time: std::time::Instant,
}

impl MetricsApiServer {
    /// Create a new metrics API server
    pub fn new(port: u16) -> Self {
        Self {
            port,
            system_collector: Arc::new(Mutex::new(SystemMetricsCollector::new())),
            start_time: std::time::Instant::now(),
        }
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<()> {
        let app = self.create_router();

        let addr = format!("0.0.0.0:{}", self.port);
        println!("🚀 Starting Metrics API server on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| Error::internal(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::internal(format!("HTTP server error: {}", e)))?;

        Ok(())
    }

    /// Create the Axum router with all endpoints
    fn create_router(&self) -> Router {
        let state = MetricsServerState {
            system_collector: Arc::clone(&self.system_collector),
            start_time: self.start_time,
        };

        Router::new()
            .route("/api/health", get(Self::health_handler))
            .route("/api/context/metrics", get(Self::comprehensive_metrics_handler))
            .route("/api/context/status", get(Self::status_handler))
            .route("/api/context/metrics/cpu", get(Self::cpu_metrics_handler))
            .route("/api/context/metrics/memory", get(Self::memory_metrics_handler))
            .route("/api/context/metrics/disk", get(Self::disk_metrics_handler))
            .route("/api/context/metrics/network", get(Self::network_metrics_handler))
            .route("/api/context/metrics/queries", get(Self::query_metrics_handler))
            .route("/api/context/metrics/cache", get(Self::cache_metrics_handler))
            .route("/api/context/metrics/process", get(Self::process_metrics_handler))
            .layer(CorsLayer::permissive())
            .with_state(state)
    }

    /// Health check endpoint
    async fn health_handler(State(state): State<MetricsServerState>) -> Json<HealthResponse> {
        let uptime = state.start_time.elapsed().as_secs();
        let pid = std::process::id();

        Json(HealthResponse {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            service: "mcp-context-browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime,
            pid,
            status: "healthy".to_string(),
        })
    }

    /// Comprehensive metrics endpoint
    async fn comprehensive_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<ComprehensiveMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;

        let (cpu, memory, disk, network, process) = system_collector
            .collect_all_metrics()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let performance_collector = global_metrics_collector();
        let performance_metrics = performance_collector.lock().await;

        let query_performance = performance_metrics.get_query_performance();
        let cache = performance_metrics.get_cache_metrics();

        Ok(Json(ComprehensiveMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu,
            memory,
            disk,
            network,
            process,
            query_performance,
            cache,
        }))
    }

    /// Status endpoint with health thresholds
    async fn status_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<StatusResponse>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;

        let (cpu, memory, disk, _, process) = system_collector
            .collect_all_metrics()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let performance_collector = global_metrics_collector();
        let performance_metrics = performance_collector.lock().await;

        let query_performance = performance_metrics.get_query_performance();
        let cache = performance_metrics.get_cache_metrics();

        // Health thresholds
        let cpu_health = if cpu.usage < 80.0 { "healthy" } else if cpu.usage < 90.0 { "warning" } else { "critical" };
        let memory_health = if memory.usage_percent < 80.0 { "healthy" } else if memory.usage_percent < 90.0 { "warning" } else { "critical" };
        let disk_health = if disk.usage_percent < 85.0 { "healthy" } else if disk.usage_percent < 95.0 { "warning" } else { "critical" };

        let uptime = state.start_time.elapsed().as_secs();

        Ok(Json(StatusResponse {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            service: "mcp-context-browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime,
            pid: process.pid,
            health: HealthStatus {
                cpu: cpu_health.to_string(),
                memory: memory_health.to_string(),
                disk: disk_health.to_string(),
            },
            metrics: StatusMetrics {
                cpu: cpu.usage,
                memory: memory.usage_percent,
                disk: disk.usage_percent,
                queries: query_performance.total_queries,
                avg_latency: query_performance.average_latency,
                cache_hit_rate: cache.hit_rate,
            },
        }))
    }

    /// Individual metrics endpoints
    async fn cpu_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::CpuMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        system_collector
            .collect_cpu_metrics()
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn memory_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::MemoryMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        system_collector
            .collect_memory_metrics()
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn disk_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::DiskMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        system_collector
            .collect_disk_metrics()
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn network_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::NetworkMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        system_collector
            .collect_network_metrics()
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn process_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::ProcessMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        system_collector
            .collect_process_metrics()
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn query_metrics_handler(
        State(_state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::QueryPerformanceMetrics>, StatusCode> {
        let performance_collector = global_metrics_collector();
        let performance_metrics = performance_collector.lock().await;
        Ok(Json(performance_metrics.get_query_performance()))
    }

    async fn cache_metrics_handler(
        State(_state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::CacheMetrics>, StatusCode> {
        let performance_collector = global_metrics_collector();
        let performance_metrics = performance_collector.lock().await;
        Ok(Json(performance_metrics.get_cache_metrics()))
    }
}

/// Server state for dependency injection
#[derive(Clone)]
struct MetricsServerState {
    system_collector: Arc<Mutex<SystemMetricsCollector>>,
    start_time: std::time::Instant,
}