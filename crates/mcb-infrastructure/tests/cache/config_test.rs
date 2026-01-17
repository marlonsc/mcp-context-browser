//! Cache Configuration Tests

use mcb_application::ports::providers::cache::{CacheEntryConfig, CacheStats};
use mcb_infrastructure::cache::config::CacheKey;
use std::time::Duration;

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
    // CacheStats is a domain type - just verify construction and calculate_hit_rate
    let stats = CacheStats {
        hits: 2,
        misses: 1,
        entries: 0,
        hit_rate: 0.0,
        bytes_used: 0,
    };

    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert!((stats.calculate_hit_rate() - 2.0 / 3.0).abs() < 0.001);
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
