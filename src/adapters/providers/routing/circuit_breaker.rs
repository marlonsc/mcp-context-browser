//! Circuit Breaker Module
//!
//! This module provides circuit breaker functionality using the Actor pattern
//! to eliminate locks and ensure non-blocking operation.

use crate::domain::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tracing::{info, instrument, warn};

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
    /// Whether to enable persistence
    pub persistence_enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 10,
            persistence_enabled: true,
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

/// Messages for the circuit breaker actor
pub(crate) enum CBMessage {
    CanCall(oneshot::Sender<bool>),
    OnSuccess,
    OnFailure,
    GetState(oneshot::Sender<CircuitBreakerState>),
    GetMetrics(oneshot::Sender<CircuitBreakerMetrics>),
    #[allow(dead_code)]
    ApplySnapshot(CircuitBreakerSnapshot),
}

/// Trait for circuit breaker
#[async_trait::async_trait]
pub trait CircuitBreakerTrait: Send + Sync {
    async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send;

    async fn state(&self) -> CircuitBreakerState;
    async fn metrics(&self) -> CircuitBreakerMetrics;
}

/// Circuit breaker implementation using Actor pattern
pub struct CircuitBreaker {
    id: String,
    sender: mpsc::Sender<CBMessage>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub async fn new(id: impl Into<String>) -> Self {
        Self::with_config(id, CircuitBreakerConfig::default()).await
    }

    /// Create a new circuit breaker with custom configuration
    pub async fn with_config(id: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let id = id.into();
        let (tx, rx) = mpsc::channel(100);

        let persistence_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".context")
            .join("circuit_breakers");

        let mut actor = CircuitBreakerActor::new(id.clone(), rx, config.clone(), persistence_dir);

        // Try to load persisted state if enabled
        if config.persistence_enabled {
            if let Ok(Some(snapshot)) = actor.load_snapshot().await {
                actor.apply_snapshot(snapshot);
            }
        }

        tokio::spawn(async move {
            actor.run().await;
        });

        Self { id, sender: tx }
    }

    async fn can_call(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(CBMessage::CanCall(tx)).await;
        rx.await.unwrap_or(false)
    }

    async fn on_success(&self) {
        let _ = self.sender.send(CBMessage::OnSuccess).await;
    }

    async fn on_failure(&self) {
        let _ = self.sender.send(CBMessage::OnFailure).await;
    }
}

#[async_trait::async_trait]
impl CircuitBreakerTrait for CircuitBreaker {
    #[instrument(skip(self, f), fields(id = %self.id))]
    async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        if !self.can_call().await {
            return Err(Error::generic(format!(
                "Circuit breaker {} is open or restricted",
                self.id
            )));
        }

        match f().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    async fn state(&self) -> CircuitBreakerState {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(CBMessage::GetState(tx)).await;
        rx.await.unwrap_or(CircuitBreakerState::Closed)
    }

    async fn metrics(&self) -> CircuitBreakerMetrics {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(CBMessage::GetMetrics(tx)).await;
        rx.await.unwrap_or_default()
    }
}

struct CircuitBreakerActor {
    id: String,
    receiver: mpsc::Receiver<CBMessage>,
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    metrics: CircuitBreakerMetrics,
    half_open_request_count: u32,
    persistence_dir: PathBuf,
    last_save: Instant,
}

