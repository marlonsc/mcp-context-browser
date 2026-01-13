//! Connection tracker tests
//!
//! Tests migrated from src/infrastructure/connection_tracker.rs

use mcp_context_browser::infrastructure::connection_tracker::{
    ConnectionTracker, ConnectionTrackerConfig,
};
use std::time::Duration;

#[tokio::test]
async fn test_connection_tracker_basic() {
    let tracker = ConnectionTracker::with_defaults();
    assert_eq!(tracker.active_count(), 0);
    assert!(!tracker.is_draining());

    // Start a request
    let guard = tracker.request_start().expect("Should accept request");
    assert_eq!(tracker.active_count(), 1);

    // Start another request
    let guard2 = tracker.request_start().expect("Should accept request");
    assert_eq!(tracker.active_count(), 2);

    // Drop first guard
    drop(guard);
    assert_eq!(tracker.active_count(), 1);

    // Drop second guard
    drop(guard2);
    assert_eq!(tracker.active_count(), 0);
}

#[tokio::test]
async fn test_connection_tracker_drain() {
    let tracker = ConnectionTracker::with_defaults();

    // Start some requests
    let _guard1 = tracker.request_start().expect("Should accept request");
    let _guard2 = tracker.request_start().expect("Should accept request");
    assert_eq!(tracker.active_count(), 2);

    // Start draining in background
    let tracker_clone = tracker.clone();
    let drain_handle =
        tokio::spawn(async move { tracker_clone.drain(Some(Duration::from_secs(5))).await });

    // New requests should be rejected during drain
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(tracker.is_draining());
    assert!(tracker.request_start().is_none());

    // Complete the requests
    drop(_guard1);
    drop(_guard2);

    // Drain should complete successfully
    let result = drain_handle.await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_connection_tracker_drain_timeout() {
    let config = ConnectionTrackerConfig {
        drain_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let tracker = ConnectionTracker::new(config);

    // Start a request but don't complete it
    let _guard = tracker.request_start().expect("Should accept request");

    // Drain should timeout
    let result = tracker.drain(None).await;
    assert!(!result);
    assert_eq!(tracker.active_count(), 1);
}

#[tokio::test]
async fn test_connection_tracker_cancel_drain_deprecated() {
    // Note: cancel_drain() is deprecated because TaskTracker's close()
    // is irreversible. This test verifies the deprecation behavior.
    let tracker = ConnectionTracker::with_defaults();

    // Start draining
    let tracker_inner = tracker.clone();
    tokio::spawn(async move {
        tracker_inner.drain(Some(Duration::from_secs(10))).await;
    });

    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(tracker.is_draining());
    assert!(tracker.request_start().is_none());

    // cancel_drain() is now a no-op due to TaskTracker's irreversible close
    #[allow(deprecated)]
    tracker.cancel_drain();

    // The tracker remains in draining state (cancel has no effect)
    assert!(tracker.is_draining());
    assert!(tracker.request_start().is_none());

    // To accept new requests, create a new tracker
    let new_tracker = ConnectionTracker::with_defaults();
    assert!(!new_tracker.is_draining());
    assert!(new_tracker.request_start().is_some());
}
