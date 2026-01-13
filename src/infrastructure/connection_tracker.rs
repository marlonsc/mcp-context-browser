//! Connection Tracker for Graceful Drain
//!
//! Tracks active requests and provides graceful drain functionality
//! for clean server shutdown or restart.
//!
//! Uses tokio-util's `TaskTracker` for built-in task lifecycle management
//! and `CancellationToken` for async-native shutdown signaling.

use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tokio_util::task::task_tracker::TaskTrackerToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info, warn};

/// Configuration for connection tracking
#[derive(Debug, Clone)]
pub struct ConnectionTrackerConfig {
    /// Default drain timeout
    pub drain_timeout: Duration,
    /// Maximum concurrent connections to track
    pub max_connections: usize,
}

impl Default for ConnectionTrackerConfig {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_secs(30),
            max_connections: 10000,
        }
    }
}

/// Tracks active connections/requests for graceful shutdown
///
/// Uses `TaskTracker` for built-in task lifecycle management and
/// `CancellationToken` for async-native drain signaling.
#[derive(Clone)]
pub struct ConnectionTracker {
    tracker: TaskTracker,
    cancel_token: CancellationToken,
    config: ConnectionTrackerConfig,
}

impl ConnectionTracker {
    /// Create a new connection tracker
    pub fn new(config: ConnectionTrackerConfig) -> Self {
        Self {
            tracker: TaskTracker::new(),
            cancel_token: CancellationToken::new(),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ConnectionTrackerConfig::default())
    }

    /// Start tracking a new request
    ///
    /// Returns None if draining (new requests should be rejected)
    pub fn request_start(&self) -> Option<RequestGuard> {
        // Reject new requests during drain
        if self.is_draining() {
            debug!("Request rejected: server is draining");
            return None;
        }

        // Check max connections
        let current = self.tracker.len();
        if current >= self.config.max_connections {
            warn!("Request rejected: max connections reached");
            return None;
        }

        // Get a token from TaskTracker - this is an RAII guard that
        // automatically decrements the count when dropped
        let token = self.tracker.token();
        debug!("Request started, active: {}", current + 1);
        Some(RequestGuard { _token: token })
    }

    /// Get the current number of active requests
    pub fn active_count(&self) -> usize {
        self.tracker.len()
    }

    /// Check if the tracker is in draining mode
    pub fn is_draining(&self) -> bool {
        self.tracker.is_closed() || self.cancel_token.is_cancelled()
    }

    /// Start draining and wait for active requests to complete
    ///
    /// Returns true if drain completed before timeout, false if timed out.
    ///
    /// Note: Once closed, the tracker cannot be reopened. Create a new
    /// ConnectionTracker instance if you need to accept requests again.
    pub async fn drain(&self, timeout: Option<Duration>) -> bool {
        let timeout = timeout.unwrap_or(self.config.drain_timeout);

        // Close the tracker to reject new requests
        self.tracker.close();
        self.cancel_token.cancel();

        info!(
            "Starting graceful drain with {}s timeout, {} active requests",
            timeout.as_secs(),
            self.tracker.len()
        );

        // Wait for all tracked tasks to complete with timeout
        tokio::select! {
            _ = self.tracker.wait() => {
                info!("Drain complete: all requests finished");
                true
            }
            _ = tokio::time::sleep(timeout) => {
                warn!(
                    "Drain timeout: {} requests still active after {}s",
                    self.tracker.len(),
                    timeout.as_secs()
                );
                false
            }
        }
    }

    /// Stop draining mode
    ///
    /// **DEPRECATED**: TaskTracker's close() is irreversible.
    /// Create a new ConnectionTracker instance to accept requests again.
    /// This method is kept for API compatibility but has no effect once
    /// `drain()` has been called.
    #[deprecated(
        since = "0.2.0",
        note = "TaskTracker close is irreversible. Create a new ConnectionTracker instead."
    )]
    pub fn cancel_drain(&self) {
        // Note: TaskTracker::close() cannot be undone.
        // CancellationToken also cannot be uncancelled.
        // This method is a no-op for backwards compatibility.
        warn!("cancel_drain() called but tracker close is irreversible. Create a new ConnectionTracker to accept new requests.");
    }
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// RAII guard for tracking request lifetime
///
/// Uses TaskTracker's built-in token which automatically
/// decrements the count when dropped.
pub struct RequestGuard {
    _token: TaskTrackerToken,
}

impl RequestGuard {
    // Note: We no longer expose tracker() since TaskTrackerToken
    // doesn't provide access to the parent tracker.
    // If metrics are needed, query the ConnectionTracker directly.
}

/// Shared connection tracker for use across handlers
pub type SharedConnectionTracker = Arc<ConnectionTracker>;

/// Create a shared connection tracker
pub fn create_shared_tracker(config: ConnectionTrackerConfig) -> SharedConnectionTracker {
    Arc::new(ConnectionTracker::new(config))
}
