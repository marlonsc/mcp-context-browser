//! HTTP REST API server for metrics endpoints
//!
//! Provides REST API endpoints on port 3001 for:
//! - Health checks
//! - System metrics (CPU, memory, disk, network)
//! - Performance metrics (queries, cache)

use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::cache::{CacheStats, get_global_cache_manager};
use crate::core::limits::ResourceLimits;
use crate::core::rate_limit::RateLimiter;
// Rate limiting middleware will be added later
use crate::server::security::request_validation_middleware;

use crate::metrics::{PerformanceMetrics, SystemMetricsCollector};

/// Comprehensive metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub timestamp: u64,
    pub cpu: crate::metrics::CpuMetrics,
    pub memory: crate::metrics::MemoryMetrics,
    pub query_performance: crate::metrics::QueryPerformanceMetrics,
    pub cache: crate::metrics::CacheMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_limits: Option<crate::core::limits::ResourceStats>,
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
    system_collector: Arc<Mutex<SystemMetricsCollector>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    start_time: std::time::Instant,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
}

impl MetricsApiServer {
    /// Create a new metrics API server
    pub fn new(port: u16) -> Self {
        Self::with_rate_limiting(port, None)
    }

    /// Create a new metrics API server with rate limiting
    pub fn with_rate_limiting(port: u16, rate_limiter: Option<Arc<RateLimiter>>) -> Self {
        Self {
            port,
            system_collector: Arc::new(Mutex::new(SystemMetricsCollector::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            start_time: std::time::Instant::now(),
            _rate_limiter: rate_limiter,
            resource_limits: None,
        }
    }

    /// Create a new metrics API server with resource limits
    pub fn with_resource_limits(port: u16, resource_limits: Option<Arc<ResourceLimits>>) -> Self {
        Self {
            port,
            system_collector: Arc::new(Mutex::new(SystemMetricsCollector::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            start_time: std::time::Instant::now(),
            _rate_limiter: None,
            resource_limits,
        }
    }

    /// Create a new metrics API server with both rate limiting and resource limits
    pub fn with_limits(
        port: u16,
        rate_limiter: Option<Arc<RateLimiter>>,
        resource_limits: Option<Arc<ResourceLimits>>,
    ) -> Self {
        Self {
            port,
            system_collector: Arc::new(Mutex::new(SystemMetricsCollector::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            start_time: std::time::Instant::now(),
            _rate_limiter: rate_limiter,
            resource_limits,
        }
    }

    /// Start the HTTP server
    pub async fn start(self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.create_router();

        let addr = format!("0.0.0.0:{}", self.port);
        println!("🚀 Starting Metrics API server on http://{}", addr);

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
            .layer(axum::middleware::from_fn(request_validation_middleware))
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
        let mut system_collector = state.system_collector.lock().await;
        let performance_metrics = state.performance_metrics.lock().await;

        let cpu = system_collector.collect_cpu_metrics();
        let memory = system_collector.collect_memory_metrics();

        let query_performance = performance_metrics.get_query_performance();
        let cache = performance_metrics.get_cache_metrics();

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
            query_performance,
            cache,
            resource_limits,
            advanced_cache_stats: cache_stats,
        }))
    }

    /// Status endpoint with health thresholds
    async fn status_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        let performance_metrics = state.performance_metrics.lock().await;

        let cpu = system_collector.collect_cpu_metrics();
        let memory = system_collector.collect_memory_metrics();

        let query_performance = performance_metrics.get_query_performance();
        let cache = performance_metrics.get_cache_metrics();

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
                "queries": query_performance.total_queries,
                "avgLatency": query_performance.average_latency,
                "cacheHitRate": cache.hit_rate
            }
        });

        Ok(Json(status))
    }

    /// Individual metrics endpoints
    async fn cpu_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::CpuMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        Ok(Json(system_collector.collect_cpu_metrics()))
    }

    async fn memory_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::MemoryMetrics>, StatusCode> {
        let mut system_collector = state.system_collector.lock().await;
        Ok(Json(system_collector.collect_memory_metrics()))
    }

    async fn query_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::QueryPerformanceMetrics>, StatusCode> {
        let performance_metrics = state.performance_metrics.lock().await;
        Ok(Json(performance_metrics.get_query_performance()))
    }

    async fn cache_metrics_handler(
        State(state): State<MetricsServerState>,
    ) -> Result<Json<crate::metrics::CacheMetrics>, StatusCode> {
        let performance_metrics = state.performance_metrics.lock().await;
        Ok(Json(performance_metrics.get_cache_metrics()))
    }
}

/// Server state for dependency injection
#[derive(Clone)]
struct MetricsServerState {
    system_collector: Arc<Mutex<SystemMetricsCollector>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    start_time: std::time::Instant,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
}
