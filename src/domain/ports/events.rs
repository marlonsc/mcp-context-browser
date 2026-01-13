//! Event Publisher Domain Port
//!
//! Defines the business contract for publishing system events. This abstraction
//! enables services to publish events without coupling to specific implementations
//! (tokio broadcast, NATS, etc.).

use crate::domain::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::Interface;
use std::sync::Arc;

/// System-wide event types for decoupled service communication
///
/// These events represent domain-level operations that services can publish
/// and subscribe to without direct coupling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainEvent {
    /// Index rebuild requested or completed
    IndexRebuild {
        /// Collection being rebuilt (None = all collections)
        collection: Option<String>,
    },
    /// Sync operation completed
    SyncCompleted {
        /// Path that was synced
        path: String,
        /// Number of files that changed
        files_changed: i32,
    },
    /// Cache invalidation requested
    CacheInvalidate {
        /// Namespace to invalidate (None = all)
        namespace: Option<String>,
    },
    /// Snapshot created for a codebase
    SnapshotCreated {
        /// Root path of the codebase
        root_path: String,
        /// Number of files in snapshot
        file_count: usize,
    },
    /// File changes detected
    FileChangesDetected {
        /// Root path being monitored
        root_path: String,
        /// Number of added files
        added: usize,
        /// Number of modified files
        modified: usize,
        /// Number of removed files
        removed: usize,
    },
}

/// Domain Port for Publishing System Events
///
/// This trait defines the contract for event publishing without coupling to
/// specific implementations. Services use this trait to publish events that
/// other parts of the system can react to.
///
/// # Example
///
/// ```rust,ignore
/// use crate::domain::ports::events::{EventPublisher, DomainEvent};
///
/// async fn notify_index_rebuild(
///     publisher: &dyn EventPublisher,
///     collection: Option<String>,
/// ) -> Result<()> {
///     publisher.publish(DomainEvent::IndexRebuild { collection }).await
/// }
/// ```
#[async_trait]
pub trait EventPublisher: Interface + Send + Sync {
    /// Publish an event to all subscribers
    ///
    /// Returns Ok(()) if the event was successfully published.
    /// Note: "successfully published" means the event was sent, not necessarily
    /// that subscribers received it (depends on implementation guarantees).
    async fn publish(&self, event: DomainEvent) -> Result<()>;

    /// Check if there are any active subscribers
    ///
    /// Returns true if at least one subscriber is listening for events.
    /// Useful for avoiding unnecessary event creation if no one is listening.
    fn has_subscribers(&self) -> bool;
}

/// Shared event publisher for dependency injection
pub type SharedEventPublisher = Arc<dyn EventPublisher>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// Mock event publisher for testing
    struct MockEventPublisher {
        publish_count: AtomicUsize,
        has_subscribers: AtomicBool,
    }

    impl MockEventPublisher {
        fn new(has_subscribers: bool) -> Self {
            Self {
                publish_count: AtomicUsize::new(0),
                has_subscribers: AtomicBool::new(has_subscribers),
            }
        }

        fn get_publish_count(&self) -> usize {
            self.publish_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, _event: DomainEvent) -> Result<()> {
            self.publish_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn has_subscribers(&self) -> bool {
            self.has_subscribers.load(Ordering::Relaxed)
        }
    }

    #[tokio::test]
    async fn test_event_publisher_publish() {
        let publisher = MockEventPublisher::new(true);

        let result = publisher
            .publish(DomainEvent::IndexRebuild { collection: None })
            .await;

        assert!(result.is_ok());
        assert_eq!(publisher.get_publish_count(), 1);
    }

    #[tokio::test]
    async fn test_event_publisher_has_subscribers() {
        let publisher_with_subs = MockEventPublisher::new(true);
        let publisher_without_subs = MockEventPublisher::new(false);

        assert!(publisher_with_subs.has_subscribers());
        assert!(!publisher_without_subs.has_subscribers());
    }

    #[test]
    fn test_domain_event_serialization() {
        let event = DomainEvent::SyncCompleted {
            path: "/test/path".to_string(),
            files_changed: 5,
        };

        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        let deserialized: DomainEvent =
            serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(event, deserialized);
    }
}
