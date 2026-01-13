//! Recovery Manager for Automatic Component Recovery
//!
//! This module provides a centralized recovery system that monitors component health
//! and automatically attempts to restart failed components with exponential backoff.
//!
//! # Features
//!
//! - Periodic health checks for all subsystems
//! - Automatic restart with configurable exponential backoff
//! - Per-subsystem recovery policies
//! - Integration with event bus for coordination
//! - Manual recovery trigger support
//!
//! # Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use mcp_context_browser::infrastructure::recovery::{RecoveryManager, RecoveryManagerInterface};
//! use mcp_context_browser::infrastructure::events::EventBus;
//! use mcp_context_browser::daemon::types::RecoveryConfig;
//!
//! async fn example() {
//!     let config = RecoveryConfig::default();
//!     let event_bus = Arc::new(EventBus::default());
//!     let manager = RecoveryManager::new(config, event_bus);
//!     manager.start().await;
//! }
//! ```

use crate::daemon::types::{RecoveryConfig, RecoveryState, RecoveryStatus, RecoveryStrategy};
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// Recovery manager interface for dependency injection
#[async_trait::async_trait]
pub trait RecoveryManagerInterface: Send + Sync {
    /// Start the recovery manager background task
    async fn start(&self) -> crate::domain::error::Result<()>;

    /// Stop the recovery manager
    async fn stop(&self) -> crate::domain::error::Result<()>;

    /// Check if the recovery manager is running
    fn is_running(&self) -> bool;

    /// Get all current recovery states
    fn get_recovery_states(&self) -> Vec<RecoveryState>;

    /// Get recovery state for a specific subsystem
    fn get_recovery_state(&self, subsystem_id: &str) -> Option<RecoveryState>;

    /// Manually trigger recovery for a subsystem
    async fn trigger_recovery(&self, subsystem_id: &str) -> crate::domain::error::Result<()>;

    /// Reset recovery state for a subsystem (clear exhausted state)
    fn reset_recovery_state(&self, subsystem_id: &str) -> crate::domain::error::Result<()>;

    /// Register a subsystem for monitoring
    fn register_subsystem(&self, subsystem_id: &str);

    /// Unregister a subsystem from monitoring
    fn unregister_subsystem(&self, subsystem_id: &str);
}

/// Centralized recovery manager for automatic component recovery
///
/// Uses `CancellationToken` for async-native shutdown signaling.
pub struct RecoveryManager {
    /// Configuration for recovery behavior
    config: RecoveryConfig,

    /// Event bus for publishing/subscribing to system events
    event_bus: SharedEventBusProvider,

    /// Recovery state for each monitored subsystem
    recovery_states: Arc<DashMap<String, RecoveryState>>,

    /// Cancellation token for shutdown signaling
    cancel_token: CancellationToken,

