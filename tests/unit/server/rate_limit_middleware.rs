//! Tests for Rate Limit Middleware
//!
//! Tests migrated from src/server/rate_limit_middleware.rs

use axum::extract::ConnectInfo;
use mcp_context_browser::infrastructure::rate_limit::RateLimiter;
use mcp_context_browser::server::rate_limit_middleware::check_rate_limit_for_ip;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::test]
async fn test_rate_limit_middleware_no_limiter() {
    // Test that functions exist and can be called
    let rate_limiter: Option<Arc<RateLimiter>> = None;
    let addr = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080)));

    // Should succeed with no rate limiter
    let result = check_rate_limit_for_ip(&rate_limiter, &addr).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_middleware_with_disabled_limiter() {
    // Test with disabled rate limiter
    let config = mcp_context_browser::infrastructure::rate_limit::RateLimitConfig {
        enabled: false,
        ..Default::default()
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let rate_limiter = Some(limiter);
    let addr = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080)));

    // Should succeed with disabled rate limiter
    let result = check_rate_limit_for_ip(&rate_limiter, &addr).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_middleware_with_enabled_limiter() {
    // Test with enabled rate limiter
    let config = mcp_context_browser::infrastructure::rate_limit::RateLimitConfig {
        enabled: true,
        max_requests_per_window: 10,
        burst_allowance: 2,
        ..Default::default()
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let rate_limiter = Some(limiter);
    let addr = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080)));

    // First request should succeed
    let result = check_rate_limit_for_ip(&rate_limiter, &addr).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_different_ips() {
    // Test that different IPs are tracked separately
    let config = mcp_context_browser::infrastructure::rate_limit::RateLimitConfig {
        enabled: true,
        max_requests_per_window: 5,
        burst_allowance: 0,
        ..Default::default()
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let rate_limiter = Some(limiter);

    let addr1 = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080)));
    let addr2 = ConnectInfo(SocketAddr::from(([192, 168, 1, 1], 8080)));

    // Both IPs should be able to make requests
    let result1 = check_rate_limit_for_ip(&rate_limiter, &addr1).await;
    let result2 = check_rate_limit_for_ip(&rate_limiter, &addr2).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_socket_addr_ip_extraction() {
    // Test that IP can be extracted from SocketAddr
    let addr = SocketAddr::from(([192, 168, 1, 100], 8080));
    let ip_string = addr.ip().to_string();
    assert_eq!(ip_string, "192.168.1.100");
}

#[test]
fn test_socket_addr_ipv6() {
    // Test IPv6 address handling
    let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 8080));
    let ip_string = addr.ip().to_string();
    assert_eq!(ip_string, "::1");
}
