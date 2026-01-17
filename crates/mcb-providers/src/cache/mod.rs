//! Cache Provider Implementations
//!
//! Provides caching backends for embedding and search result caching.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | [`NullCacheProvider`] | Testing | No-op stub for testing |
//! | [`MokaCacheProvider`] | Local | In-memory cache (high performance) |
//! | [`RedisCacheProvider`] | Distributed | Redis-backed for multi-instance |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `NullCacheProvider` for unit tests
//! - **Single Instance**: Use `MokaCacheProvider` for high performance
//! - **Multi Instance**: Use `RedisCacheProvider` for distributed caching

#[cfg(feature = "cache-moka")]
pub mod moka;
pub mod null;
#[cfg(feature = "cache-redis")]
pub mod redis;

// Re-export for convenience
#[cfg(feature = "cache-moka")]
pub use moka::MokaCacheProvider;
pub use null::NullCacheProvider;
#[cfg(feature = "cache-redis")]
pub use redis::RedisCacheProvider;

// Re-export domain types used by cache providers
pub use mcb_application::ports::providers::cache::{CacheEntryConfig, CacheStats};
