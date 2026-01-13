//! HTTP REST API server for metrics endpoints
//!
//! Provides REST API endpoints on port 3001 for:
//! - Health checks
//! - System metrics (CPU, memory, disk, network)
//! - Performance metrics (queries, cache)
//!
//! Includes production middleware stack:
//! - Panic recovery (prevents handler panics from crashing server)
//! - Request IDs (trace requests across logs)
//! - Structured request/response logging
//! - Global request timeout
//! - Request body size limits (DoS prevention)
//! - Response compression

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::infrastructure::cache::{CacheStats, SharedCacheProvider};
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::rate_limit::RateLimiter;
use crate::infrastructure::service_helpers::UptimeTracker;
use crate::infrastructure::utils::TimeUtils;

use crate::infrastructure::metrics::{
    system::SystemMetricsCollectorInterface, CacheMetrics, CpuMetrics, MemoryMetrics,
};
use crate::server::metrics::PerformanceMetricsInterface;

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
    uptime: UptimeTracker,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
    cache_provider: Option<SharedCacheProvider>,
    external_router: Option<Router>,
    /// MCP protocol router (merged under /mcp/* path)
    mcp_router: Option<Router>,
}

impl MetricsApiServer {
    /// Create a new metrics API server
    pub fn new(
        port: u16,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        Self::with_limits(
            port,
            system_collector,
            performance_metrics,
            None,
            None,
            None,
        )
    }

    /// Create a new metrics API server with both rate limiting and resource limits
    pub fn with_limits(
        port: u16,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
        rate_limiter: Option<Arc<RateLimiter>>,
        resource_limits: Option<Arc<ResourceLimits>>,
        cache_provider: Option<SharedCacheProvider>,
    ) -> Self {
        Self {
            port,
            system_collector,
            performance_metrics,
            uptime: UptimeTracker::start(),
            _rate_limiter: rate_limiter,
            resource_limits,
            cache_provider,
            external_router: None,
            mcp_router: None,
        }
    }

    /// Add an external router to be merged with the metrics router (typically admin routes)
    pub fn with_external_router(mut self, router: Router) -> Self {
        self.external_router = Some(router);
        self
    }

    /// Add MCP protocol router to be merged under /mcp path
    ///
    /// This enables unified port architecture where MCP, Admin, and Metrics
    /// all serve from the same port (default 3001).
    pub fn with_mcp_router(mut self, router: Router) -> Self {
        self.mcp_router = Some(router);
        self
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
            uptime: self.uptime,
            _rate_limiter: self._rate_limiter.clone(),
            resource_limits: self.resource_limits.clone(),
            cache_provider: self.cache_provider.clone(),
        };

        // Production middleware stack
        // Layer order (outermost to innermost):
        // 1. CatchPanic - prevent handler panics from crashing server
        // 2. RequestId - trace requests across logs
        // 3. Trace - structured request/response logging
        // 4. Timeout - global request timeout
        // 5. BodyLimit - prevent DoS via large payloads
        // 6. Compression - reduce bandwidth
        // 7. CORS - cross-origin requests
        //
        // Note: Layers are applied in reverse order (last .layer() is innermost)
        let mut router = Router::new()
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
            .with_state(state)
            // Apply layers (last is innermost, first is outermost)
            .layer(tower_http::cors::CorsLayer::permissive())
            .layer(CompressionLayer::new())
            .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(30),
            ))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(CatchPanicLayer::new());

        // Merge external router (typically admin routes under /admin/*)
        if let Some(external) = self.external_router.clone() {
            router = router.merge(external);
        }

        // Merge MCP router (MCP protocol under /mcp/*)
        if let Some(mcp) = self.mcp_router.clone() {
            router = router.merge(mcp);
        }

        router
    }

    /// Health check endpoint
    async fn health_handler(State(state): State<MetricsServerState>) -> Json<HealthResponse> {
        let uptime = state.uptime.elapsed_secs();
        let pid = std::process::id();

        Json(HealthResponse {
            timestamp: TimeUtils::now_unix_millis() as u64,
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
        let cache_stats = if let Some(ref cache_provider) = state.cache_provider {
            cache_provider.get_stats("metadata").await.ok()
        } else {
            None
        };

        Ok(Json(ComprehensiveMetrics {
            timestamp: TimeUtils::now_unix_millis() as u64,
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

        let uptime = state.uptime.elapsed_secs();

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
            "timestamp": TimeUtils::now_unix_millis() as u64,
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
    uptime: UptimeTracker,
    _rate_limiter: Option<Arc<RateLimiter>>,
    resource_limits: Option<Arc<ResourceLimits>>,
    cache_provider: Option<SharedCacheProvider>,
}
