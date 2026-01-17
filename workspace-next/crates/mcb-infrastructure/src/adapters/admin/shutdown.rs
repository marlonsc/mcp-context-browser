//! Shutdown Coordinator Implementation
//!
//! Provides graceful shutdown coordination for the MCP server.
//! Components can subscribe to shutdown signals and be notified
//! when the server is shutting down.

use mcb_domain::ports::admin::ShutdownCoordinator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

/// Shutdown signal sender type
pub type ShutdownSender = broadcast::Sender<()>;

/// Shutdown signal receiver type
pub type ShutdownReceiver = broadcast::Receiver<()>;

/// Default shutdown coordinator implementation
///
/// Uses a broadcast channel to signal shutdown to all subscribers.
/// Thread-safe and can be cloned/shared across components.
pub struct DefaultShutdownCoordinator {
    sender: ShutdownSender,
    shutting_down: Arc<AtomicBool>,
}

impl DefaultShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            sender,
            shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create an Arc-wrapped coordinator for sharing across components
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Subscribe to shutdown signals
    ///
    /// Returns a receiver that will receive a signal when shutdown is initiated.
    pub fn subscribe(&self) -> ShutdownReceiver {
        self.sender.subscribe()
    }

    /// Wait for shutdown with a timeout
    ///
    /// Returns true if shutdown was signaled, false if timeout occurred.
    pub async fn wait_for_shutdown_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> bool {
        let mut rx = self.subscribe();
        tokio::select! {
            _ = rx.recv() => true,
            _ = tokio::time::sleep(timeout) => false,
        }
    }
}

impl Default for DefaultShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownCoordinator for DefaultShutdownCoordinator {
    fn signal_shutdown(&self) {
        if !self.shutting_down.swap(true, Ordering::SeqCst) {
            info!("Shutdown signaled");
            // Ignore send errors (no receivers is okay)
            let _ = self.sender.send(());
        }
    }

    fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }
}

impl Clone for DefaultShutdownCoordinator {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            shutting_down: Arc::clone(&self.shutting_down),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_coordinator_creation() {
        let coordinator = DefaultShutdownCoordinator::new();
        assert!(!coordinator.is_shutting_down());
    }

    #[test]
    fn test_shutdown_signal() {
        let coordinator = DefaultShutdownCoordinator::new();

        assert!(!coordinator.is_shutting_down());
        coordinator.signal_shutdown();
        assert!(coordinator.is_shutting_down());
    }

    #[test]
    fn test_shutdown_signal_idempotent() {
        let coordinator = DefaultShutdownCoordinator::new();

        // Multiple signals should not panic
        coordinator.signal_shutdown();
        coordinator.signal_shutdown();
        coordinator.signal_shutdown();

        assert!(coordinator.is_shutting_down());
    }

    #[tokio::test]
    async fn test_shutdown_subscribe() {
        let coordinator = DefaultShutdownCoordinator::new();
        let mut rx = coordinator.subscribe();

        // Spawn a task to signal shutdown
        let coord_clone = coordinator.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            coord_clone.signal_shutdown();
        });

        // Wait for signal
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv(),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_shutdown_timeout_signal() {
        let coordinator = DefaultShutdownCoordinator::new();

        // Spawn a task to signal shutdown
        let coord_clone = coordinator.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            coord_clone.signal_shutdown();
        });

        let received = coordinator
            .wait_for_shutdown_timeout(tokio::time::Duration::from_millis(100))
            .await;

        assert!(received);
    }

    #[tokio::test]
    async fn test_wait_for_shutdown_timeout_expires() {
        let coordinator = DefaultShutdownCoordinator::new();

        // No signal will be sent
        let received = coordinator
            .wait_for_shutdown_timeout(tokio::time::Duration::from_millis(10))
            .await;

        assert!(!received);
    }

    #[test]
    fn test_shutdown_coordinator_clone() {
        let coord1 = DefaultShutdownCoordinator::new();
        let coord2 = coord1.clone();

        // Signal on one should affect the other
        coord1.signal_shutdown();
        assert!(coord2.is_shutting_down());
    }
}
