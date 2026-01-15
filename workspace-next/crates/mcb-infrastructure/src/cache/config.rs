//! Cache configuration structures
//!
//! Defines configuration structures for cache providers and settings.

use crate::constants::*;
use mcb_domain::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache entry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryConfig {
    /// Time to live for the cache entry
    pub ttl: Option<Duration>,
    /// Namespace for the cache entry
    pub namespace: Option<String>,
}

impl CacheEntryConfig {
    /// Create a new cache entry config with default TTL
    pub fn new() -> Self {
        Self {
            ttl: Some(Duration::from_secs(CACHE_DEFAULT_TTL_SECS)),
            namespace: None,
        }
    }

    /// Set the TTL for the cache entry
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set the namespace for the cache entry
    pub fn with_namespace<S: Into<String>>(mut self, namespace: S) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Get the effective TTL, falling back to default
    pub fn effective_ttl(&self) -> Duration {
        self.ttl.unwrap_or(Duration::from_secs(CACHE_DEFAULT_TTL_SECS))
    }

    /// Get the effective namespace, falling back to default
    pub fn effective_namespace(&self) -> String {
        self.namespace.clone().unwrap_or_else(|| "default".to_string())
    }
}

impl Default for CacheEntryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache entries
    pub entries: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Total bytes used by cache
    pub bytes_used: u64,
}

impl CacheStats {
    /// Create empty cache statistics
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            entries: 0,
            hit_rate: 0.0,
            bytes_used: 0,
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.update_hit_rate();
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.update_hit_rate();
    }

    /// Update the hit rate
    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache key utilities
pub struct CacheKey;

impl CacheKey {
    /// Create a namespaced cache key
    pub fn namespaced(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace, key)
    }

    /// Extract namespace from a namespaced key
    pub fn extract_namespace(key: &str) -> Option<&str> {
        key.split(':').next()
    }

    /// Extract the key part from a namespaced key
    pub fn extract_key(key: &str) -> &str {
        key.splitn(2, ':').nth(1).unwrap_or(key)
    }

    /// Validate cache key format
    pub fn validate_key(key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(Error::Configuration {
                message: "Cache key cannot be empty".to_string(),
                source: None,
            });
        }

        if key.len() > 250 {
            return Err(Error::Configuration {
                message: "Cache key too long (max 250 characters)".to_string(),
                source: None,
            });
        }

        // Check for invalid characters
        if key.chars().any(|c| c.is_control() || c == '\n' || c == '\r') {
            return Err(Error::Configuration {
                message: "Cache key contains invalid characters".to_string(),
                source: None,
            });
        }

        Ok(())
    }

    /// Sanitize a cache key by removing/replacing invalid characters
    pub fn sanitize_key(key: &str) -> String {
        key.chars()
            .map(|c| if c.is_control() || c == '\n' || c == '\r' { '_' } else { c })
            .take(250)
            .collect()
    }
}

/// Cache value serialization utilities
pub struct CacheValue;

impl CacheValue {
    /// Estimate the size of a cache value in bytes
    pub fn estimate_size<T: serde::Serialize>(value: &T) -> usize {
        // Simple estimation: serialize to JSON and get byte length
        // In production, you might want more sophisticated size estimation
        serde_json::to_string(value)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Check if a value exceeds the maximum cache entry size
    pub fn exceeds_max_size<T: serde::Serialize>(value: &T, max_size: usize) -> bool {
        Self::estimate_size(value) > max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_config() {
        let config = CacheEntryConfig::new()
            .with_ttl(Duration::from_secs(300))
            .with_namespace("test");

        assert_eq!(config.effective_ttl(), Duration::from_secs(300));
        assert_eq!(config.effective_namespace(), "test");
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 2.0 / 3.0);
    }

    #[test]
    fn test_cache_key_utilities() {
        let namespaced = CacheKey::namespaced("ns", "key");
        assert_eq!(namespaced, "ns:key");

        assert_eq!(CacheKey::extract_namespace("ns:key"), Some("ns"));
        assert_eq!(CacheKey::extract_key("ns:key"), "key");

        // Valid key
        assert!(CacheKey::validate_key("valid_key").is_ok());

        // Invalid keys
        assert!(CacheKey::validate_key("").is_err());
        assert!(CacheKey::validate_key(&"a".repeat(251)).is_err());
        assert!(CacheKey::validate_key("key\nwith\nlines").is_err());

        // Sanitization
        let sanitized = CacheKey::sanitize_key("key\nwith\nlines");
        assert_eq!(sanitized, "key_with_lines");
    }
}