//! Rate limiting for authentication endpoints
//!
//! Provides DDoS protection and brute-force attack prevention
//! using the governor crate for rate limiting.

use crate::infrastructure::constants::{
    RATE_LIMIT_AUTH_LOCKOUT_DURATION, RATE_LIMIT_AUTH_MAX_FAILED_ATTEMPTS,
    RATE_LIMIT_AUTH_MAX_REQUESTS, RATE_LIMIT_AUTH_WINDOW_SECONDS,
};
use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
    /// Lockout duration after exceeding limit
    pub lockout_duration: Duration,
    /// Maximum failed login attempts before lockout
    pub max_failed_attempts: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: RATE_LIMIT_AUTH_MAX_REQUESTS,
            window: Duration::from_secs(RATE_LIMIT_AUTH_WINDOW_SECONDS),
            lockout_duration: Duration::from_secs(RATE_LIMIT_AUTH_LOCKOUT_DURATION),
            max_failed_attempts: RATE_LIMIT_AUTH_MAX_FAILED_ATTEMPTS,
        }
    }
}

/// Rate limit entry for a single client
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Request count in current window
    request_count: u32,
    /// Failed login attempts
    failed_attempts: u32,
    /// Window start time
    window_start: Instant,
    /// Lockout end time (if locked out)
    lockout_until: Option<Instant>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            request_count: 0,
            failed_attempts: 0,
            window_start: Instant::now(),
            lockout_until: None,
        }
    }

    fn is_locked_out(&self) -> bool {
        self.lockout_until
            .is_some_and(|until| Instant::now() < until)
    }

    fn reset_window(&mut self) {
        self.request_count = 0;
        self.window_start = Instant::now();
    }
}

/// Authentication rate limiter
///
/// Tracks request rates per client IP and implements:
/// - Request rate limiting
/// - Failed login attempt tracking
/// - Automatic lockout after exceeded limits
pub struct AuthRateLimiter {
    config: RateLimitConfig,
    entries: RwLock<HashMap<String, RateLimitEntry>>,
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

impl AuthRateLimiter {
    /// Create a new rate limiter with custom configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a request should be allowed
    ///
    /// Returns Ok(()) if allowed, Err with retry-after duration if rate limited.
    /// Returns default duration on lock poisoning (defensive - allows request).
    pub fn check_request(&self, client_id: &str) -> Result<(), Duration> {
        let Ok(mut entries) = self.entries.write() else {
            // Lock poisoned - defensive: allow the request but log warning
            tracing::warn!("Rate limiter lock poisoned, allowing request");
            return Ok(());
        };
        let entry = entries
            .entry(client_id.to_string())
            .or_insert_with(RateLimitEntry::new);

        // Check lockout
        if entry.is_locked_out() {
            // Safe: is_locked_out() ensures lockout_until is Some
            if let Some(until) = entry.lockout_until {
                let remaining = until - Instant::now();
                return Err(remaining);
            }
        }

        // Check if window has expired
        if entry.window_start.elapsed() > self.config.window {
            entry.reset_window();
        }

        // Check rate limit
        if entry.request_count >= self.config.max_requests {
            let remaining = self.config.window - entry.window_start.elapsed();
            return Err(remaining);
        }

        entry.request_count += 1;
        Ok(())
    }

    /// Record a failed login attempt
    ///
    /// After max_failed_attempts, the client is locked out.
    pub fn record_failed_attempt(&self, client_id: &str) {
        let Ok(mut entries) = self.entries.write() else {
            tracing::warn!("Rate limiter lock poisoned, cannot record failed attempt");
            return;
        };
        let entry = entries
            .entry(client_id.to_string())
            .or_insert_with(RateLimitEntry::new);

        entry.failed_attempts += 1;

        if entry.failed_attempts >= self.config.max_failed_attempts {
            entry.lockout_until = Some(Instant::now() + self.config.lockout_duration);
            tracing::warn!(
                "Client {} locked out after {} failed attempts",
                client_id,
                entry.failed_attempts
            );
        }
    }

    /// Record a successful login (resets failed attempt counter)
    pub fn record_success(&self, client_id: &str) {
        let Ok(mut entries) = self.entries.write() else {
            tracing::warn!("Rate limiter lock poisoned, cannot record success");
            return;
        };
        if let Some(entry) = entries.get_mut(client_id) {
            entry.failed_attempts = 0;
            entry.lockout_until = None;
        }
    }

