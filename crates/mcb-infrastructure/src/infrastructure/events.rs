//! Event Bus Infrastructure
//!
//! Provides Tokio-based event bus for internal infrastructure use.
//! For testing, use NullEventBusProvider from mcb-providers.

use async_trait::async_trait;
use dill::{component, interface};
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
#[derive(Clone)]
#[component]
#[interface(dyn EventBusProvider)]
pub struct TokioBroadcastEventBus {
    sender: Arc<broadcast::Sender<DomainEvent>>,
}

impl TokioBroadcastEventBus {
    /// Create a new tokio event bus with default capacity (1024)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
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
            .field("subscribers", &self.sender.receiver_count())
            .finish()
    }
}

#[async_trait]
impl EventBusProvider for TokioBroadcastEventBus {
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
        let event: DomainEvent = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                warn!(
                    "Failed to deserialize event payload for topic '{}': {}",
                    topic, e
                );
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
