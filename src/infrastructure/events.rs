//! Event Bus System for Decoupled Communication
//!
//! Provides a publish/subscribe event system using tokio::sync::broadcast
//! for decoupling components like AdminService from core logic.

use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver, Sender};

/// System-wide events for internal communication
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Request to clear all caches
    CacheClear {
        /// Optional namespace to clear (None = all)
        namespace: Option<String>,
    },
    /// Request to create a backup
    BackupCreate {
        /// Target path for backup
        path: String,
    },
    /// Request to rebuild the index
    IndexRebuild {
        /// Collection to rebuild (None = all)
        collection: Option<String>,
    },
    /// Configuration was reloaded
    ConfigReloaded,
    /// Server is shutting down
    Shutdown,
    /// Request to reload configuration (SIGHUP)
    Reload,
    /// Request to respawn the server binary (SIGUSR1)
    Respawn,
    /// Binary file was updated, prepare for respawn
    BinaryUpdated {
        /// New binary path
        path: String,
    },
    /// Sync operation completed
    SyncCompleted {
        /// Path that was synced
        path: String,
        /// Number of files that changed
        files_changed: i32,
    },
}

/// Event Bus for publishing and subscribing to system events
#[derive(Clone)]
pub struct EventBus {
    sender: Sender<SystemEvent>,
}

impl EventBus {
    /// Create a new EventBus with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Create a new EventBus with default capacity (100)
    pub fn with_default_capacity() -> Self {
        Self::new(100)
    }

    /// Publish an event to all subscribers
    pub fn publish(
        &self,
        event: SystemEvent,
    ) -> Result<usize, broadcast::error::SendError<SystemEvent>> {
        self.sender.send(event)
    }

    /// Subscribe to receive events
    pub fn subscribe(&self) -> Receiver<SystemEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Shared EventBus wrapped in Arc for thread-safe sharing
pub type SharedEventBus = Arc<EventBus>;

/// Create a shared EventBus
pub fn create_shared_event_bus() -> SharedEventBus {
    Arc::new(EventBus::default())
}
