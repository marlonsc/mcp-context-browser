//! HTTP REST API server for metrics endpoints
//!
//! Provides REST API endpoints on port 3001 for:
//! - Health checks
//! - System metrics (CPU, memory, disk, network)
//! - Performance metrics (queries, cache)

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::infrastructure::cache::{get_global_cache_manager, CacheStats};
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::rate_limit::RateLimiter;
// Rate limiting middleware will be added later

use crate::infrastructure::metrics::{
    system::SystemMetricsCollectorInterface, CacheMetrics, CpuMetrics, MemoryMetrics,
};
use crate::server::mcp_server::PerformanceMetricsInterface;

/// Comprehensive metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub timestamp: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub query_performance: crate::admin::service::PerformanceMetricsData, // Updated to match interface
    pub cache: crate::infrastructure::metrics::CacheMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_limits: Option<crate::infrastructure::limits::ResourceStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_cache_stats: Option<CacheStats>,
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

/// HTTP API server state
pub struct MetricsApiServer {
    port: u16,
    system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    start_time: std::time::Instant,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
}

impl MetricsApiServer {
    /// Create a new metrics API server
    pub fn new(
        port: u16,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        Self::with_limits(port, system_collector, performance_metrics, None, None)
    }

    /// Create a new metrics API server with both rate limiting and resource limits
    pub fn with_limits(
        port: u16,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
        rate_limiter: Option<Arc<RateLimiter>>,
        resource_limits: Option<Arc<ResourceLimits>>,
    ) -> Self {
        Self {
            port,
            system_collector,
            performance_metrics,
            start_time: std::time::Instant::now(),
            _rate_limiter: rate_limiter,
            resource_limits,
        }
    }

    /// Start the HTTP server
    pub async fn start(self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.create_router();

        let addr = format!("0.0.0.0:{}", self.port);
        tracing::info!("🚀 Starting Metrics API server on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Create the Axum router with all endpoints
    fn create_router(&self) -> Router {
        let state = MetricsServerState {
            system_collector: Arc::clone(&self.system_collector),
            performance_metrics: Arc::clone(&self.performance_metrics),
            start_time: self.start_time,
            _rate_limiter: self._rate_limiter.clone(),
            resource_limits: self.resource_limits.clone(),
        };

        Router::new()
            .route("/api/health", get(Self::health_handler))
            .route(
                "/api/context/metrics",
                get(Self::comprehensive_metrics_handler),
            )
            .route("/api/context/metrics/cpu", get(Self::cpu_metrics_handler))
            .route(
                "/api/context/metrics/memory",
                get(Self::memory_metrics_handler),
            )
            .route(
                "/api/context/metrics/queries",
                get(Self::query_metrics_handler),
            )
            .route(
                "/api/context/metrics/cache",
                get(Self::cache_metrics_handler),
            )
            .route("/api/context/status", get(Self::status_handler))
            // .layer(axum::middleware::from_fn(request_validation_middleware))
            .layer(tower_http::cors::CorsLayer::permissive())
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
        let cpu = state
            .system_collector
            .collect_cpu_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let memory = state
            .system_collector
            .collect_memory_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let performance_data = state.performance_metrics.get_performance_metrics();

        // Convert to CacheMetrics for response compatibility
        let cache = CacheMetrics {
            hits: performance_data.successful_queries, // This is a bit of a stretch but we follow the data we have
            misses: performance_data.failed_queries,
            hit_rate: performance_data.cache_hit_rate,
            size: 0, // Interface doesn't expose size yet
        };

        // Get resource limits stats if available
        let resource_limits = if let Some(ref limits) = state.resource_limits {
            match limits.get_stats().await {
                Ok(stats) => Some(stats),
                Err(e) => {
                    tracing::warn!("Failed to collect resource limits stats: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Get cache stats if available
        let cache_stats = if let Some(cache_manager) = get_global_cache_manager() {
            Some(cache_manager.get_stats().await)
        } else {
            None
        };

        Ok(Json(ComprehensiveMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu,
            memory,
            query_performance: performance_data,
            cache,
            resource_limits,
            advanced_cache_stats: cache_stats,
        }))
    }

    /// Status endpoint with health thresholds
    async fn status_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        let cpu = state
            .system_collector
            .collect_cpu_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let memory = state
            .system_collector
            .collect_memory_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let performance = state.performance_metrics.get_performance_metrics();

        let uptime = state.start_time.elapsed().as_secs();

        // Health thresholds
        let cpu_health = if cpu.usage < 80.0 {
            "healthy"
        } else if cpu.usage < 90.0 {
            "warning"
        } else {
            "critical"
        };
        let memory_health = if memory.usage_percent < 80.0 {
            "healthy"
        } else if memory.usage_percent < 90.0 {
            "warning"
        } else {
            "critical"
        };

        let status = serde_json::json!({
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            "service": "mcp-context-browser",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": uptime,
            "pid": std::process::id(),
            "health": {
                "cpu": cpu_health,
                "memory": memory_health
            },
            "metrics": {
                "cpu": cpu.usage,
                "memory": memory.usage_percent,
                "queries": performance.total_queries,
                "avgLatency": performance.average_response_time_ms,
                "cacheHitRate": performance.cache_hit_rate
            }
        });

        Ok(Json(status))
    }

    /// Individual metrics endpoints
    async fn cpu_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<CpuMetrics>, StatusCode> {
        let metrics = state
            .system_collector
            .collect_cpu_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(metrics))
    }

    async fn memory_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<MemoryMetrics>, StatusCode> {
        let metrics = state
            .system_collector
            .collect_memory_metrics()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(metrics))
    }

    async fn query_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::admin::service::PerformanceMetricsData>, StatusCode> {
        Ok(Json(state.performance_metrics.get_performance_metrics()))
    }

    async fn cache_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<CacheMetrics>, StatusCode> {
        let performance = state.performance_metrics.get_performance_metrics();
        Ok(Json(CacheMetrics {
            hits: performance.successful_queries, // Approximate
            misses: performance.failed_queries,
            hit_rate: performance.cache_hit_rate,
            size: 0,
        }))
    }
}

/// Server state for dependency injection
#[derive(Clone)]
struct MetricsServerState {
    system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    start_time: std::time::Instant,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
}
