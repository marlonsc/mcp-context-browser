//! EventBus configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// EventBus provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventBusProvider {
    /// In-process broadcast channel (Tokio) - default, high performance
    #[default]
    Tokio,
    /// Distributed message queue (NATS) - for multi-process/distributed systems
    Nats,
    /// No-op event bus for testing
    Null,
}

/// EventBus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// EventBus provider to use
    pub provider: EventBusProvider,

    /// Buffer capacity for in-process event bus (Tokio)
    /// Number of events that can be buffered before oldest events are dropped
    pub capacity: usize,

    /// NATS server URL (for NATS provider)
    /// Example: "nats://localhost:4222"
    pub nats_url: Option<String>,

    /// NATS client name (for NATS provider)
    pub nats_client_name: Option<String>,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Reconnection attempts for distributed providers
    pub max_reconnect_attempts: u32,
}

/// Returns default event bus configuration with:
/// - Tokio in-process provider with 1024 event capacity
/// - NATS client name from infrastructure constants
/// - Connection timeout 5s, max 5 reconnect attempts
impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            provider: EventBusProvider::Tokio,
            capacity: 1024,
            nats_url: None,
            nats_client_name: Some(DEFAULT_NATS_CLIENT_NAME.to_string()),
            connection_timeout_ms: 5000,
            max_reconnect_attempts: 5,
        }
    }
}

impl EventBusConfig {
    /// Create config for Tokio broadcast (default)
    pub fn tokio() -> Self {
        Self::default()
    }

    /// Create config for Tokio broadcast with custom capacity
    pub fn tokio_with_capacity(capacity: usize) -> Self {
        Self {
            provider: EventBusProvider::Tokio,
            capacity,
            ..Default::default()
        }
    }

    /// Create config for NATS
    pub fn nats(url: impl Into<String>) -> Self {
        Self {
            provider: EventBusProvider::Nats,
            nats_url: Some(url.into()),
            ..Default::default()
        }
    }

    /// Create config for Null (testing)
    pub fn null() -> Self {
        Self {
            provider: EventBusProvider::Null,
            ..Default::default()
        }
    }
}
