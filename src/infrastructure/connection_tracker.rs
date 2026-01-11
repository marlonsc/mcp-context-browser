//! Connection Tracker for Graceful Drain
//!
//! Tracks active requests and provides graceful drain functionality
//! for clean server shutdown or restart.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
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
#[derive(Clone)]
pub struct ConnectionTracker {
    active_requests: Arc<AtomicUsize>,
    draining: Arc<AtomicBool>,
    drain_complete: Arc<Notify>,
    config: ConnectionTrackerConfig,
}

impl ConnectionTracker {
    /// Create a new connection tracker
    pub fn new(config: ConnectionTrackerConfig) -> Self {
        Self {
            active_requests: Arc::new(AtomicUsize::new(0)),
            draining: Arc::new(AtomicBool::new(false)),
            drain_complete: Arc::new(Notify::new()),
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
        if self.draining.load(Ordering::SeqCst) {
            debug!("Request rejected: server is draining");
            return None;
        }

        // Check max connections
        let current = self.active_requests.fetch_add(1, Ordering::SeqCst);
        if current >= self.config.max_connections {
            self.active_requests.fetch_sub(1, Ordering::SeqCst);
            warn!("Request rejected: max connections reached");
            return None;
        }

        debug!("Request started, active: {}", current + 1);
        Some(RequestGuard {
            tracker: self.clone(),
        })
    }

    /// Get the current number of active requests
    pub fn active_count(&self) -> usize {
        self.active_requests.load(Ordering::SeqCst)
    }

    /// Check if the tracker is in draining mode
    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    /// Start draining and wait for active requests to complete
    ///
    /// Returns true if drain completed before timeout, false if timed out
    pub async fn drain(&self, timeout: Option<Duration>) -> bool {
        let timeout = timeout.unwrap_or(self.config.drain_timeout);
        let start = Instant::now();

        // Set draining flag to reject new requests
        self.draining.store(true, Ordering::SeqCst);
        info!(
            "Starting graceful drain with {}s timeout",
            timeout.as_secs()
        );

        // Wait for active requests to complete
        loop {
            let active = self.active_requests.load(Ordering::SeqCst);
            if active == 0 {
                info!("Drain complete: all requests finished");
                return true;
            }

            // Check timeout
            if start.elapsed() >= timeout {
                warn!(
                    "Drain timeout: {} requests still active after {}s",
                    active,
                    timeout.as_secs()
                );
                return false;
            }

            // Wait for notification or short interval
            tokio::select! {
                _ = self.drain_complete.notified() => {
                    debug!("Received drain completion notification");
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    debug!("Drain check: {} active requests", active);
                }
            }
        }
    }

    /// Stop draining mode (e.g., if drain was cancelled)
    pub fn cancel_drain(&self) {
        self.draining.store(false, Ordering::SeqCst);
        info!("Drain cancelled, accepting new requests");
    }

    /// Internal: decrement request count
    fn request_end(&self) {
        let prev = self.active_requests.fetch_sub(1, Ordering::SeqCst);
        debug!("Request ended, active: {}", prev.saturating_sub(1));

        // Notify if we're draining and this was the last request
        if self.draining.load(Ordering::SeqCst) && prev == 1 {
            self.drain_complete.notify_waiters();
        }
    }
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// RAII guard for tracking request lifetime
pub struct RequestGuard {
    tracker: ConnectionTracker,
}

impl RequestGuard {
    /// Get reference to the tracker (for metrics, etc.)
    pub fn tracker(&self) -> &ConnectionTracker {
        &self.tracker
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        self.tracker.request_end();
    }
}

/// Shared connection tracker for use across handlers
pub type SharedConnectionTracker = Arc<ConnectionTracker>;

/// Create a shared connection tracker
pub fn create_shared_tracker(config: ConnectionTrackerConfig) -> SharedConnectionTracker {
    Arc::new(ConnectionTracker::new(config))
}
