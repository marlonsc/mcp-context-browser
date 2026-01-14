//! Circuit breaker implementations
//!
//! - `TowerCircuitBreaker`: tower-resilience based (single-node)
//! - Future: Redis-backed for cluster coordination

use super::traits::{CircuitBreakerBackend, CircuitBreakerState};
use crate::infrastructure::constants::{
    CIRCUIT_BREAKER_FAILURE_THRESHOLD, CIRCUIT_BREAKER_HALF_OPEN_MAX_REQUESTS,
    CIRCUIT_BREAKER_RECOVERY_TIMEOUT,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure rate threshold (0-100) to trip the circuit
    pub failure_threshold: u32,
    /// Window size for tracking calls
    pub window_size: usize,
    /// How long to wait before trying half-open
    pub recovery_timeout: Duration,
    /// Max requests to allow in half-open state
    pub half_open_max_requests: u32,
    /// Circuit breaker name/id
    pub name: String,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: CIRCUIT_BREAKER_FAILURE_THRESHOLD as u32,
            window_size: 10, // Default window size for tracking calls
            recovery_timeout: CIRCUIT_BREAKER_RECOVERY_TIMEOUT,
            half_open_max_requests: CIRCUIT_BREAKER_HALF_OPEN_MAX_REQUESTS,
            name: "default".to_string(),
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new configuration with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the recovery timeout
    pub fn with_recovery_timeout(mut self, timeout: Duration) -> Self {
        self.recovery_timeout = timeout;
        self
    }
}

/// In-memory circuit breaker implementation
///
/// Implements the circuit breaker pattern with sliding window failure tracking.
/// Suitable for single-node deployments.
pub struct TowerCircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current state
    state: RwLock<CircuitBreakerState>,
    /// Failure count in current window
    failure_count: AtomicU32,
    /// Success count in current window
    success_count: AtomicU32,
    /// Total calls in current window
    total_calls: AtomicU32,
    /// Time when circuit opened (for recovery timeout)
    opened_at: RwLock<Option<Instant>>,
    /// Calls allowed in half-open state
    half_open_calls: AtomicU32,
}

impl TowerCircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitBreakerState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            total_calls: AtomicU32::new(0),
            opened_at: RwLock::new(None),
            half_open_calls: AtomicU32::new(0),
        }
    }

    /// Calculate current failure rate
    fn failure_rate(&self) -> f64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failures = self.failure_count.load(Ordering::Relaxed);
        (failures as f64 / total as f64) * 100.0
    }

    /// Check if we should transition to half-open
    fn should_try_half_open(&self) -> bool {
        if let Ok(opened_at) = self.opened_at.read() {
            if let Some(opened) = *opened_at {
                return opened.elapsed() >= self.config.recovery_timeout;
            }
        }
        false
    }

    /// Transition to a new state
    fn transition_to(&self, new_state: CircuitBreakerState) {
        if let Ok(mut state) = self.state.write() {
            let old_state = *state;
            if old_state != new_state {
                tracing::info!(
                    circuit_breaker = %self.config.name,
                    from = %old_state,
                    to = %new_state,
                    "Circuit breaker state transition"
                );

                *state = new_state;

                match new_state {
                    CircuitBreakerState::Open => {
                        if let Ok(mut opened_at) = self.opened_at.write() {
                            *opened_at = Some(Instant::now());
                        }
                    }
                    CircuitBreakerState::Closed => {
                        // Reset counters on close
                        self.failure_count.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                        self.total_calls.store(0, Ordering::Relaxed);
                        if let Ok(mut opened_at) = self.opened_at.write() {
                            *opened_at = None;
                        }
                    }
                    CircuitBreakerState::HalfOpen => {
                        self.half_open_calls.store(0, Ordering::Relaxed);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl CircuitBreakerBackend for TowerCircuitBreaker {
    fn is_call_permitted(&self) -> bool {
        let state = self
            .state
            .read()
            .map(|s| *s)
            .unwrap_or(CircuitBreakerState::Closed);

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if self.should_try_half_open() {
                    self.transition_to(CircuitBreakerState::HalfOpen);
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                let current = self.half_open_calls.fetch_add(1, Ordering::Relaxed);
                current < self.config.half_open_max_requests
            }
        }
    }

    async fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        let state = self
            .state
            .read()
            .map(|s| *s)
            .unwrap_or(CircuitBreakerState::Closed);

        if state == CircuitBreakerState::HalfOpen {
            // Success in half-open transitions to closed
            let successes = self.success_count.load(Ordering::Relaxed);
            if successes >= self.config.half_open_max_requests {
                self.transition_to(CircuitBreakerState::Closed);
            }
        }
    }

    async fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        let state = self
            .state
            .read()
            .map(|s| *s)
            .unwrap_or(CircuitBreakerState::Closed);

        match state {
            CircuitBreakerState::Closed => {
                let total = self.total_calls.load(Ordering::Relaxed);
                if total >= self.config.window_size as u32 {
                    let rate = self.failure_rate();
                    if rate >= self.config.failure_threshold as f64 {
                        self.transition_to(CircuitBreakerState::Open);
                    }
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open trips back to open
                self.transition_to(CircuitBreakerState::Open);
            }
            CircuitBreakerState::Open => {
                // Already open, nothing to do
            }
        }
    }

    fn state(&self) -> CircuitBreakerState {
        self.state
            .read()
            .map(|s| *s)
            .unwrap_or(CircuitBreakerState::Closed)
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn backend_type(&self) -> &'static str {
        "tower"
    }
}

/// Null circuit breaker - always allows (for testing)
/// Null circuit breaker that always allows calls (no-op implementation)
pub struct NullCircuitBreaker {
    /// Name identifier for this circuit breaker instance
    name: String,
}

impl NullCircuitBreaker {
    /// Create a new null circuit breaker (always allows calls)
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl CircuitBreakerBackend for NullCircuitBreaker {
    /// Check if a call is permitted (always returns true for null implementation)
    fn is_call_permitted(&self) -> bool {
        true
    }

    /// Record a successful call (no-op for null implementation)
    async fn record_success(&self) {}

    /// Record a failed call (no-op for null implementation)
    async fn record_failure(&self) {}

    /// Get the current circuit breaker state (always Closed for null implementation)
    fn state(&self) -> CircuitBreakerState {
        CircuitBreakerState::Closed
    }

    /// Get the circuit breaker name
    fn name(&self) -> &str {
        &self.name
    }

    fn backend_type(&self) -> &'static str {
        "null"
    }
}
