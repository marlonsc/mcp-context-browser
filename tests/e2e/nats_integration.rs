//! NATS Event Bus Integration Tests
//!
//! Tests the NatsEventBus against a real local NATS instance with JetStream.
//! Requires NATS running on localhost:4222 with JetStream enabled.
//!
//! Run with: cargo test --test nats_integration -- --nocapture

use mcp_context_browser::infrastructure::events::{EventBusProvider, NatsEventBus, SystemEvent};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

// Counter for unique test prefixes
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Get a unique test prefix for stream isolation
fn unique_test_prefix() -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("TEST_{}", count)
}

/// Get NATS URL from environment or default to localhost
fn get_nats_url() -> String {
    std::env::var("NATS_URL")
        .or_else(|_| std::env::var("MCP_NATS_URL"))
        .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string())
}

/// Check if NATS with JetStream is available
async fn is_nats_available() -> bool {
    let url = get_nats_url();
    let client = match async_nats::connect(&url).await {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Verify JetStream is enabled by querying account info
    let jetstream = async_nats::jetstream::new(client);
    match jetstream.query_account().await {
        Ok(_) => true,
        Err(e) => {
            // If error contains "jetstream", JetStream is not enabled
            let err_str = format!("{:?}", e);
            !err_str.to_lowercase().contains("jetstream")
        }
    }
}

/// Helper to skip test if NATS is not available
macro_rules! skip_if_no_nats {
    () => {
        if !is_nats_available().await {
            eprintln!(
                "⚠️  Skipping test: NATS with JetStream not available on {}",
                get_nats_url()
            );
            eprintln!("    Start NATS with: nats-server --jetstream");
            return;
        }
    };
}

#[tokio::test]
async fn test_nats_event_bus_creation() {
    skip_if_no_nats!();

    let prefix = unique_test_prefix();
    let bus = NatsEventBus::new_with_prefix(&get_nats_url(), &prefix)
        .await
        .expect("Failed to create NATS event bus");

    // Verify we have a working bus by checking subscriber count
    assert_eq!(
        bus.subscriber_count(),
        0,
        "Initial subscriber count should be 0"
    );
    println!("✅ NATS event bus created successfully");
}

#[tokio::test]
async fn test_nats_event_bus_publish_subscribe() {
    skip_if_no_nats!();

    let prefix = unique_test_prefix();
    let bus = std::sync::Arc::new(
        NatsEventBus::new_with_prefix(&get_nats_url(), &prefix)
            .await
            .expect("Failed to create NATS event bus"),
    );

    let mut receiver = bus
        .subscribe()
        .await
        .expect("Failed to subscribe to event bus");

    // Give subscription time to establish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish an event
    let event = SystemEvent::CacheClear { namespace: None };
    let result = bus.publish(event).await.expect("Failed to publish event");

    // Should have at least 1 subscriber receiving the event
    assert!(
        result >= 1,
        "Expected at least 1 subscriber to receive event"
    );

    // Receive the event (with timeout)
    tokio::select! {
        result = receiver.recv() => {
            let received_event = result.expect("Failed to receive event");
            assert!(matches!(received_event, SystemEvent::CacheClear { namespace: None }));
            println!("✅ NATS publish/subscribe works correctly");
        }
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            panic!("Timeout waiting for event on NATS");
        }
    }
}

#[tokio::test]
async fn test_nats_event_bus_multiple_subscribers() {
    skip_if_no_nats!();

    let prefix = unique_test_prefix();
    let bus = std::sync::Arc::new(
        NatsEventBus::new_with_prefix(&get_nats_url(), &prefix)
            .await
            .expect("Failed to create NATS event bus"),
    );

    let mut receiver1 = bus
        .subscribe()
        .await
        .expect("Failed to subscribe (receiver 1)");
    let mut receiver2 = bus
        .subscribe()
        .await
        .expect("Failed to subscribe (receiver 2)");

    // Give subscribers time to establish subscriptions
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Publish an event - both receivers should get it (fan-out with ephemeral consumers)
    let event = SystemEvent::ConfigReloaded;
    bus.publish(event).await.expect("Failed to publish event");

    // Both receivers should get the same event
    let (result1, result2) = tokio::join!(
        async {
            tokio::select! {
                result = receiver1.recv() => {
                    result.expect("Failed to receive event on receiver 1")
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    panic!("Timeout waiting for event on receiver 1");
                }
            }
        },
        async {
            tokio::select! {
                result = receiver2.recv() => {
                    result.expect("Failed to receive event on receiver 2")
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    panic!("Timeout waiting for event on receiver 2");
                }
            }
        }
    );

    assert!(matches!(result1, SystemEvent::ConfigReloaded));
    assert!(matches!(result2, SystemEvent::ConfigReloaded));
    println!("✅ NATS multiple subscribers work correctly (fan-out)");
}

#[tokio::test]
async fn test_nats_event_bus_different_event_types() {
    skip_if_no_nats!();

    let prefix = unique_test_prefix();
    let bus = std::sync::Arc::new(
        NatsEventBus::new_with_prefix(&get_nats_url(), &prefix)
            .await
            .expect("Failed to create NATS event bus"),
    );

    let mut receiver = bus
        .subscribe()
        .await
        .expect("Failed to subscribe to event bus");

    // Give subscription time to establish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish different event types
    let events = vec![
        SystemEvent::ConfigReloaded,
        SystemEvent::CacheClear {
            namespace: Some("test_ns".to_string()),
        },
        SystemEvent::Shutdown,
    ];

    for event in events {
        bus.publish(event).await.expect("Failed to publish event");

        // Receive with timeout
        tokio::select! {
            result = receiver.recv() => {
                let _received = result.expect("Failed to receive event");
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                panic!("Timeout waiting for event");
            }
        }
    }

    println!("✅ NATS different event types work correctly");
}

#[tokio::test]
async fn test_nats_event_bus_concurrent_publishing() {
    skip_if_no_nats!();

    let prefix = unique_test_prefix();
    let bus = std::sync::Arc::new(
        NatsEventBus::new_with_prefix(&get_nats_url(), &prefix)
            .await
            .expect("Failed to create NATS event bus"),
    );

    // Spawn multiple concurrent publishers
    let mut handles = vec![];

    for i in 0..5 {
        let bus_clone = std::sync::Arc::clone(&bus);

        let handle = tokio::spawn(async move {
            let event = if i % 2 == 0 {
                SystemEvent::ConfigReloaded
            } else {
                SystemEvent::CacheClear {
                    namespace: Some(format!("ns_{}", i)),
                }
            };

            bus_clone
                .publish(event)
                .await
                .unwrap_or_else(|_| panic!("Failed to publish event {}", i));

            i
        });

        handles.push(handle);
    }

    // Wait for all publishers
    for handle in handles {
        let result = handle.await.expect("Publisher task panicked");
        println!("  ✓ Publisher {} completed", result);
    }

    println!("✅ NATS concurrent publishing works correctly");
}
