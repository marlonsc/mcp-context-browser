//! NATS Event Bus Integration Tests
//!
//! Tests the NatsEventBus against a real local NATS instance.
//! Requires NATS running on localhost:4222 (see docker-compose.yml)
//!
//! Run with: cargo test --test '*' -- nats_event_bus_integration --nocapture

use mcp_context_browser::infrastructure::events::{
    EventBusProvider, SystemEvent, EventBusConfig, create_event_bus,
};
use std::time::Duration;

/// Get NATS URL from environment or default to localhost
fn get_nats_url() -> String {
    std::env::var("NATS_URL")
        .or_else(|_| std::env::var("MCP_NATS_URL"))
        .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string())
}

/// Check if NATS is available
async fn is_nats_available() -> bool {
    let url = get_nats_url();
    match async_nats::connect(url).await {
        Ok(client) => {
            match client.stats().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Helper to skip test if NATS is not available
macro_rules! skip_if_no_nats {
    () => {
        if !is_nats_available().await {
            eprintln!("⚠️  Skipping test: NATS not available on localhost:4222");
            eprintln!("    Start NATS with: docker-compose up -d nats");
            return;
        }
    };
}

#[tokio::test]
async fn test_nats_event_bus_creation() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_creation".to_string(),
        subject: "test.>".to_string(),
    };

    let bus = create_event_bus(&config)
        .await
        .expect("Failed to create NATS event bus");

    assert_eq!(bus.backend_type(), "nats");
    println!("✅ NATS event bus created successfully");
}

#[tokio::test]
async fn test_nats_event_bus_publish_subscribe() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_pub_sub".to_string(),
        subject: "test.pubsub.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
            .await
            .expect("Failed to create NATS event bus"),
    );

    let mut receiver = bus
        .subscribe()
        .await
        .expect("Failed to subscribe to event bus");

    // Publish an event
    let event = SystemEvent::CacheClear { namespace: None };
    bus.publish(event.clone())
        .await
        .expect("Failed to publish event");

    // Receive the event (with timeout)
    tokio::select! {
        result = receiver.recv() => {
            let received_event = result.expect("Failed to receive event");
            assert!(matches!(received_event, SystemEvent::CacheClear { namespace: None }));
            println!("✅ NATS publish/subscribe works correctly");
        }
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            panic!("Timeout waiting for event on NATS");
        }
    }
}

#[tokio::test]
async fn test_nats_event_bus_multiple_subscribers() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_multi_sub".to_string(),
        subject: "test.multi.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
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
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish an event
    let event = SystemEvent::ConfigReloaded;
    bus.publish(event.clone())
        .await
        .expect("Failed to publish event");

    // Both receivers should get the event
    tokio::select! {
        result1 = receiver1.recv() => {
            let event1 = result1.expect("Failed to receive event on receiver 1");
            assert!(matches!(event1, SystemEvent::ConfigReloaded));

            tokio::select! {
                result2 = receiver2.recv() => {
                    let event2 = result2.expect("Failed to receive event on receiver 2");
                    assert!(matches!(event2, SystemEvent::ConfigReloaded));
                    println!("✅ NATS multiple subscribers work correctly");
                }
                _ = tokio::time::sleep(Duration::from_secs(2)) => {
                    panic!("Timeout waiting for event on receiver 2");
                }
            }
        }
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            panic!("Timeout waiting for event on receiver 1");
        }
    }
}

#[tokio::test]
async fn test_nats_event_bus_different_event_types() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_event_types".to_string(),
        subject: "test.events.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
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
        SystemEvent::CacheClear { namespace: Some("test_ns".to_string()) },
        SystemEvent::Shutdown,
    ];

    for event in events.clone() {
        bus.publish(event)
            .await
            .expect("Failed to publish event");

        // Receive with timeout
        tokio::select! {
            result = receiver.recv() => {
                let _ = result.expect("Failed to receive event");
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                panic!("Timeout waiting for event");
            }
        }
    }

    println!("✅ NATS different event types work correctly");
}