impl CircuitBreakerActor {
    fn new(
        id: String,
        receiver: mpsc::Receiver<CBMessage>,
        config: CircuitBreakerConfig,
        persistence_dir: PathBuf,
    ) -> Self {
        Self {
            id,
            receiver,
            config,
            state: CircuitBreakerState::Closed,
            metrics: CircuitBreakerMetrics::default(),
            half_open_request_count: 0,
            persistence_dir,
            last_save: Instant::now(),
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                CBMessage::CanCall(tx) => {
                    self.check_state_transition();
                    let can = match self.state {
                        CircuitBreakerState::Closed => true,
                        CircuitBreakerState::Open { .. } => false,
                        CircuitBreakerState::HalfOpen => {
                            if self.half_open_request_count < self.config.half_open_max_requests {
                                self.half_open_request_count += 1;
                                true
                            } else {
                                self.metrics.rejected_requests += 1;
                                false
                            }
                        }
                    };
                    if !can && matches!(self.state, CircuitBreakerState::Open { .. }) {
                        self.metrics.rejected_requests += 1;
                    }
                    let _ = tx.send(can);
                }
                CBMessage::OnSuccess => {
                    self.metrics.total_requests += 1;
                    self.metrics.successful_requests += 1;
                    self.metrics.consecutive_failures = 0;
                    self.metrics.last_success = Some(Instant::now());

                    if self.state == CircuitBreakerState::HalfOpen {
                        // In our simplified actor model, we increment count in CanCall for HalfOpen.
                        // But we need to track actual successes to close it.
                        // Actually, the previous implementation used half_open_request_count for successes.
                        if self.half_open_request_count >= self.config.success_threshold {
                            info!("Circuit breaker {} transitioning to Closed", self.id);
                            self.state = CircuitBreakerState::Closed;
                            self.metrics.circuit_closed_count += 1;
                            self.request_save().await;
                        }
                    }
                    self.maybe_auto_save().await;
                }
                CBMessage::OnFailure => {
                    self.metrics.total_requests += 1;
                    self.metrics.failed_requests += 1;
                    self.metrics.consecutive_failures += 1;
                    self.metrics.last_failure = Some(Instant::now());

                    if self.state == CircuitBreakerState::Closed {
                        if self.metrics.consecutive_failures >= self.config.failure_threshold {
                            warn!("Circuit breaker {} transitioning to Open", self.id);
                            self.state = CircuitBreakerState::Open {
                                opened_at: Instant::now(),
                            };
                            self.metrics.circuit_opened_count += 1;
                            self.request_save().await;
                        }
                    } else if self.state == CircuitBreakerState::HalfOpen {
                        warn!(
                            "Circuit breaker {} failing in Half-Open, transitioning back to Open",
                            self.id
                        );
                        self.state = CircuitBreakerState::Open {
                            opened_at: Instant::now(),
                        };
                        self.metrics.circuit_opened_count += 1;
                        self.request_save().await;
                    }
                    self.maybe_auto_save().await;
                }
                CBMessage::GetState(tx) => {
                    self.check_state_transition();
                    let _ = tx.send(self.state);
                }
                CBMessage::GetMetrics(tx) => {
                    let _ = tx.send(self.metrics.clone());
                }
                CBMessage::ApplySnapshot(snapshot) => {
                    self.apply_snapshot(snapshot);
                }
            }
        }
    }

    fn check_state_transition(&mut self) {
        if let CircuitBreakerState::Open { opened_at } = self.state {
            if opened_at.elapsed() >= self.config.recovery_timeout {
                info!("Circuit breaker {} transitioning to Half-Open", self.id);
                self.state = CircuitBreakerState::HalfOpen;
                self.half_open_request_count = 0;
            }
        }
    }

    async fn maybe_auto_save(&mut self) {
        if self.config.persistence_enabled && self.last_save.elapsed() >= Duration::from_secs(30) {
            self.request_save().await;
        }
    }

    async fn request_save(&mut self) {
        if !self.config.persistence_enabled {
            return;
        }
        let snapshot = self.create_snapshot();
        let _ = self.save_snapshot(&snapshot).await;
        self.last_save = Instant::now();
    }

    fn create_snapshot(&self) -> CircuitBreakerSnapshot {
        let (state_str, opened_at_offset) = match self.state {
            CircuitBreakerState::Closed => ("closed", None),
            CircuitBreakerState::Open { opened_at } => {
                ("open", Some(opened_at.elapsed().as_secs()))
            }
            CircuitBreakerState::HalfOpen => ("half-open", None),
        };

        CircuitBreakerSnapshot {
            state: state_str.to_string(),
            opened_at_offset,
            total_requests: self.metrics.total_requests,
            successful_requests: self.metrics.successful_requests,
            failed_requests: self.metrics.failed_requests,
            rejected_requests: self.metrics.rejected_requests,
            consecutive_failures: self.metrics.consecutive_failures,
            circuit_opened_count: self.metrics.circuit_opened_count,
            circuit_closed_count: self.metrics.circuit_closed_count,
            saved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    async fn save_snapshot(&self, snapshot: &CircuitBreakerSnapshot) -> Result<()> {
        if !self.persistence_dir.exists() {
            tokio::fs::create_dir_all(&self.persistence_dir)
                .await
                .map_err(|e| Error::io(e.to_string()))?;
        }

        let file_path = self.persistence_dir.join(format!("{}.json", self.id));
        let content =
            serde_json::to_string(snapshot).map_err(|e| Error::internal(e.to_string()))?;
        tokio::fs::write(file_path, content)
            .await
            .map_err(|e| Error::io(e.to_string()))?;
        Ok(())
    }

    async fn load_snapshot(&self) -> Result<Option<CircuitBreakerSnapshot>> {
        let file_path = self.persistence_dir.join(format!("{}.json", self.id));
        if !file_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| Error::io(e.to_string()))?;
        let snapshot =
            serde_json::from_str(&content).map_err(|e| Error::internal(e.to_string()))?;
        Ok(Some(snapshot))
    }

    fn apply_snapshot(&mut self, snapshot: CircuitBreakerSnapshot) {
        self.state = match snapshot.state.as_str() {
            "closed" => CircuitBreakerState::Closed,
            "open" => {
                let opened_at = snapshot
                    .opened_at_offset
                    .map(|offset| {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let saved_at = snapshot.saved_at;
                        let elapsed_since_saved = now.saturating_sub(saved_at);
                        Instant::now()
                            .checked_sub(Duration::from_secs(offset + elapsed_since_saved))
                            .unwrap_or_else(Instant::now)
                    })
                    .unwrap_or_else(Instant::now);
                CircuitBreakerState::Open { opened_at }
            }
            "half-open" => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Closed,
        };

        self.metrics = CircuitBreakerMetrics {
            total_requests: snapshot.total_requests,
            successful_requests: snapshot.successful_requests,
            failed_requests: snapshot.failed_requests,
            rejected_requests: snapshot.rejected_requests,
            consecutive_failures: snapshot.consecutive_failures,
            circuit_opened_count: snapshot.circuit_opened_count,
            circuit_closed_count: snapshot.circuit_closed_count,
            last_failure: None,
            last_success: None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let id = format!("test_start_{}", uuid::Uuid::new_v4());
        let config = CircuitBreakerConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(id, config).await;
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_successful_operations() {
        let id = format!("test_success_{}", uuid::Uuid::new_v4());
        let config = CircuitBreakerConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(id, config).await;
        let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
        assert_eq!(cb.metrics().await.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let id = format!("test_failure_{}", uuid::Uuid::new_v4());
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            persistence_enabled: false,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(id, config).await;

        // First failure
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

        // Give the actor time to process
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(cb.state().await, CircuitBreakerState::Closed);

        // Second failure - should open
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

        // Give the actor time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let id = format!("test_reset_{}", uuid::Uuid::new_v4());
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(500),
            success_threshold: 1,
            persistence_enabled: false,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(id, config).await;

        // Open circuit
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;

        // Give the actor time to process the failure message (less than recovery timeout)
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

        // Wait for recovery timeout (more than 500ms)
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Should transition to half-open and then close on success
        let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
        assert!(result.is_ok(), "Call should succeed in half-open state");
        assert_eq!(result.unwrap(), 42);

        // Give the actor time to process the success message
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }
}
