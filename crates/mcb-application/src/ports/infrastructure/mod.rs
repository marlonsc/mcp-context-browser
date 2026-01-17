//! Infrastructure Ports
//!
//! Ports for infrastructure services that provide technical capabilities
//! to the domain. These interfaces define contracts for file synchronization,
//! snapshot management, and other cross-cutting infrastructure concerns.
//!
//! ## Infrastructure Ports
//!
//! | Port | Description |
//! |------|-------------|
//! | [`SyncCoordinator`] | File system synchronization services |
//! | [`SnapshotProvider`] | Codebase snapshot management |
//! | [`AuthServiceInterface`] | Authentication and token services |
//! | [`EventBusProvider`] | Event publish/subscribe services |
//! | [`SystemMetricsCollectorInterface`] | System metrics collection |
//! | [`LockProvider`] | Distributed lock coordination |
//! | [`StateStoreProvider`] | Key-value state persistence |

/// Authentication service port
pub mod auth;
/// Event bus provider port
pub mod events;
/// Distributed lock provider port
pub mod lock;
/// System metrics collector port
pub mod metrics;
/// Snapshot management infrastructure port
pub mod snapshot;
/// Key-value state store port
pub mod state_store;
/// File synchronization infrastructure port
pub mod sync;

// Re-export infrastructure ports
pub use auth::AuthServiceInterface;
pub use events::{DomainEventStream, EventBusProvider};
pub use lock::{LockGuard, LockProvider};
pub use metrics::{SystemMetrics, SystemMetricsCollectorInterface};
pub use snapshot::{SnapshotProvider, SyncProvider};
pub use state_store::StateStoreProvider;
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
