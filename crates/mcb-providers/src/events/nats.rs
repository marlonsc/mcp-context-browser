//! NATS Event Bus Provider
//!
//! Event bus implementation using NATS for distributed event distribution.
//!
//! ## Features
//!
//! - Distributed event broadcasting across multiple processes/nodes
//! - Multiple subscribers support
//! - Configurable subject for event routing
//! - Reconnection support built into async-nats
//!
//! ## Example
//!
//! ```ignore
//! use mcb_providers::events::NatsEventBusProvider;
//!
//! let bus = NatsEventBusProvider::new("nats://localhost:4222").await?;
//!
//! // Subscribe to events
//! let stream = bus.subscribe_events().await?;
//!
//! // Publish events
//! bus.publish_event(DomainEvent::IndexRebuild { collection: None }).await?;
//! ```

use async_nats::Client;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::error::{Error, Result};
use mcb_domain::events::DomainEvent;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Default subject for domain events
const DEFAULT_SUBJECT: &str = "mcb.events";

/// Event bus provider using NATS for distributed systems
///
/// Provides distributed event broadcasting across multiple processes/nodes.
/// Events are serialized to JSON and published to a NATS subject.
///
/// ## Subject
///
/// By default, events are published to `mcb.events`. Use `with_subject`
/// to customize the subject for namespace isolation.
pub struct NatsEventBusProvider {
    /// NATS client
    client: Arc<Client>,
    /// Subject for publishing events
    subject: String,
    /// Active subscriber count (local tracking only)
    subscriber_count: Arc<RwLock<usize>>,
}

impl NatsEventBusProvider {
    /// Create a new NATS event bus provider
    ///
    /// # Arguments
    ///
    /// * `url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Errors
    ///
    /// Returns an error if connection to NATS server fails.
    pub async fn new(url: &str) -> Result<Self> {
        Self::with_subject(url, DEFAULT_SUBJECT).await
    }

    /// Create with custom subject
    ///
    /// # Arguments
    ///
    /// * `url` - NATS server URL
    /// * `subject` - Subject for publishing events
    pub async fn with_subject(url: &str, subject: &str) -> Result<Self> {
        info!("Connecting to NATS server at {}", url);

        let client = async_nats::connect(url).await.map_err(|e| Error::Infrastructure {
            message: format!("Failed to connect to NATS server at {}: {}", url, e),
            source: None,
        })?;

        info!("Connected to NATS server at {}", url);

        Ok(Self {
            client: Arc::new(client),
            subject: subject.to_string(),
            subscriber_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Create with client name for identification
    ///
    /// # Arguments
    ///
    /// * `url` - NATS server URL
    /// * `subject` - Subject for publishing events
    /// * `client_name` - Optional client name for server-side identification
    pub async fn with_options(
        url: &str,
        subject: &str,
        client_name: Option<&str>,
    ) -> Result<Self> {
        info!("Connecting to NATS server at {} with options", url);

        let mut options = async_nats::ConnectOptions::new();

        if let Some(name) = client_name {
            options = options.name(name);
        }

        let client = options.connect(url).await.map_err(|e| Error::Infrastructure {
            message: format!("Failed to connect to NATS server at {}: {}", url, e),
            source: None,
        })?;

        info!("Connected to NATS server at {}", url);

        Ok(Self {
            client: Arc::new(client),
            subject: subject.to_string(),
            subscriber_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Create as Arc for sharing
    pub async fn new_shared(url: &str) -> Result<Arc<Self>> {
        Ok(Arc::new(Self::new(url).await?))
    }

    /// Get the configured subject
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Debug for NatsEventBusProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsEventBusProvider")
            .field("subject", &self.subject)
            .finish()
    }
}

#[async_trait]
impl EventBusProvider for NatsEventBusProvider {
    async fn publish_event(&self, event: DomainEvent) -> Result<()> {
        let payload = serde_json::to_vec(&event).map_err(|e| Error::Infrastructure {
            message: format!("Failed to serialize event: {}", e),
            source: None,
        })?;

        self.client
            .publish(self.subject.clone(), payload.into())
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to publish event to NATS: {}", e),
                source: None,
            })?;

        debug!("Published event to NATS subject '{}'", self.subject);
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        let subscriber = self
            .client
            .subscribe(self.subject.clone())
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to subscribe to NATS subject '{}': {}", self.subject, e),
                source: None,
            })?;

        // Increment subscriber count
        {
            let mut count = self.subscriber_count.write().await;
            *count += 1;
        }

        let subscriber_count = Arc::clone(&self.subscriber_count);

        // Convert NATS messages to DomainEvent stream
        let stream = stream::unfold(
            (subscriber, subscriber_count),
            |(mut sub, count)| async move {
                match sub.next().await {
                    Some(msg) => {
                        match serde_json::from_slice::<DomainEvent>(&msg.payload) {
                            Ok(event) => Some((event, (sub, count))),
                            Err(e) => {
                                warn!("Failed to deserialize NATS message: {}", e);
                                // Skip bad messages and continue
                                Some((
                                    DomainEvent::MetricsSnapshot {
                                        timestamp: chrono::Utc::now(),
                                    },
                                    (sub, count),
                                ))
                            }
                        }
                    }
                    None => {
                        // Decrement subscriber count when stream ends
                        let mut c = count.write().await;
                        *c = c.saturating_sub(1);
                        None
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    fn has_subscribers(&self) -> bool {
        match self.subscriber_count.try_read() {
            Ok(count) => *count > 0,
            Err(_) => false,
        }
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let subject = if topic.is_empty() {
            self.subject.clone()
        } else {
            format!("{}.{}", self.subject, topic)
        };

        self.client
            .publish(subject.clone(), payload.to_vec().into())
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to publish to NATS subject '{}': {}", subject, e),
                source: None,
            })?;

        debug!("Published raw payload to NATS subject '{}'", subject);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<String> {
        let subject = if topic.is_empty() {
            self.subject.clone()
        } else {
            format!("{}.{}", self.subject, topic)
        };

        let _sub = self
            .client
            .subscribe(subject.clone())
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to subscribe to NATS subject '{}': {}", subject, e),
                source: None,
            })?;

        let sub_id = format!("nats-{}-{}", subject, uuid::Uuid::new_v4());
        debug!("Created NATS subscription: {}", sub_id);

        Ok(sub_id)
    }
}

// Keep backward compatibility with old name
pub type NatsEventPublisher = NatsEventBusProvider;
