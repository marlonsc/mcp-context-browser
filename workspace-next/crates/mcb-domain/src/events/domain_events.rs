//! Event Publisher Domain Port
//!
//! Defines the business contract for publishing system events. This abstraction
//! enables services to publish events without coupling to specific implementations
//! (tokio broadcast, NATS, etc.).

use crate::error::Result;
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
/// ```rust,no_run
/// use mcb_domain::events::{EventPublisher, DomainEvent};
///
/// async fn notify_index_rebuild(
///     publisher: &dyn EventPublisher,
///     collection: Option<String>,
/// ) -> mcb_domain::Result<()> {
///     publisher.publish(DomainEvent::IndexRebuild { collection }).await?;
///     Ok(())
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
