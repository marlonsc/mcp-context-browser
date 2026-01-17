//! Caching infrastructure with TTL and namespaces
//!
//! Provides caching configuration and wiring.
//! Cache provider implementations are in mcb-providers crate.
//! Types (CacheEntryConfig, CacheStats, CacheProvider) are in mcb-domain.

pub mod config;
pub mod provider;
pub mod queue;