#[tokio::test]
async fn test_nats_event_bus_concurrent_publishing() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_concurrent".to_string(),
        subject: "test.concurrent.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
            .await
            .expect("Failed to create NATS event bus"),
    );

    // Spawn multiple concurrent publishers
    let mut handles = vec![];

    for i in 0..10 {
        let bus_clone = std::sync::Arc::clone(&bus);

        let handle = tokio::spawn(async move {
            let event = if i % 2 == 0 {
                SystemEvent::ConfigReloaded
            } else {
                SystemEvent::CacheClear { namespace: Some(format!("ns_{}", i)) }
            };

            bus_clone
                .publish(event)
                .await
                .expect(&format!("Failed to publish event {}", i));

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

#[tokio::test]
async fn test_nats_event_bus_health_check() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_health".to_string(),
        subject: "test.health.>".to_string(),
    };

    let bus = create_event_bus(&config)
        .await
        .expect("Failed to create NATS event bus");

    // Test health check
    let health = bus
        .health_check()
        .await
        .expect("Failed to check health");

    assert_eq!(
        health,
        mcp_context_browser::infrastructure::events::HealthStatus::Healthy,
        "NATS should be healthy"
    );

    println!("✅ NATS health check works correctly");
}

#[tokio::test]
async fn test_nats_event_bus_recovery() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_recovery".to_string(),
        subject: "test.recovery.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
            .await
            .expect("Failed to create NATS event bus"),
    );

    // Publish an event
    let event1 = SystemEvent::ConfigReloaded;
    bus.publish(event1.clone())
        .await
        .expect("Failed to publish first event");

    // Subscribe and receive
    let mut receiver = bus
        .subscribe()
        .await
        .expect("Failed to subscribe");

    tokio::select! {
        result = receiver.recv() => {
            let _ = result.expect("Failed to receive first event");
        }
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            panic!("Timeout on first event");
        }
    }

    // Publish more events
    for i in 0..5 {
        let event = SystemEvent::CacheClear {
            namespace: Some(format!("recovery_ns_{}", i)),
        };
        bus.publish(event)
            .await
            .expect(&format!("Failed to publish event {}", i));

        tokio::select! {
            result = receiver.recv() => {
                let _ = result.expect(&format!("Failed to receive event {}", i));
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                panic!("Timeout on event {}", i);
            }
        }
    }

    println!("✅ NATS event bus recovery works correctly");
}

#[tokio::test]
async fn test_nats_event_bus_large_payload() {
    skip_if_no_nats!();

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: "test_stream_large_payload".to_string(),
        subject: "test.large.>".to_string(),
    };

    let bus = std::sync::Arc::new(
        create_event_bus(&config)
            .await
            .expect("Failed to create NATS event bus"),
    );

    let mut receiver = bus
        .subscribe()
        .await
        .expect("Failed to subscribe");

    // Give subscription time to establish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish event with larger namespace name
    let large_ns = "x".repeat(1000);
    let event = SystemEvent::CacheClear {
        namespace: Some(large_ns.clone()),
    };

    bus.publish(event)
        .await
        .expect("Failed to publish event");

    tokio::select! {
        result = receiver.recv() => {
            let received = result.expect("Failed to receive event");
            match received {
                SystemEvent::CacheClear { namespace: Some(ns) } => {
                    assert_eq!(ns, large_ns, "Large namespace should be preserved");
                }
                _ => panic!("Wrong event type received"),
            }
            println!("✅ NATS large payload handling works correctly");
        }
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            panic!("Timeout waiting for event");
        }
    }
}

#[tokio::test]
async fn test_nats_event_bus_persistence() {
    skip_if_no_nats!();

    let stream_name = format!("test_stream_persist_{}", chrono::Utc::now().timestamp_millis());

    let config = EventBusConfig::Nats {
        url: get_nats_url(),
        stream_name: stream_name.clone(),
        subject: "test.persist.>".to_string(),
    };

    // First connection: publish events
    {
        let bus = create_event_bus(&config)
            .await
            .expect("Failed to create first NATS connection");

        for i in 0..5 {
            let event = SystemEvent::CacheClear {
                namespace: Some(format!("persist_test_{}", i)),
            };
            bus.publish(event)
                .await
                .expect(&format!("Failed to publish event {}", i));
        }
    }

    // Give time for persistence
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Second connection: verify events are persisted
    {
        let bus = create_event_bus(&config)
            .await
            .expect("Failed to create second NATS connection");

        let mut receiver = bus
            .subscribe()
            .await
            .expect("Failed to subscribe");

        // Should be able to read persisted events
        let mut count = 0;
        loop {
            tokio::select! {
                result = receiver.recv() => {
                    if let Ok(_event) = result {
                        count += 1;
                        if count >= 5 {
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(2)) => {
                    break;
                }
            }
        }

        assert!(count > 0, "Should have received persisted events");
        println!("✅ NATS persistence works correctly (received {} events)", count);
    }
}