    /// Handle to the background recovery task
    task_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new(config: RecoveryConfig, event_bus: SharedEventBusProvider) -> Self {
        Self {
            config,
            event_bus,
            recovery_states: Arc::new(DashMap::new()),
            cancel_token: CancellationToken::new(),
            task_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &RecoveryConfig {
        &self.config
    }

    /// Spawn the background recovery loop
    async fn spawn_recovery_loop(&self) {
        let config = self.config.clone();
        let event_bus = Arc::clone(&self.event_bus);
        let recovery_states = Arc::clone(&self.recovery_states);
        let running = Arc::clone(&self.running);

        let handle = tokio::spawn(async move {
            Self::recovery_loop(config, event_bus, recovery_states, running).await;
        });

        let mut task_guard = self.task_handle.lock().await;
        *task_guard = Some(handle);
    }

    /// Main recovery loop that runs periodic health checks
    async fn recovery_loop(
        config: RecoveryConfig,
        event_bus: SharedEventBusProvider,
        recovery_states: Arc<DashMap<String, RecoveryState>>,
        running: Arc<AtomicBool>,
    ) {
        let interval = Duration::from_secs(config.health_check_interval_secs);
        let mut event_receiver = match event_bus.subscribe().await {
            Ok(receiver) => receiver,
            Err(e) => {
                tracing::error!("[RECOVERY] Failed to subscribe to event bus: {}", e);
                return;
            }
        };

        tracing::info!(
            "[RECOVERY] Recovery manager started with {}s health check interval",
            config.health_check_interval_secs
        );

        while running.load(Ordering::SeqCst) {
            tokio::select! {
                // Handle incoming events
                event = event_receiver.recv() => {
                    if let Ok(event) = event {
                        Self::handle_event(&event, &config, &event_bus, &recovery_states).await;
                    }
                }

                // Periodic health check and recovery
                _ = tokio::time::sleep(interval) => {
                    if config.enabled {
                        Self::process_recovery_cycle(&config, &event_bus, &recovery_states).await;
                    }
                }
            }
        }

        tracing::info!("[RECOVERY] Recovery manager stopped");
    }

    /// Handle system events
    async fn handle_event(
        event: &SystemEvent,
        config: &RecoveryConfig,
        event_bus: &SharedEventBusProvider,
        recovery_states: &DashMap<String, RecoveryState>,
    ) {
        match event {
            SystemEvent::SubsystemHealthCheck { subsystem_id } => {
                tracing::debug!(
                    "[RECOVERY] Health check requested for subsystem: {}",
                    subsystem_id
                );
                Self::check_and_recover_subsystem(subsystem_id, config, event_bus, recovery_states)
                    .await;
            }
            SystemEvent::ProviderRestart {
                provider_type,
                provider_id,
            } => {
                // Record that a restart was triggered externally
                let subsystem_id = format!("{}:{}", provider_type, provider_id);
                if let Some(mut state) = recovery_states.get_mut(&subsystem_id) {
                    tracing::info!(
                        "[RECOVERY] External restart triggered for {}, recording attempt",
                        subsystem_id
                    );
                    state.record_recovery_attempt();
                }
            }
            _ => {}
        }
    }

    /// Process one recovery cycle for all monitored subsystems
    async fn process_recovery_cycle(
        config: &RecoveryConfig,
        event_bus: &SharedEventBusProvider,
        recovery_states: &DashMap<String, RecoveryState>,
    ) {
        let subsystem_ids: Vec<String> = recovery_states
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for subsystem_id in subsystem_ids {
            Self::check_and_recover_subsystem(&subsystem_id, config, event_bus, recovery_states)
                .await;
        }
    }

    /// Check a single subsystem and attempt recovery if needed
    async fn check_and_recover_subsystem(
        subsystem_id: &str,
        config: &RecoveryConfig,
        event_bus: &SharedEventBusProvider,
        recovery_states: &DashMap<String, RecoveryState>,
    ) {
        let policy = config.get_policy(subsystem_id);

        // Get or create recovery state
        let mut state = recovery_states
            .entry(subsystem_id.to_string())
            .or_insert_with(|| RecoveryState::new(subsystem_id.to_string(), policy.max_retries));

        // Check if we should attempt recovery
        if state.should_attempt_recovery(policy) {
            tracing::info!(
                "[RECOVERY] Attempting recovery for {} (retry {}/{})",
                subsystem_id,
                state.current_retry + 1,
                if policy.max_retries == 0 {
                    "∞".to_string()
                } else {
                    policy.max_retries.to_string()
                }
            );

            // Execute recovery based on strategy
            let result = Self::execute_recovery(subsystem_id, &policy.strategy, event_bus).await;

            match result {
                Ok(()) => {
                    state.record_recovery_attempt();
                    tracing::info!(
                        "[RECOVERY] Recovery attempt {} initiated for {}",
                        state.current_retry,
                        subsystem_id
                    );
                }
                Err(e) => {
                    state.record_failure(Some(e.to_string()));
                    tracing::error!("[RECOVERY] Recovery failed for {}: {}", subsystem_id, e);
                }
            }
        }
    }

    /// Execute recovery action based on strategy
    async fn execute_recovery(
        subsystem_id: &str,
        strategy: &RecoveryStrategy,
        event_bus: &SharedEventBusProvider,
    ) -> crate::domain::error::Result<()> {
        match strategy {
            RecoveryStrategy::Restart => {
                // Parse subsystem ID to get provider type and ID
                let parts: Vec<&str> = subsystem_id.splitn(2, ':').collect();
                let (provider_type, provider_id) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (String::new(), subsystem_id.to_string())
                };

                // Publish restart event
                event_bus
                    .publish(SystemEvent::ProviderRestart {
                        provider_type,
                        provider_id,
                    })
                    .await
                    .map_err(|e| crate::domain::error::Error::Internal {
                        message: format!("Failed to publish restart event: {}", e),
                    })?;

                Ok(())
            }
            RecoveryStrategy::Skip => {
                tracing::warn!(
                    "[RECOVERY] Skipping recovery for {} (Skip strategy)",
                    subsystem_id
                );
                Ok(())
            }
            RecoveryStrategy::Alert => {
                tracing::error!(
                    "[RECOVERY] ALERT: Subsystem {} requires manual intervention",
                    subsystem_id
                );
                Ok(())
            }
            RecoveryStrategy::Respawn => {
                tracing::warn!(
                    "[RECOVERY] Escalating to process respawn for {}",
                    subsystem_id
                );
                event_bus.publish(SystemEvent::Respawn).await.map_err(|e| {
                    crate::domain::error::Error::Internal {
                        message: format!("Failed to publish respawn event: {}", e),
                    }
                })?;
                Ok(())
            }
        }
    }

    /// Mark a subsystem as healthy (called when health check succeeds)
    pub fn mark_healthy(&self, subsystem_id: &str) {
        if let Some(mut state) = self.recovery_states.get_mut(subsystem_id) {
            if state.status != RecoveryStatus::Healthy {
                tracing::info!(
                    "[RECOVERY] Subsystem {} recovered successfully",
                    subsystem_id
                );
            }
            state.record_success();
        }
    }

    /// Mark a subsystem as unhealthy (called when health check fails)
    pub fn mark_unhealthy(&self, subsystem_id: &str, error: Option<String>) {
        let policy = self.config.get_policy(subsystem_id);

        let mut state = self
            .recovery_states
            .entry(subsystem_id.to_string())
            .or_insert_with(|| RecoveryState::new(subsystem_id.to_string(), policy.max_retries));

        state.record_failure(error.clone());

        tracing::warn!(
            "[RECOVERY] Subsystem {} marked unhealthy: {:?} (failures: {})",
            subsystem_id,
            error,
            state.consecutive_failures
        );
    }

    /// Get event receiver for external health check integration
    pub async fn subscribe(
        &self,
    ) -> crate::domain::error::Result<Box<dyn crate::infrastructure::events::EventReceiver>> {
        self.event_bus.subscribe().await
    }
}

#[async_trait::async_trait]
impl RecoveryManagerInterface for RecoveryManager {
    async fn start(&self) -> crate::domain::error::Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        self.spawn_recovery_loop().await;

