//! Rate limiting tests
//!
//! Tests migrated from src/infrastructure/rate_limit.rs

use mcp_context_browser::infrastructure::rate_limit::{
    RateLimitBackend, RateLimitConfig, RateLimitKey, RateLimiter,
};

fn default_redis_timeout() -> u64 {
    5
}

#[tokio::test]
async fn test_rate_limiter_disabled() {
    let config = RateLimitConfig {
        enabled: false,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    let key = RateLimitKey::Ip("127.0.0.1".to_string());
    let result = limiter.check_rate_limit(&key).await.unwrap();

    assert!(result.allowed);
    assert_eq!(result.remaining, u32::MAX);
    assert_eq!(result.reset_in_seconds, 0);
}

#[tokio::test]
async fn test_memory_backend_basic() {
    let config = RateLimitConfig {
        backend: RateLimitBackend::Memory { max_entries: 1000 },
        window_seconds: 60,
        max_requests_per_window: 10,
        burst_allowance: 2,
        enabled: true,
        redis_timeout_seconds: default_redis_timeout(),
    };
    let limiter = RateLimiter::new(config);
    limiter.init().await.unwrap();

    // Use a unique key for this test to avoid cache interference
    let key = RateLimitKey::Ip(format!("test-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()));

    // First request should be allowed
    let result = limiter.check_rate_limit(&key).await.unwrap();
    assert!(result.allowed, "First request should be allowed");

    // Verify rate limiter reports correct limits
    assert_eq!(result.limit, 12); // 10 + 2 burst
}

#[tokio::test]
async fn test_backend_types() {
    let memory_config = RateLimitConfig {
        backend: RateLimitBackend::Memory { max_entries: 1000 },
        ..Default::default()
    };
    let memory_limiter = RateLimiter::new(memory_config);
    assert_eq!(memory_limiter.backend_type(), "memory");

    let redis_config = RateLimitConfig {
        backend: RateLimitBackend::Redis {
            url: "redis://localhost:6379".to_string(),
        },
        redis_timeout_seconds: 5,
        ..Default::default()
    };
    let redis_limiter = RateLimiter::new(redis_config);
    assert_eq!(redis_limiter.backend_type(), "redis");
}

#[tokio::test]
async fn test_rate_limit_keys() {
    let ip_key = RateLimitKey::Ip("192.168.1.1".to_string());
    let user_key = RateLimitKey::User("user123".to_string());
    let api_key = RateLimitKey::ApiKey("key456".to_string());
    let endpoint_key = RateLimitKey::Endpoint("/api/search".to_string());

    assert_eq!(ip_key.to_string(), "ip:192.168.1.1");
    assert_eq!(user_key.to_string(), "user:user123");
    assert_eq!(api_key.to_string(), "apikey:key456");
    assert_eq!(endpoint_key.to_string(), "endpoint:/api/search");
}

#[test]
fn test_rate_limit_config_default() {
    let config = RateLimitConfig::default();
    match config.backend {
        RateLimitBackend::Memory { .. } => {} // Default is memory
        _ => panic!("Expected memory backend"),
    }
    assert_eq!(config.window_seconds, 60);
    assert_eq!(config.max_requests_per_window, 100);
    assert_eq!(config.burst_allowance, 20);
    assert!(config.enabled);
    assert_eq!(config.redis_timeout_seconds, 5);
}
