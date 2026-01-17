//! Event Bus Adapter
//!
//! Null implementation of the event bus port for testing.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::EventBusProvider;
use mcb_domain::error::Result;

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = EventBusProvider)]
pub struct NullEventBus;

impl NullEventBus {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBusProvider for NullEventBus {
    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }
    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("null-sub".to_string())
    }
}
