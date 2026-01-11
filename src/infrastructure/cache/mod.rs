//! Advanced distributed caching system with Redis
//!
//! Provides high-performance caching for embeddings, search results, and metadata
//! with intelligent TTL management and cache invalidation strategies.

mod config;
mod local;
mod manager;
mod operations;
mod redis;
mod stats;

// Re-export configuration types
pub use config::{
    CacheConfig, CacheEntry, CacheNamespaceConfig, CacheNamespacesConfig, CacheResult, CacheStats,
};

// Re-export implementation
pub use manager::CacheManager;

use crate::domain::error::Error;

/// Convert Redis errors to domain errors in the infrastructure layer
impl From<::redis::RedisError> for Error {
    fn from(err: ::redis::RedisError) -> Self {
        Self::Cache {
            message: err.to_string(),
        }
    }
}
