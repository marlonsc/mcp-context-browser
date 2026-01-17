//! Event Bus Provider Interface

use async_trait::async_trait;
use shaku::Interface;
use mcb_domain::error::Result;

/// Event bus provider interface for pub/sub
#[async_trait]
pub trait EventBusProvider: Interface + Send + Sync {
    /// Publish an event
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Subscribe to events (returns subscription ID)
    async fn subscribe(&self, topic: &str) -> Result<String>;
}

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = EventBusProvider)]
pub struct NullEventBus;

impl NullEventBus {
    pub fn new() -> Self { Self }
}

impl Default for NullEventBus {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl EventBusProvider for NullEventBus {
    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> { Ok(()) }
    async fn subscribe(&self, _topic: &str) -> Result<String> { Ok("null-sub".to_string()) }
}
