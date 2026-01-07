//! Circuit Breaker Module
//!
//! This module provides circuit breaker functionality using established patterns
//! and libraries, following SOLID principles with proper separation of concerns.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open { opened_at: Instant },
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerState::Closed => write!(f, "closed"),
            CircuitBreakerState::Open { .. } => write!(f, "open"),
            CircuitBreakerState::HalfOpen => write!(f, "half-open"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting recovery
    pub recovery_timeout: Duration,
    /// Number of successes needed to close circuit when half-open
    pub success_threshold: u32,
    /// Maximum requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 10,
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub consecutive_failures: u32,
    pub circuit_opened_count: u32,
    pub circuit_closed_count: u32,
    pub last_failure: Option<Instant>,
    pub last_success: Option<Instant>,
}

/// Persisted circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerSnapshot {
    /// Current state
    pub state: String, // "closed", "open", "half-open"
    /// When the circuit was opened (seconds since saved_at)
    pub opened_at_offset: Option<u64>,
    /// Metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub consecutive_failures: u32,
    pub circuit_opened_count: u32,
    pub circuit_closed_count: u32,
    /// Last saved timestamp
    pub saved_at: u64,
}

/// Circuit breaker implementation using established patterns
#[derive(Clone)]
pub struct CircuitBreaker {
    /// Circuit breaker identifier
    id: String,
    /// Persistence directory
    persistence_dir: PathBuf,
    /// Channel sender for background persistence
    persistence_sender: mpsc::UnboundedSender<()>,
    /// Current state
    state: Arc<RwLock<CircuitBreakerState>>,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Metrics
    metrics: Arc<RwLock<CircuitBreakerMetrics>>,
    /// Request timestamps for rolling window (simplified)
    failure_timestamps: Arc<RwLock<Vec<Instant>>>,
    /// Success count in half-open state
    half_open_success_count: Arc<RwLock<u32>>,
    /// Request count in half-open state
    half_open_request_count: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self::with_id_and_config("default".to_string(), config)
    }

    /// Create a new circuit breaker with ID and configuration
    pub fn with_id_and_config(id: String, config: CircuitBreakerConfig) -> Self {
        let persistence_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".context")
            .join("circuit_breakers");

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&persistence_dir) {
            warn!(
                "Failed to create circuit breaker persistence directory: {}",
                e
            );
        }

        // Create channel for background persistence
        let (persistence_sender, mut persistence_receiver) = mpsc::unbounded_channel::<()>();

        let mut breaker = Self {
            id: id.clone(),
            persistence_dir: persistence_dir.clone(),
            persistence_sender,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            config,
            metrics: Arc::new(RwLock::new(CircuitBreakerMetrics::default())),
            failure_timestamps: Arc::new(RwLock::new(Vec::new())),
            half_open_success_count: Arc::new(RwLock::new(0)),
            half_open_request_count: Arc::new(RwLock::new(0)),
        };

        // Load persisted state if available
        if let Err(e) = breaker.load_state() {
            debug!("Failed to load persisted circuit breaker state: {}", e);
        }

        // Spawn background persistence task
        let breaker_clone = Arc::new(breaker.clone());
        tokio::spawn(async move {
            while persistence_receiver.recv().await.is_some() {
                if let Err(e) = breaker_clone.save_state_async().await {
                    warn!("Failed to persist circuit breaker state for {}: {}", id, e);
                }
            }
        });

        breaker
    }

    /// Get the circuit breaker ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Save current state to disk (async version to avoid deadlocks)
    async fn save_state_async(&self) -> Result<()> {
        let snapshot = self.create_snapshot();
        let file_path = self.persistence_dir.join(format!("{}.json", self.id));

        let content = serde_json::to_string_pretty(&snapshot).map_err(|e| {
            Error::internal(format!("Failed to serialize circuit breaker state: {}", e))
        })?;

        // Use tokio::fs for async file operations
        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| Error::internal(format!("Failed to save circuit breaker state: {}", e)))?;

        debug!("Saved circuit breaker state for {}", self.id);
        Ok(())
    }

    /// Load state from disk
    fn load_state(&mut self) -> Result<()> {
        let file_path = self.persistence_dir.join(format!("{}.json", self.id));

        if !file_path.exists() {
            debug!("No persisted state found for circuit breaker {}", self.id);
            return Ok(());
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| Error::internal(format!("Failed to read circuit breaker state: {}", e)))?;

        let snapshot: CircuitBreakerSnapshot = serde_json::from_str(&content).map_err(|e| {
            Error::internal(format!(
                "Failed to deserialize circuit breaker state: {}",
                e
            ))
        })?;

        self.restore_from_snapshot(snapshot);
        debug!("Loaded circuit breaker state for {}", self.id);
        Ok(())
    }

    /// Create a snapshot of current state
    fn create_snapshot(&self) -> CircuitBreakerSnapshot {
        futures::executor::block_on(async {
            let state = *self.state.read().await;
            let metrics = self.metrics.read().await;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let state_str = match state {
                CircuitBreakerState::Closed => "closed".to_string(),
                CircuitBreakerState::Open { .. } => "open".to_string(),
                CircuitBreakerState::HalfOpen => "half-open".to_string(),
            };

            CircuitBreakerSnapshot {
                state: state_str,
                opened_at_offset: None, // Simplified: will restore to closed state
                total_requests: metrics.total_requests,
                successful_requests: metrics.successful_requests,
                failed_requests: metrics.failed_requests,
                rejected_requests: metrics.rejected_requests,
                consecutive_failures: metrics.consecutive_failures,
                circuit_opened_count: metrics.circuit_opened_count,
                circuit_closed_count: metrics.circuit_closed_count,
                saved_at: now,
            }
        })
    }

    /// Restore state from snapshot
    fn restore_from_snapshot(&mut self, snapshot: CircuitBreakerSnapshot) {
        futures::executor::block_on(async {
            // Always restore to closed state for simplicity
            // The important part is preserving the metrics
            *self.state.write().await = CircuitBreakerState::Closed;

            // Restore metrics
            let mut metrics = self.metrics.write().await;
            metrics.total_requests = snapshot.total_requests;
            metrics.successful_requests = snapshot.successful_requests;
            metrics.failed_requests = snapshot.failed_requests;
            metrics.rejected_requests = snapshot.rejected_requests;
            metrics.consecutive_failures = snapshot.consecutive_failures;
            metrics.circuit_opened_count = snapshot.circuit_opened_count;
            metrics.circuit_closed_count = snapshot.circuit_closed_count;
        });
    }

    /// Execute an operation with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if we should allow the request
        if !self.should_allow_request().await {
            let mut metrics = self.metrics.write().await;
            metrics.rejected_requests += 1;
            return Err(Error::generic("Circuit breaker is open"));
        }

        // Execute the operation
        let start_time = Instant::now();
        let result = operation().await;
        let response_time = start_time.elapsed();

        // Update metrics and state based on result
        match result {
            Ok(value) => {
                self.handle_success(response_time).await;
                Ok(value)
            }
            Err(e) => {
                self.handle_failure(response_time).await;
                Err(e)
            }
        }
    }

    /// Check if a request should be allowed
    async fn should_allow_request(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                // Check if recovery timeout has passed
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    // Transition to half-open
                    *self.state.write().await = CircuitBreakerState::HalfOpen;
                    *self.half_open_success_count.write().await = 0;
                    *self.half_open_request_count.write().await = 0;
                    debug!("Circuit breaker transitioning to half-open state");
                    // Trigger background persistence
                    let _ = self.persistence_sender.send(());
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                let request_count = *self.half_open_request_count.read().await;
                request_count < self.config.half_open_max_requests
            }
        }
    }

    /// Handle successful operation
    async fn handle_success(&self, _response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        metrics.consecutive_failures = 0;
        metrics.last_success = Some(Instant::now());

        let state = *self.state.read().await;
        match state {
            CircuitBreakerState::HalfOpen => {
                let mut success_count = self.half_open_success_count.write().await;
                let mut request_count = self.half_open_request_count.write().await;

                *success_count += 1;
                *request_count += 1;

                // Check if we've reached success threshold
                if *success_count >= self.config.success_threshold {
                    // Close the circuit
                    *self.state.write().await = CircuitBreakerState::Closed;
                    *success_count = 0;
                    *request_count = 0;

                    metrics.circuit_closed_count += 1;
                    info!("Circuit breaker closed after successful recovery");

                    // Trigger background persistence
                    let _ = self.persistence_sender.send(());
                }
            }
            CircuitBreakerState::Closed => {
                // Clean old failure timestamps (rolling window of 5 minutes)
                let mut failure_timestamps = self.failure_timestamps.write().await;
                let cutoff = Instant::now() - Duration::from_secs(300);
                failure_timestamps.retain(|&timestamp| timestamp > cutoff);
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, just log success
            }
        }
    }

    /// Handle failed operation
    async fn handle_failure(&self, _response_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        metrics.consecutive_failures += 1;
        metrics.last_failure = Some(Instant::now());

        // Add failure timestamp for rolling window
        {
            let mut failure_timestamps = self.failure_timestamps.write().await;
            failure_timestamps.push(Instant::now());

            // Clean old timestamps (rolling window of 5 minutes)
            let cutoff = Instant::now() - Duration::from_secs(300);
            failure_timestamps.retain(|&timestamp| timestamp > cutoff);
        }

        let state = *self.state.read().await;
        match state {
            CircuitBreakerState::HalfOpen => {
                *self.half_open_request_count.write().await += 1;

                // Failure in half-open state - go back to open
                *self.state.write().await = CircuitBreakerState::Open {
                    opened_at: Instant::now(),
                };

                metrics.circuit_opened_count += 1;
                warn!("Circuit breaker reopened due to failure in half-open state");

                drop(metrics);
                if let Err(e) = self.save_state_async().await {
                    warn!("Failed to persist circuit breaker state change: {}", e);
                }
            }
            CircuitBreakerState::Closed => {
                // Check if we've exceeded failure threshold in rolling window
                let failure_timestamps = self.failure_timestamps.read().await;
                if failure_timestamps.len() >= self.config.failure_threshold as usize {
                    *self.state.write().await = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                    };

                    metrics.circuit_opened_count += 1;
                    warn!("Circuit breaker opened due to failure threshold exceeded");

                    // Trigger background persistence
                    let _ = self.persistence_sender.send(());
                }
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, just log
            }
        }
    }

    /// Get current circuit breaker state
    pub async fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    /// Get circuit breaker metrics
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset circuit breaker to closed state
    pub async fn reset(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        *self.metrics.write().await = CircuitBreakerMetrics::default();
        *self.failure_timestamps.write().await = Vec::new();
        *self.half_open_success_count.write().await = 0;
        *self.half_open_request_count.write().await = 0;
    }

    /// Check if circuit breaker allows requests
    pub async fn allows_requests(&self) -> bool {
        let state = *self.state.read().await;
        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                opened_at.elapsed() >= self.config.recovery_timeout
            }
            CircuitBreakerState::HalfOpen => {
                *self.half_open_request_count.read().await < self.config.half_open_max_requests
            }
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
        assert!(cb.allows_requests().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_successful_operations() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::with_id_and_config("test_successful".to_string(), config);

        // Multiple successful operations should keep circuit closed
        for _ in 0..10 {
            let result = cb.call(|| async { Ok::<u32, Error>(42) }).await;
            assert!(result.is_ok());
            assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
        }

        let metrics = cb.get_metrics().await;
        assert_eq!(metrics.successful_requests, 10);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Fail multiple times to open circuit
        for i in 0..3 {
            let result = cb
                .call(|| async { Err::<u32, Error>(Error::generic("test error")) })
                .await;
            assert!(result.is_err());

            if i < 2 {
                assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
            }
        }

        // Circuit should be open
        assert!(matches!(
            cb.get_state().await,
            CircuitBreakerState::Open { .. }
        ));
        assert!(!cb.allows_requests().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Fail to open circuit
        for _ in 0..2 {
            let _ = cb
                .call(|| async { Err::<u32, Error>(Error::generic("test error")) })
                .await;
        }

        assert!(matches!(
            cb.get_state().await,
            CircuitBreakerState::Open { .. }
        ));

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next call should go to half-open and succeed
        let result = cb.call(|| async { Ok::<u32, Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state().await, CircuitBreakerState::HalfOpen);

        // Another success should close the circuit
        let result = cb.call(|| async { Ok::<u32, Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new();

        // Open circuit with failures
        for _ in 0..5 {
            let _ = cb
                .call(|| async { Err::<u32, Error>(Error::generic("test error")) })
                .await;
        }

        assert!(matches!(
            cb.get_state().await,
            CircuitBreakerState::Open { .. }
        ));

        // Reset should close circuit
        cb.reset().await;
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);

        let metrics = cb.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
    }
}
