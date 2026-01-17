//! Event Bus Adapters
//!
//! Provides event bus implementations for the `EventBusProvider` port trait.
//!
//! ## ARCHITECTURE RULE
//!
//! These implementations are **INTERNAL** to the infrastructure layer.
//! External code MUST resolve `Arc<dyn EventBusProvider>` via Shaku DI.
//! NEVER import or use these types directly.
//!
//! ## Available Implementations
//!
//! | Provider | Use Case |
//! |----------|----------|
//! | `TokioBroadcastEventBus` | Production (single process, high performance) |
//! | `NullEventBus` | Testing (no-op) |

use async_trait::async_trait;
use futures::stream;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Default buffer capacity for the broadcast channel
const DEFAULT_CAPACITY: usize = 1024;

// ============================================================================
// TokioBroadcastEventBus - Production Implementation
// ============================================================================

/// Production EventBus using Tokio broadcast channel
///
/// This implementation is INTERNAL. Resolve via Shaku DI as `Arc<dyn EventBusProvider>`.
#[derive(Clone)]
pub(crate) struct TokioBroadcastEventBus {
    sender: Arc<broadcast::Sender<DomainEvent>>,
    capacity: usize,
}

impl TokioBroadcastEventBus {
    /// Create a new event bus with default capacity (1024 events)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
            capacity,
        }
    }
}

impl Default for TokioBroadcastEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TokioBroadcastEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioBroadcastEventBus")
            .field("capacity", &self.capacity)
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

// Shaku Component implementation for DI container
impl<M: shaku::Module> shaku::Component<M> for TokioBroadcastEventBus {
    type Interface = dyn EventBusProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(TokioBroadcastEventBus::new())
    }
}

#[async_trait]
impl EventBusProvider for TokioBroadcastEventBus {
    // ========================================================================
    // High-Level Typed API
    // ========================================================================

    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        match self.sender.send(event) {
            Ok(count) => {
                debug!("Published event to {} subscribers", count);
            }
            Err(_) => {
                // No receivers - not an error, just no one listening
                debug!("Published event but no subscribers");
            }
        }
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        let receiver = self.sender.subscribe();

        // Convert broadcast receiver to a Stream that handles lagged errors
        let stream = stream::unfold(receiver, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => return Some((event, rx)),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Event stream lagged by {} events", n);
                        // Continue receiving
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return None;
                    }
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }

    // ========================================================================
    // Low-Level Raw API
    // ========================================================================

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        // Deserialize payload to DomainEvent
        let event: DomainEvent = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to deserialize event payload for topic '{}': {}", topic, e);
                return Ok(()); // Don't fail on bad payload
            }
        };

        debug!("Publishing event to topic '{}': {:?}", topic, event);
        self.publish_event(event).await
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        // Returns an ID for tracking. Actual streaming is via subscribe_events()
        let id = format!("tokio-broadcast-{}-{}", topic, uuid::Uuid::new_v4());
        debug!("Created subscription: {}", id);
        Ok(id)
    }
}

// ============================================================================
// NullEventBus - Testing Implementation
// ============================================================================

/// Null implementation for testing
///
/// This implementation is INTERNAL. Resolve via Shaku DI as `Arc<dyn EventBusProvider>`.
/// All operations succeed but no events are delivered.
#[allow(dead_code)] // Used in tests and as DI override
pub(crate) struct NullEventBus;

impl NullEventBus {
    #[allow(dead_code)] // Used in tests
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Shaku Component implementation for DI container
impl<M: shaku::Module> shaku::Component<M> for NullEventBus {
    type Interface = dyn EventBusProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(NullEventBus::new())
    }
}

#[async_trait]
impl EventBusProvider for NullEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        // Return an empty stream that never yields
        Ok(Box::pin(stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("null-sub".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use mcb_domain::events::ServiceState;

    #[tokio::test]
    async fn test_tokio_broadcast_publish_subscribe_via_trait() {
        let bus = TokioBroadcastEventBus::new();

        // Subscribe via trait method
        let mut stream = bus.subscribe_events().await.unwrap();

        // Publish via trait method
        let event = DomainEvent::ServiceStateChanged {
            name: "test-service".to_string(),
            state: ServiceState::Running,
            previous_state: Some(ServiceState::Starting),
        };

        bus.publish_event(event.clone()).await.unwrap();

        // Receive via stream
        let received = stream.next().await.unwrap();
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn test_tokio_broadcast_has_subscribers() {
        let bus = TokioBroadcastEventBus::new();

        assert!(!bus.has_subscribers());

        let _stream = bus.subscribe_events().await.unwrap();
        assert!(bus.has_subscribers());
    }

    #[tokio::test]
    async fn test_null_event_bus_via_trait() {
        let bus = NullEventBus::new();

        // All operations should succeed
        assert!(!bus.has_subscribers());

        bus.publish_event(DomainEvent::MetricsSnapshot {
            timestamp: chrono::Utc::now(),
        })
        .await
        .unwrap();

        let mut stream = bus.subscribe_events().await.unwrap();
        // Stream should be empty (returns None immediately)
        assert!(stream.next().await.is_none());
    }
}