    /// Get current status for a client
    pub fn get_status(&self, client_id: &str) -> RateLimitStatus {
        let Ok(entries) = self.entries.read() else {
            // Lock poisoned - return default OK status
            return RateLimitStatus::Ok {
                remaining_requests: self.config.max_requests,
                window_remaining_secs: self.config.window.as_secs(),
                failed_attempts: 0,
            };
        };

        if let Some(entry) = entries.get(client_id) {
            if entry.is_locked_out() {
                // Safe: is_locked_out() ensures lockout_until is Some
                if let Some(until) = entry.lockout_until {
                    let remaining = until - Instant::now();
                    return RateLimitStatus::LockedOut {
                        remaining_secs: remaining.as_secs(),
                    };
                }
            }

            let remaining_requests = self.config.max_requests.saturating_sub(entry.request_count);
            let window_remaining = self
                .config
                .window
                .saturating_sub(entry.window_start.elapsed());

            RateLimitStatus::Ok {
                remaining_requests,
                window_remaining_secs: window_remaining.as_secs(),
                failed_attempts: entry.failed_attempts,
            }
        } else {
            RateLimitStatus::Ok {
                remaining_requests: self.config.max_requests,
                window_remaining_secs: self.config.window.as_secs(),
                failed_attempts: 0,
            }
        }
    }

    /// Clean up expired entries (call periodically)
    pub fn cleanup(&self) {
        let Ok(mut entries) = self.entries.write() else {
            tracing::warn!("Rate limiter lock poisoned, cannot cleanup");
            return;
        };
        let now = Instant::now();
        let window = self.config.window;

        entries.retain(|_, entry| {
            // Keep if within window or locked out
            entry.window_start.elapsed() < window * 2
                || entry.lockout_until.is_some_and(|until| until > now)
        });
    }
}

/// Rate limit status for a client
#[derive(Debug, Clone)]
pub enum RateLimitStatus {
    /// Client is allowed to make requests
    Ok {
        remaining_requests: u32,
        window_remaining_secs: u64,
        failed_attempts: u32,
    },
    /// Client is locked out
    LockedOut { remaining_secs: u64 },
}

/// Rate limit error response
pub struct RateLimitError {
    pub retry_after: Duration,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let headers = [("Retry-After", self.retry_after.as_secs().to_string())];

        (
            StatusCode::TOO_MANY_REQUESTS,
            headers,
            format!(
                "Rate limit exceeded. Retry after {} seconds.",
                self.retry_after.as_secs()
            ),
        )
            .into_response()
    }
}

/// Extract client identifier from request
///
/// Uses IP address from ConnectInfo, falling back to a default.
pub fn extract_client_id(addr: Option<&ConnectInfo<SocketAddr>>) -> String {
    addr.map(|a| a.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Rate limiting middleware state
pub type RateLimiterState = Arc<AuthRateLimiter>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_allows_requests() {
        let limiter = AuthRateLimiter::new(RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
            max_failed_attempts: 3,
        });

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_request("client1").is_ok());
        }

        // 6th request should be rate limited
        assert!(limiter.check_request("client1").is_err());
    }

    #[test]
    fn test_failed_attempts_lockout() {
        let limiter = AuthRateLimiter::new(RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
            max_failed_attempts: 3,
        });

        // Record failed attempts
        limiter.record_failed_attempt("client1");
        limiter.record_failed_attempt("client1");
        assert!(limiter.check_request("client1").is_ok());

        // Third failure triggers lockout
        limiter.record_failed_attempt("client1");

        // Should be locked out
        let result = limiter.check_request("client1");
        assert!(result.is_err());
    }

    #[test]
    fn test_success_resets_failed_attempts() {
        let limiter = AuthRateLimiter::new(RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
            max_failed_attempts: 3,
        });

        limiter.record_failed_attempt("client1");
        limiter.record_failed_attempt("client1");

        // Success should reset counter
        limiter.record_success("client1");

        // Should need 3 more failures to lock out
        limiter.record_failed_attempt("client1");
        limiter.record_failed_attempt("client1");
        assert!(limiter.check_request("client1").is_ok());
    }

    #[test]
    fn test_different_clients_independent() {
        let limiter = AuthRateLimiter::new(RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
            max_failed_attempts: 3,
        });

        // Exhaust client1's limit
        limiter.check_request("client1").unwrap();
        limiter.check_request("client1").unwrap();
        assert!(limiter.check_request("client1").is_err());

        // client2 should still be allowed
        assert!(limiter.check_request("client2").is_ok());
    }

    #[test]
    fn test_status_reporting() {
        let limiter = AuthRateLimiter::new(RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
            max_failed_attempts: 3,
        });

        limiter.check_request("client1").unwrap();
        limiter.check_request("client1").unwrap();

        match limiter.get_status("client1") {
            RateLimitStatus::Ok {
                remaining_requests,
                failed_attempts,
                ..
            } => {
                assert_eq!(remaining_requests, 3);
                assert_eq!(failed_attempts, 0);
            }
            _ => panic!("Expected Ok status"),
        }
    }
}
