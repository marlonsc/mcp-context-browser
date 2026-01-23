//! Resilience configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Resilience configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,

    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_secs: u64,

    /// Circuit breaker success threshold
    pub circuit_breaker_success_threshold: u32,

    /// Rate limiter requests per second
    pub rate_limiter_rps: u32,

    /// Rate limiter burst size
    pub rate_limiter_burst: u32,

    /// Retry attempts
    pub retry_attempts: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

/// Returns default resilience configuration with:
/// - Circuit breaker thresholds from infrastructure constants
/// - Rate limiter settings from infrastructure constants
/// - 3 retry attempts with 1 second delay
impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_failure_threshold: CIRCUIT_BREAKER_FAILURE_THRESHOLD,
            circuit_breaker_timeout_secs: CIRCUIT_BREAKER_TIMEOUT_SECS,
            circuit_breaker_success_threshold: CIRCUIT_BREAKER_SUCCESS_THRESHOLD,
            rate_limiter_rps: RATE_LIMITER_DEFAULT_RPS,
            rate_limiter_burst: RATE_LIMITER_DEFAULT_BURST,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}
