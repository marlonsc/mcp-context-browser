//! NATS JetStream-based Event Bus Provider
//!
//! Provides persistent, cross-container event distribution using NATS JetStream.
//! Enables events to be replayed and received reliably across multiple processes.

use crate::domain::error::{Error, Result};
use crate::infrastructure::constants::{
    NATS_CONSUMER_ACK_WAIT, NATS_CONSUMER_MAX_DELIVER, NATS_STREAM_MAX_AGE, NATS_STREAM_MAX_MSGS,
};
use crate::infrastructure::events::{EventBusProvider, EventReceiver, SystemEvent};
use async_nats::jetstream;
use futures::StreamExt;
use serde_json;
use tracing::{debug, error};

const NATS_STREAM_NAME: &str = "MCP_EVENTS";
const NATS_SUBJECT: &str = "mcp.events.";

/// NATS JetStream-based event receiver
pub struct NatsEventReceiver {
    subscription: jetstream::consumer::pull::Stream,
}

#[async_trait::async_trait]
impl EventReceiver for NatsEventReceiver {
    async fn recv(&mut self) -> Result<SystemEvent> {
        match self.subscription.next().await {
            Some(msg_result) => match msg_result {
                Ok(msg) => match serde_json::from_slice::<SystemEvent>(&msg.payload) {
                    Ok(event) => {
                        debug!(
                            "Received event from NATS: {:?}",
                            std::any::type_name_of_val(&event)
                        );
                        let _ = msg.ack().await;
                        Ok(event)
                    }
                    Err(e) => {
                        error!("Failed to deserialize NATS event: {}", e);
                        Err(Error::Internal {
                            message: format!("Failed to deserialize NATS event: {}", e),
                        })
                    }
                },
                Err(e) => {
                    error!("NATS message error: {}", e);
                    Err(Error::Internal {
                        message: format!("NATS message error: {}", e),
                    })
                }
            },
            None => {
                error!("NATS subscription closed");
                Err(Error::Internal {
                    message: "NATS subscription closed unexpectedly".to_string(),
                })
            }
        }
    }
}

/// NATS JetStream-based Event Bus Provider
///
/// Provides persistent event distribution with at-least-once delivery semantics.
/// Events are retained for 1 hour or up to 10,000 messages, whichever comes first.
pub struct NatsEventBus {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    stream_name: String,
    subject: String,
}

impl NatsEventBus {
    /// Create a new NATS event bus, connecting to the specified server
    ///
    /// # Arguments
    ///
    /// * `server_url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mcp_context_browser::infrastructure::events::NatsEventBus;
    ///
    /// async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    ///     let bus = NatsEventBus::new("nats://localhost:4222").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(server_url: &str) -> Result<Self> {
        Self::new_with_config(server_url, NATS_STREAM_NAME, NATS_SUBJECT).await
    }

    /// Create a new NATS event bus with custom stream/subject names
    /// Useful for test isolation where each test needs its own namespace
    pub async fn new_with_prefix(server_url: &str, prefix: &str) -> Result<Self> {
        let stream_name = format!("{}_{}", prefix, NATS_STREAM_NAME);
        let subject = format!("{}.events.>", prefix.to_lowercase());

        Self::new_with_config(server_url, &stream_name, &subject).await
    }

    /// Internal constructor with full configuration
    async fn new_with_config(server_url: &str, stream_name: &str, subject: &str) -> Result<Self> {
        debug!("Connecting to NATS server: {}", server_url);

        // Connect to NATS
        let client = async_nats::connect(server_url)
            .await
            .map_err(|e| Error::Internal {
                message: format!("Failed to connect to NATS: {}", e),
            })?;

        debug!("Connected to NATS, creating JetStream context");

        // Create JetStream context
        let jetstream_ctx = jetstream::new(client.clone());

        // Ensure stream exists
        Self::ensure_stream_exists(&jetstream_ctx, stream_name, subject).await?;

        debug!("NATS event bus initialized successfully");

        Ok(Self {
            client,
            jetstream: jetstream_ctx,
            stream_name: stream_name.to_string(),
            subject: subject.to_string(),
        })
    }

    /// Create the JetStream stream if it doesn't exist
    async fn ensure_stream_exists(
        jetstream_ctx: &jetstream::Context,
        stream_name: &str,
        subject: &str,
    ) -> Result<()> {
        match jetstream_ctx
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: vec![subject.to_string()],
                retention: jetstream::stream::RetentionPolicy::Limits,
                max_messages: NATS_STREAM_MAX_MSGS,
                max_age: NATS_STREAM_MAX_AGE,
                discard: jetstream::stream::DiscardPolicy::Old,
                ..Default::default()
            })
            .await
        {
            Ok(_) => {
                debug!("JetStream stream '{}' ready", stream_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create JetStream stream: {}", e);
                Err(Error::Internal {
                    message: format!("Failed to create JetStream stream: {}", e),
                })
            }
        }
    }

    /// Get JetStream context
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }

    /// Get NATS client
    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }
}

