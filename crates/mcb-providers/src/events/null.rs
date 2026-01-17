//! Null Event Bus Provider
//!
//! Testing stub implementation that discards all events.
//!
//! ## Usage
//!
//! ```rust
//! use mcb_providers::events::NullEventBusProvider;
//!
//! let bus = NullEventBusProvider::new();
//! // All events are silently discarded
//! ```

use async_trait::async_trait;
use futures::stream;
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use std::sync::Arc;

/// Null event bus provider for testing
///
/// Discards all published events without any side effects.
/// Useful for testing when event publishing is not relevant.
#[derive(Debug, Default)]
pub struct NullEventBusProvider;

impl NullEventBusProvider {
    /// Create a new null event bus provider
    pub fn new() -> Self {
        Self
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

// Shaku Component implementation for DI container
impl<M: shaku::Module> shaku::Component<M> for NullEventBusProvider {
    type Interface = dyn EventBusProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(NullEventBusProvider::new())
    }
}

#[async_trait]
impl EventBusProvider for NullEventBusProvider {
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

// Keep backward compatibility with old name
pub type NullEventPublisher = NullEventBusProvider;
