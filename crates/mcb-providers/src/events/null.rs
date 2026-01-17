//! Null Event Publisher
//!
//! Testing stub implementation that discards all events.
//!
//! ## Usage
//!
//! ```rust
//! use mcb_providers::events::NullEventPublisher;
//!
//! let publisher = NullEventPublisher::new();
//! // All events are silently discarded
//! ```

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::events::domain_events::{DomainEvent, EventPublisher};
use std::sync::Arc;

/// Null event publisher for testing
///
/// Discards all published events without any side effects.
/// Useful for testing when event publishing is not relevant.
#[derive(Debug, Default, shaku::Component)]
#[shaku(interface = EventPublisher)]
pub struct NullEventPublisher;

impl NullEventPublisher {
    /// Create a new null event publisher
    pub fn new() -> Self {
        Self
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl EventPublisher for NullEventPublisher {
    async fn publish(&self, _event: DomainEvent) -> Result<()> {
        // Discard all events
        Ok(())
    }

    fn has_subscribers(&self) -> bool {
        // No subscribers in null implementation
        false
    }
}