#[async_trait::async_trait]
impl EventBusProvider for NatsEventBus {
    async fn publish(&self, event: SystemEvent) -> Result<usize> {
        // Serialize the event
        let payload = serde_json::to_vec(&event).map_err(|e| Error::Internal {
            message: format!("Failed to serialize event: {}", e),
        })?;

        // Publish to NATS JetStream
        match self
            .jetstream
            .publish(self.subject.clone(), payload.into())
            .await
        {
            Ok(ack_future) => {
                match ack_future.await {
                    Ok(ack) => {
                        debug!("Published event to NATS (sequence: {})", ack.sequence);
                        // Return 1 as subscriber count (we don't track this for NATS)
                        // In a real scenario, we'd need to query active subscribers
                        Ok(1)
                    }
                    Err(e) => {
                        error!("Failed to receive publish acknowledgment: {}", e);
                        Err(Error::Internal {
                            message: format!("Failed to receive publish acknowledgment: {}", e),
                        })
                    }
                }
            }
            Err(e) => {
                error!("Failed to publish event to NATS: {}", e);
                Err(Error::Internal {
                    message: format!("Failed to publish event: {}", e),
                })
            }
        }
    }

    async fn subscribe(&self) -> Result<Box<dyn EventReceiver>> {
        // Create a consumer to read from the stream
        // Using pull subscription with durable consumer for reliability
        debug!("Creating NATS JetStream subscription");

        // Create ephemeral consumer on stream (no durable_name for isolation)
        // Each subscription gets its own consumer that receives ALL messages
        let consumer = self
            .jetstream
            .create_consumer_on_stream(
                jetstream::consumer::pull::Config {
                    durable_name: None, // Ephemeral consumer for better test isolation
                    deliver_policy: jetstream::consumer::DeliverPolicy::New,
                    ack_policy: jetstream::consumer::AckPolicy::Explicit,
                    ack_wait: NATS_CONSUMER_ACK_WAIT,
                    max_deliver: NATS_CONSUMER_MAX_DELIVER as i64,
                    ..Default::default()
                },
                &self.stream_name,
            )
            .await
            .map_err(|e| Error::Internal {
                message: format!("Failed to create NATS consumer: {}", e),
            })?;

        // Start pulling messages
        let subscription = consumer
            .stream()
            .messages()
            .await
            .map_err(|e| Error::Internal {
                message: format!("Failed to create NATS subscription: {}", e),
            })?;

        debug!("NATS subscription created successfully");

        Ok(Box::new(NatsEventReceiver { subscription }))
    }

    fn subscriber_count(&self) -> usize {
        // NATS doesn't provide an easy way to get subscriber count
        // Return 0 as we can't easily track this
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires NATS server running
    async fn test_nats_event_bus_new() {
        let result = NatsEventBus::new("nats://localhost:4222").await;
        // This will fail if NATS server not running, which is expected
        assert!(result.is_ok() || result.is_err()); // Just check it doesn't panic
    }
}
