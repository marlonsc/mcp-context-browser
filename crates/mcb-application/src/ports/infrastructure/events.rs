//! Event Bus Provider Port
//!
//! Defines the contract for event publish/subscribe services.

use async_trait::async_trait;
use mcb_domain::error::Result;
use shaku::Interface;

/// Event bus provider interface for pub/sub
#[async_trait]
pub trait EventBusProvider: Interface + Send + Sync {
    /// Publish an event to a topic
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Subscribe to events on a topic (returns subscription ID)
    async fn subscribe(&self, topic: &str) -> Result<String>;
}
