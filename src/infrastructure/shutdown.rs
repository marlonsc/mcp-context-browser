//! Unified Shutdown Coordinator
//!
//! Centralizes all background task lifecycle management using tokio-util primitives:
//! - `CancellationToken` for hierarchical shutdown signaling
//! - `TaskTracker` for tracking and awaiting all spawned tasks
//!
//! ## Usage
//!
//! ```rust,ignore
//! let coordinator = ShutdownCoordinator::new();
//!
//! // Spawn tracked tasks
//! coordinator.spawn("cleanup", async move { /* ... */ });
//! coordinator.spawn("monitoring", async move { /* ... */ });
//!
//! // Get child token for a service
//! let service_token = coordinator.child_token();
//!
//! // Graceful shutdown with timeout
//! let completed = coordinator.shutdown(Duration::from_secs(30)).await;
//! ```

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info, warn};

/// Unified shutdown coordinator for all background tasks
///
/// Provides centralized lifecycle management for background services:
/// - Spawns and tracks all background tasks
/// - Provides hierarchical cancellation via child tokens
/// - Graceful shutdown with configurable timeout
#[derive(Clone)]
pub struct ShutdownCoordinator {
    cancel_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            task_tracker: TaskTracker::new(),
        }
    }

    /// Get a child cancellation token for a service
    ///
    /// Child tokens are cancelled when the parent is cancelled,
    /// enabling hierarchical shutdown.
    pub fn child_token(&self) -> CancellationToken {
        self.cancel_token.child_token()
    }

    /// Get the root cancellation token
    ///
    /// Use this when you need to pass the token directly rather than a child.
    pub fn token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Check if shutdown has been initiated
    pub fn is_shutting_down(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Get the number of active tracked tasks
    pub fn active_tasks(&self) -> usize {
        self.task_tracker.len()
    }

    /// Spawn and track a background task
    ///
    /// The task will be tracked and awaited during shutdown.
    /// Returns a JoinHandle for optional direct awaiting.
    pub fn spawn<F>(&self, name: &'static str, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let tracked = self.task_tracker.track_future(future);
        debug!("[SHUTDOWN] Spawning tracked task: {}", name);
        tokio::spawn(tracked)
    }

    /// Spawn a task that respects cancellation
    ///
    /// Provides both the future and a child cancellation token.
    /// The task should check the token and exit gracefully when cancelled.
    pub fn spawn_cancellable<F, Fut>(
        &self,
        name: &'static str,
        task_fn: F,
    ) -> JoinHandle<Fut::Output>
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let token = self.child_token();
        let future = task_fn(token);
        self.spawn(name, future)
    }

    /// Initiate graceful shutdown
    ///
    /// 1. Cancels all tokens (signals tasks to stop)
    /// 2. Closes the task tracker (prevents new tasks)
    /// 3. Waits for all tasks to complete (with timeout)
    ///
    /// Returns `true` if all tasks completed before timeout, `false` otherwise.
    pub async fn shutdown(&self, timeout: Duration) -> bool {
        info!(
            "[SHUTDOWN] Initiating graceful shutdown with {}s timeout, {} active tasks",
            timeout.as_secs(),
            self.task_tracker.len()
        );

        // Signal all tasks to stop
        self.cancel_token.cancel();

        // Close tracker to prevent new tasks
        self.task_tracker.close();

        // Wait for all tasks with timeout
        tokio::select! {
            _ = self.task_tracker.wait() => {
                info!("[SHUTDOWN] All tasks completed cleanly");
                true
            }
            _ = tokio::time::sleep(timeout) => {
                warn!(
                    "[SHUTDOWN] Timeout reached, {} tasks still active",
                    self.task_tracker.len()
                );
                false
            }
        }
    }

    /// Wait for shutdown signal
    ///
    /// Use this in services to block until shutdown is initiated.
    pub async fn wait_for_shutdown(&self) {
        self.cancel_token.cancelled().await;
    }
}

/// Shared shutdown coordinator for use across the application
pub type SharedShutdownCoordinator = Arc<ShutdownCoordinator>;

/// Create a shared shutdown coordinator
pub fn create_shared_coordinator() -> SharedShutdownCoordinator {
    Arc::new(ShutdownCoordinator::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_coordinator_basic() {
        let coordinator = ShutdownCoordinator::new();
        assert!(!coordinator.is_shutting_down());
        assert_eq!(coordinator.active_tasks(), 0);
    }

    #[tokio::test]
    async fn test_spawn_and_track() {
        let coordinator = ShutdownCoordinator::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        coordinator.spawn("test_task", async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Wait briefly for task to complete
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_cancellable_task() {
        let coordinator = ShutdownCoordinator::new();
        let completed = Arc::new(AtomicUsize::new(0));

        let completed_clone = completed.clone();
        coordinator.spawn_cancellable("cancellable", |token| async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => {
                        completed_clone.fetch_add(1, Ordering::SeqCst);
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                }
            }
        });

        // Give task time to start
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(completed.load(Ordering::SeqCst), 0);

        // Shutdown should trigger cancellation
        let success = coordinator.shutdown(Duration::from_secs(1)).await;
        assert!(success);
        assert_eq!(completed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_child_token_hierarchy() {
        let coordinator = ShutdownCoordinator::new();
        let child = coordinator.child_token();

        assert!(!child.is_cancelled());
        coordinator.shutdown(Duration::from_millis(10)).await;
        assert!(child.is_cancelled());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let coordinator = ShutdownCoordinator::new();

        // Spawn a task that completes quickly
        coordinator.spawn("quick", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        // Shutdown should complete successfully
        let success = coordinator.shutdown(Duration::from_secs(1)).await;
        assert!(success);
        assert!(coordinator.is_shutting_down());
    }
}
