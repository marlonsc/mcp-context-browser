//! Event bus tests
//!
//! Tests migrated from src/infrastructure/events.rs

use mcp_context_browser::infrastructure::events::{
    create_shared_event_bus, EventBus, SystemEvent,
};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_event_bus_publish_subscribe() -> Result<(), Box<dyn std::error::Error>> {
    let bus = EventBus::new(10);
    let mut receiver = bus.subscribe();

    // Publish an event
    let result = bus.publish(SystemEvent::CacheClear { namespace: None });
    assert!(result.is_ok());

    // Receive the event
    let event = timeout(Duration::from_millis(100), receiver.recv())
        .await?
        .map_err(|e| format!("receive error: {}", e))?;

    matches!(event, SystemEvent::CacheClear { namespace: None });
    Ok(())
}

#[tokio::test]
async fn test_event_bus_multiple_subscribers() -> Result<(), Box<dyn std::error::Error>> {
    let bus = EventBus::new(10);
    let mut receiver1 = bus.subscribe();
    let mut receiver2 = bus.subscribe();

    assert_eq!(bus.subscriber_count(), 2);

    bus.publish(SystemEvent::ConfigReloaded)?;

    let event1 = receiver1.recv().await?;
    let event2 = receiver2.recv().await?;

    assert!(matches!(event1, SystemEvent::ConfigReloaded));
    assert!(matches!(event2, SystemEvent::ConfigReloaded));
    Ok(())
}

#[tokio::test]
async fn test_shared_event_bus() -> Result<(), Box<dyn std::error::Error>> {
    let bus = create_shared_event_bus();
    let bus_clone = Arc::clone(&bus);

    let mut receiver = bus.subscribe();

    // Publish from clone
    bus_clone.publish(SystemEvent::Shutdown)?;

    let event = receiver.recv().await?;
    assert!(matches!(event, SystemEvent::Shutdown));
    Ok(())
}
