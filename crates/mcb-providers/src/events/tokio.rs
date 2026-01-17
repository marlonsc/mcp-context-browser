//! Tokio Broadcast Event Publisher
//!
//! Event publisher implementation using tokio broadcast channels for
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
//! use mcb_providers::events::TokioEventPublisher;
//!
//! let publisher = TokioEventPublisher::new();
//! let mut subscriber = publisher.subscribe();
//!
//! publisher.publish(DomainEvent::IndexRebuild { collection: None }).await?;
//!
//! // In another task
//! while let Ok(event) = subscriber.recv().await {
//!     println!("Received: {:?}", event);
//! }
//! ```

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::events::domain_events::{DomainEvent, EventPublisher};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Default channel capacity
const DEFAULT_CAPACITY: usize = 100;

/// Event publisher using tokio broadcast channels
///
/// Provides in-process event distribution with multiple subscribers.
/// Events are broadcast to all active subscribers without persistence.
///
/// ## Capacity
///
/// When the channel is full, the oldest events are dropped. Configure
/// capacity based on expected event volume and subscriber processing speed.
pub struct TokioEventPublisher {
    /// Broadcast sender for publishing events
    sender: broadcast::Sender<DomainEvent>,
}

impl TokioEventPublisher {
    /// Create a new tokio event publisher with default capacity (100)
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
        Self { sender }
    }

    /// Get a subscriber receiver for listening to events
    ///
    /// Each subscriber receives all events published after subscribing.
    /// Multiple subscribers can receive the same events independently.
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
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

impl Default for TokioEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for TokioEventPublisher {
    async fn publish(&self, event: DomainEvent) -> Result<()> {
        // Broadcast to all subscribers, ignore errors if no subscribers
        let _ = self.sender.send(event);
        Ok(())
    }

    fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }
}

impl std::fmt::Debug for TokioEventPublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioEventPublisher")
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}
