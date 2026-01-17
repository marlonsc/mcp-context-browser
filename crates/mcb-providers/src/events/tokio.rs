//! Tokio Broadcast Event Bus Provider
//!
//! Event bus implementation using tokio broadcast channels for
//! in-process event distribution.
//!
//! ## Features
//!
//! - In-process event broadcasting
//! - Multiple subscribers support
//! - Configurable channel capacity
//! - No persistence (events are ephemeral)
//!
//! ## Example
//!
//! ```ignore
//! use mcb_providers::events::TokioEventBusProvider;
//!
//! let bus = TokioEventBusProvider::new();
//!
//! // Subscribe to events
//! let stream = bus.subscribe_events().await?;
//!
//! // Publish events
//! bus.publish_event(DomainEvent::IndexRebuild { collection: None }).await?;
//! ```

use async_trait::async_trait;
use futures::stream;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Default channel capacity
const DEFAULT_CAPACITY: usize = 1024;

/// Event bus provider using tokio broadcast channels
///
/// Provides in-process event distribution with multiple subscribers.
/// Events are broadcast to all active subscribers without persistence.
///
/// ## Capacity
///
/// When the channel is full, the oldest events are dropped. Configure
/// capacity based on expected event volume and subscriber processing speed.
#[derive(Clone)]
pub struct TokioEventBusProvider {
    /// Broadcast sender for publishing events
    sender: Arc<broadcast::Sender<DomainEvent>>,
    /// Channel capacity
    capacity: usize,
}

impl TokioEventBusProvider {
    /// Create a new tokio event bus with default capacity (1024)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create with custom capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events in the channel buffer
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
            capacity,
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Get the current number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for TokioEventBusProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TokioEventBusProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioEventBusProvider")
            .field("capacity", &self.capacity)
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

// Shaku Component implementation for DI container
impl<M: shaku::Module> shaku::Component<M> for TokioEventBusProvider {
    type Interface = dyn EventBusProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(TokioEventBusProvider::new())
    }
}

#[async_trait]
impl EventBusProvider for TokioEventBusProvider {
    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        match self.sender.send(event) {
            Ok(count) => {
                debug!("Published event to {} subscribers", count);
            }
            Err(_) => {
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

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        // Deserialize payload to DomainEvent
        let event: DomainEvent = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to deserialize event payload for topic '{}': {}", topic, e);
                return Ok(());
            }
        };

        debug!("Publishing event to topic '{}': {:?}", topic, event);
        self.publish_event(event).await
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        let id = format!("tokio-broadcast-{}-{}", topic, uuid::Uuid::new_v4());
        debug!("Created subscription: {}", id);
        Ok(id)
    }
}

// Keep backward compatibility with old name
pub type TokioEventPublisher = TokioEventBusProvider;
