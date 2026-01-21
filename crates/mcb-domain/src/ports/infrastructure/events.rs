//! Event Bus Provider Port
//!
//! Defines the contract for event publish/subscribe services.
//!
//! ## Architecture
//!
//! This port defines the abstraction for event-driven communication.
//! Implementations (TokioBroadcastEventBus, NatsEventBus, etc.) are in
//! the infrastructure layer and registered via dill Catalog.
//!
//! ## Usage
//!
//! ```no_run
//! use mcb_domain::ports::infrastructure::EventBusProvider;
//! use mcb_domain::events::DomainEvent;
//! use std::sync::Arc;
//!
//! async fn publish_event(event_bus: Arc<dyn EventBusProvider>) -> mcb_domain::Result<()> {
//!     // Publish typed domain event
//!     let event = DomainEvent::IndexingStarted {
//!         collection: "my-project".to_string(),
//!         total_files: 100,
//!     };
//!     event_bus.publish_event(event).await?;
//!     Ok(())
//! }
//! ```

use crate::error::Result;
use crate::events::DomainEvent;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Boxed async stream of domain events
///
/// This type alias provides an abstract stream type that hides implementation details.
/// Implementations can use any async stream internally (broadcast, mpsc, etc.).
pub type DomainEventStream = Pin<Box<dyn Stream<Item = DomainEvent> + Send + Sync + 'static>>;

/// Event bus provider interface for typed event pub/sub
///
/// This trait defines the contract for event-driven communication in the system.
/// All implementations MUST be resolved via dill Catalog - never instantiate directly.
///
/// ## Methods
///
/// | Method | Purpose |
/// |--------|---------|
/// | `publish_event` | Publish a typed `DomainEvent` |
/// | `subscribe_events` | Get a stream of `DomainEvent` for real-time updates |
/// | `publish` | Low-level: publish raw bytes to a topic |
/// | `subscribe` | Low-level: subscribe to raw topic (returns ID) |
#[async_trait]
pub trait EventBusProvider: Send + Sync {
    // ========================================================================
    // High-Level Typed API (Preferred)
    // ========================================================================

    /// Publish a typed domain event
    ///
    /// This is the preferred method for publishing events. The event will be
    /// serialized and delivered to all subscribers.
    async fn publish_event(&self, event: DomainEvent) -> Result<()>;

    /// Subscribe to receive typed domain events
    ///
    /// Returns a stream that yields `DomainEvent` instances. Use this for
    /// SSE endpoints, WebSocket handlers, or any real-time event consumer.
    ///
    /// The returned stream is `Send + Sync` and can be used across async tasks.
    async fn subscribe_events(&self) -> Result<DomainEventStream>;

    /// Check if there are any active event subscribers
    ///
    /// Useful for avoiding unnecessary event serialization when no one is listening.
    fn has_subscribers(&self) -> bool;

    // ========================================================================
    // Low-Level Raw API (For advanced use cases)
    // ========================================================================

    /// Publish raw bytes to a topic
    ///
    /// Use `publish_event` instead for typed events. This method is for
    /// advanced use cases requiring raw byte payloads.
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Subscribe to events on a topic (returns subscription ID)
    ///
    /// Use `subscribe_events` instead for typed events. This method is for
    /// advanced use cases requiring topic-based filtering.
    async fn subscribe(&self, topic: &str) -> Result<String>;
}
