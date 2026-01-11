//! Provider Routing System
//!
//! This module provides a comprehensive provider routing system using established libraries
//! and patterns, following SOLID principles with proper separation of concerns.
//!
//! ## Architecture
//!
//! The routing system is composed of several specialized modules:
//!
//! - `health`: Health monitoring using established health check patterns
//! - `circuit_breaker`: Circuit breaker implementation for resilience
//! - `metrics`: Metrics collection using prometheus and metrics crates
//! - `cost_tracker`: Cost tracking with thread-safe operations
//! - `failover`: Failover management with configurable strategies
//! - `router`: Main router coordinating all components via dependency injection
//!
//! ## Key Features
//!
//! - **Health Monitoring**: Automatic provider health checks with configurable thresholds
//! - **Circuit Breaker**: Resilience pattern preventing cascade failures
//! - **Cost Tracking**: Multi-provider cost optimization and budget enforcement
//! - **Failover**: Automatic failover with priority-based and round-robin strategies
//! - **Metrics**: Comprehensive observability using established crates
//! - **Rate Limiting**: Governor-based rate limiting for API protection
//!
//! ## Usage
//!
//! ```rust,no_run
//! use mcp_context_browser::adapters::providers::routing::{ProviderRouter, ProviderContext};
//! use mcp_context_browser::infrastructure::di::registry::ProviderRegistry;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = Arc::new(ProviderRegistry::new());
//! let router = ProviderRouter::with_defaults(registry).await?;
//!
//! let context = ProviderContext {
//!     operation_type: "embedding".to_string(),
//!     cost_sensitivity: 0.8,
//!     ..Default::default()
//! };
//!
//! let provider_id = router.select_embedding_provider(&context).await?;
//! let provider = router.get_embedding_provider(&context).await?;
//! # Ok(())
//! # }
//! ```

pub mod circuit_breaker;
pub mod cost_tracker;
pub mod failover;
pub mod health;
pub mod metrics;
pub mod router;

// Re-export main types for convenience
pub use router::{
    ContextualStrategy, LoadLevel, ProviderContext, ProviderRouter, ProviderRouterDeps,
    ProviderSelectionStrategy, RouterStatistics,
};

pub use health::{
    HealthCheckResult, HealthMonitor, ProviderHealth, ProviderHealthChecker, ProviderHealthStatus,
    RealProviderHealthChecker,
};

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerMetrics, CircuitBreakerState,
};

pub use metrics::{MetricsSummary, ProviderMetricsCollector};

pub use cost_tracker::{CostTracker, CostTrackerConfig, ProviderCost, UsageMetrics};

pub use failover::{
    FailoverContext, FailoverManager, FailoverStrategy, PriorityBasedStrategy, RoundRobinStrategy,
};
