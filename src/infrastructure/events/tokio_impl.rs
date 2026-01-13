//! Tokio-based EventBus implementation
//!
//! Provides in-process event publishing and subscription using tokio::sync::broadcast

use super::{EventBusProvider, EventReceiver, SystemEvent};
use crate::domain::error::Result;
use crate::domain::ports::events::{DomainEvent, EventPublisher};
use crate::infrastructure::constants::EVENT_BUS_TOKIO_CAPACITY;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::debug;

/// Event Bus for publishing and subscribing to system events using tokio broadcast
#[derive(Clone)]
pub struct EventBus {
    sender: Sender<SystemEvent>,
}

impl EventBus {
    /// Create a new EventBus with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Create a new EventBus with default capacity
    pub fn with_default_capacity() -> Self {
        Self::new(EVENT_BUS_TOKIO_CAPACITY)
    }

    /// Publish an event (synchronous version for sync contexts)
    ///
    /// This is thread-safe and can be called from non-async code.
    /// Returns the number of receivers that got the event, or 0 if all channels are closed.
    pub fn publish_sync(&self, event: SystemEvent) -> std::result::Result<usize, String> {
        self.sender
            .send(event)
            .map_err(|_| "Event channel closed".to_string())
    }

    /// Subscribe to receive events (synchronous version for sync contexts)
    ///
    /// Returns a receiver that can be used in an async context.
    /// Typically used in sync code that spawns an async task:
    /// ```ignore
    /// let receiver = event_bus.subscribe_sync();
    /// tokio::spawn(async move {
    ///     // use receiver here
    /// });
    /// ```
    pub fn subscribe_sync(&self) -> Receiver<SystemEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Create a shared EventBus that implements EventBusProvider
///
/// Returns a trait object so it can be used wherever Arc<dyn EventBusProvider> is expected.
/// This enables DI patterns where different implementations (Tokio, NATS, Kafka) can be swapped.
pub fn create_shared_event_bus() -> Arc<dyn super::EventBusProvider> {
    Arc::new(EventBus::default())
}

/// EventBus implementation using tokio broadcast
#[async_trait::async_trait]
impl EventBusProvider for EventBus {
    async fn publish(&self, event: SystemEvent) -> Result<usize> {
        Ok(self.sender.send(event).unwrap_or(0))
    }

    async fn subscribe(&self) -> Result<Box<dyn EventReceiver>> {
        Ok(Box::new(TokioEventReceiver {
            receiver: self.sender.subscribe(),
        }))
    }

    fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Event receiver using tokio broadcast channel
pub struct TokioEventReceiver {
    receiver: Receiver<SystemEvent>,
}

/// Tokio-based event receiver implementation
#[async_trait::async_trait]
impl EventReceiver for TokioEventReceiver {
    async fn recv(&mut self) -> Result<SystemEvent> {
        Ok(self.receiver.recv().await?)
    }
}

/// EventPublisher implementation for EventBus
///
/// Maps DomainEvent variants to SystemEvent variants where applicable.
/// Events without direct SystemEvent equivalents are logged but not published.
#[async_trait]
impl EventPublisher for EventBus {
    async fn publish(&self, event: DomainEvent) -> Result<()> {
        // Map DomainEvent to SystemEvent where applicable
        let system_event = match event {
            DomainEvent::CacheInvalidate { namespace } => Some(SystemEvent::CacheClear { namespace }),
            DomainEvent::IndexRebuild { collection: _ } => {
                debug!("IndexRebuild event received - no SystemEvent mapping");
                None
            }
            DomainEvent::SyncCompleted {
                path: _,
                files_changed: _,
            } => {
                debug!("SyncCompleted event received - no SystemEvent mapping");
                None
            }
            DomainEvent::SnapshotCreated {
                root_path: _,
                file_count: _,
            } => {
                debug!("SnapshotCreated event received - no SystemEvent mapping");
                None
            }
            DomainEvent::FileChangesDetected {
                root_path: _,
                added: _,
                modified: _,
                removed: _,
            } => {
                debug!("FileChangesDetected event received - no SystemEvent mapping");
                None
            }
        };

        if let Some(sys_event) = system_event {
            let _ = self.sender.send(sys_event);
        }

        Ok(())
    }

    fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(10);
        let mut receiver = bus.sender.subscribe();

        // Send event directly without unnecessary clone
        let _ = bus.sender.send(SystemEvent::Reload);

        let received = receiver.recv().await.unwrap();
        assert!(matches!(received, SystemEvent::Reload));
    }

    #[tokio::test]
    async fn test_event_bus_provider_trait() {
        let bus: Arc<dyn EventBusProvider> = Arc::new(EventBus::new(10));

        // Subscribe BEFORE publishing to avoid race condition
        let mut receiver = bus.subscribe().await.unwrap();

        let event = SystemEvent::Shutdown;
        let _ = bus.publish(event).await;

        let received = receiver.recv().await.unwrap();
        assert!(matches!(received, SystemEvent::Shutdown));
    }
}