        Ok(())
    }

    async fn stop(&self) -> crate::domain::error::Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(false, Ordering::SeqCst);

        // Wait for the task to complete
        let mut task_guard = self.task_handle.lock().await;
        if let Some(handle) = task_guard.take() {
            handle.abort();
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn get_recovery_states(&self) -> Vec<RecoveryState> {
        self.recovery_states
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    fn get_recovery_state(&self, subsystem_id: &str) -> Option<RecoveryState> {
        self.recovery_states
            .get(subsystem_id)
            .map(|entry| entry.value().clone())
    }

    async fn trigger_recovery(&self, subsystem_id: &str) -> crate::domain::error::Result<()> {
        let policy = self.config.get_policy(subsystem_id);

        // Reset the state to allow recovery
        if let Some(mut state) = self.recovery_states.get_mut(subsystem_id) {
            // Only reset if exhausted
            if state.status == RecoveryStatus::Exhausted {
                state.current_retry = 0;
                state.status = RecoveryStatus::Recovering;
            } else if state.status == RecoveryStatus::Healthy {
                // Force into recovering state for manual trigger
                state.status = RecoveryStatus::Recovering;
            }
        } else {
            // Create new state in recovering status
            let mut state = RecoveryState::new(subsystem_id.to_string(), policy.max_retries);
            state.status = RecoveryStatus::Recovering;
            self.recovery_states.insert(subsystem_id.to_string(), state);
        }

        // Trigger immediate recovery
        Self::check_and_recover_subsystem(
            subsystem_id,
            &self.config,
            &self.event_bus,
            &self.recovery_states,
        )
        .await;

        Ok(())
    }

    fn reset_recovery_state(&self, subsystem_id: &str) -> crate::domain::error::Result<()> {
        if let Some(mut state) = self.recovery_states.get_mut(subsystem_id) {
            state.reset();
            tracing::info!(
                "[RECOVERY] Recovery state reset for subsystem: {}",
                subsystem_id
            );
            Ok(())
        } else {
            Err(crate::domain::error::Error::NotFound {
                resource: format!("subsystem:{}", subsystem_id),
            })
        }
    }

    fn register_subsystem(&self, subsystem_id: &str) {
        let policy = self.config.get_policy(subsystem_id);
        self.recovery_states.insert(
            subsystem_id.to_string(),
            RecoveryState::new(subsystem_id.to_string(), policy.max_retries),
        );
        tracing::debug!("[RECOVERY] Registered subsystem: {}", subsystem_id);
    }

    fn unregister_subsystem(&self, subsystem_id: &str) {
        self.recovery_states.remove(subsystem_id);
        tracing::debug!("[RECOVERY] Unregistered subsystem: {}", subsystem_id);
    }
}

/// Shared RecoveryManager wrapped in Arc for thread-safe sharing
pub type SharedRecoveryManager = Arc<dyn RecoveryManagerInterface>;

/// Create a shared recovery manager
pub fn create_recovery_manager(
    config: RecoveryConfig,
    event_bus: SharedEventBusProvider,
) -> SharedRecoveryManager {
    Arc::new(RecoveryManager::new(config, event_bus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::events::EventBus;

    #[test]
    fn test_recovery_state_transitions() {
        let mut state = RecoveryState::new("test".to_string(), 3);
        assert_eq!(state.status, RecoveryStatus::Healthy);

        // Record failure transitions to Recovering
        state.record_failure(Some("error".to_string()));
        assert_eq!(state.status, RecoveryStatus::Recovering);
        assert_eq!(state.consecutive_failures, 1);

        // Record more failures
        state.record_failure(None);
        state.record_failure(None);
        assert_eq!(state.consecutive_failures, 3);

        // Record recovery attempts until exhausted
        state.record_recovery_attempt();
        assert_eq!(state.current_retry, 1);
        state.record_recovery_attempt();
        assert_eq!(state.current_retry, 2);
        state.record_recovery_attempt();
        assert_eq!(state.current_retry, 3);
        assert_eq!(state.status, RecoveryStatus::Exhausted);

        // Reset clears everything
        state.reset();
        assert_eq!(state.status, RecoveryStatus::Healthy);
        assert_eq!(state.consecutive_failures, 0);
        assert_eq!(state.current_retry, 0);
    }

    #[test]
    fn test_recovery_policy_backoff() {
        let policy = crate::daemon::types::RecoveryPolicy {
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            ..Default::default()
        };

        assert_eq!(policy.calculate_backoff(0), 1000);
        assert_eq!(policy.calculate_backoff(1), 2000);
        assert_eq!(policy.calculate_backoff(2), 4000);
        assert_eq!(policy.calculate_backoff(3), 8000);
        assert_eq!(policy.calculate_backoff(4), 16000);
        assert_eq!(policy.calculate_backoff(5), 30000); // Capped at max
        assert_eq!(policy.calculate_backoff(10), 30000); // Still capped
    }

    #[tokio::test]
    async fn test_recovery_manager_lifecycle() {
        let config = RecoveryConfig::default();
        let event_bus = Arc::new(EventBus::default());
        let manager = RecoveryManager::new(config, event_bus);

        assert!(!manager.is_running());

        manager.start().await.unwrap();
        assert!(manager.is_running());

        manager.stop().await.unwrap();
        // Give a moment for the task to stop
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!manager.is_running());
    }

    #[test]
    fn test_recovery_manager_subsystem_registration() {
        let config = RecoveryConfig::default();
        let event_bus = Arc::new(EventBus::default());
        let manager = RecoveryManager::new(config, event_bus);

        // Register subsystem
        manager.register_subsystem("embedding:ollama");
        assert!(manager.get_recovery_state("embedding:ollama").is_some());

        // Unregister
        manager.unregister_subsystem("embedding:ollama");
        assert!(manager.get_recovery_state("embedding:ollama").is_none());
    }

    #[test]
    fn test_mark_healthy_unhealthy() {
        let config = RecoveryConfig::default();
        let event_bus = Arc::new(EventBus::default());
        let manager = RecoveryManager::new(config, event_bus);

        manager.register_subsystem("test:provider");

        // Mark unhealthy
        manager.mark_unhealthy("test:provider", Some("Connection failed".to_string()));
        let state = manager.get_recovery_state("test:provider").unwrap();
        assert_eq!(state.status, RecoveryStatus::Recovering);
        assert_eq!(state.consecutive_failures, 1);

        // Mark healthy
        manager.mark_healthy("test:provider");
        let state = manager.get_recovery_state("test:provider").unwrap();
        assert_eq!(state.status, RecoveryStatus::Healthy);
        assert_eq!(state.consecutive_failures, 0);
    }
}
