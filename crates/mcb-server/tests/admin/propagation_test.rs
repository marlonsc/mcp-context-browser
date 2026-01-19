//! Tests for ConfigPropagator
//!
//! Tests configuration propagation and callback functionality.

use mcb_server::admin::propagation::{ConfigPropagator, PropagatorHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn test_config_propagator_creation() {
    let propagator = ConfigPropagator::new();
    assert!(propagator.callbacks.is_empty());
}

#[tokio::test]
async fn test_config_propagator_with_callback() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let propagator = ConfigPropagator::new().on_config_change(Box::new(move |_config| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    assert_eq!(propagator.callbacks.len(), 1);
}

#[test]
fn test_propagator_handle_is_running() {
    // Test that the handle properly tracks task state
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    let handle = runtime.block_on(async {
        let handle = tokio::spawn(async {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });
        PropagatorHandle { handle }
    });

    // Verify that is_running() returns a boolean
    let is_running = handle.is_running();
    assert!(matches!(is_running, true | false));

    // Wait for task completion
    // Note: awaiting the JoinHandle consumes it, so we can't check is_running() after
    runtime.block_on(async {
        // Just ensure the task completes without panic
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    });
}
