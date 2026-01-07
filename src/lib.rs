//! MCP Context Browser - A semantic code search server

pub mod chunking;
pub mod config;
pub mod core;
pub mod daemon;
pub mod di;
pub mod metrics;
pub mod providers;
pub mod server;
pub mod services;
pub mod snapshot;
pub mod sync;

// Re-export rate limiting system
pub use core::rate_limit::{RateLimitConfig, RateLimitKey, RateLimitResult, RateLimiter};

// Re-export resource limits system
pub use core::limits::{ResourceLimits, ResourceLimitsConfig, ResourceStats, ResourceViolation};

// Re-export advanced caching system
pub use core::cache::{CacheConfig, CacheManager, CacheResult, CacheStats};

// Re-export hybrid search system
pub use core::hybrid_search::{BM25Params, BM25Scorer, HybridSearchConfig, HybridSearchEngine};

// Re-export multi-provider strategy system
pub use providers::routing::{
    ProviderContext, ProviderRouter, ProviderSelectionStrategy, circuit_breaker::CircuitBreaker,
    metrics::ProviderMetricsCollector,
};
