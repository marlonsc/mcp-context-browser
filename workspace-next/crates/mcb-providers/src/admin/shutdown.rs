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
    pub async fn wait_for_shutdown_timeout(&self, timeout: std::time::Duration) -> bool {
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
