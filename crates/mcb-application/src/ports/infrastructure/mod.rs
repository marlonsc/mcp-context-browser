//! Infrastructure Ports
//!
//! Re-exports infrastructure port interfaces from mcb-domain for backward compatibility.
//!
//! **Note**: These types are defined in `mcb_domain::ports::infrastructure`. This module
//! provides re-exports for code that historically imported from mcb-application.
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
//! | [`ProviderRouter`] | Provider routing and selection services |

/// Authentication service port - re-exports from mcb-domain
pub mod auth;
/// Event bus provider port - re-exports from mcb-domain
pub mod events;
/// Distributed lock provider port - re-exports from mcb-domain
pub mod lock;
/// System metrics collector port - re-exports from mcb-domain
pub mod metrics;
/// Provider routing and selection port - re-exports from mcb-domain
pub mod routing;
/// Snapshot management infrastructure port - re-exports from mcb-domain
pub mod snapshot;
/// Key-value state store port - re-exports from mcb-domain
pub mod state_store;
/// File synchronization infrastructure port - re-exports from mcb-domain
pub mod sync;

// Re-export infrastructure ports at module level for convenience
pub use mcb_domain::ports::infrastructure::{
    AuthServiceInterface, DomainEventStream, EventBusProvider, LockGuard, LockProvider,
    ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SyncOptions, SyncProvider, SyncResult, SystemMetrics,
    SystemMetricsCollectorInterface,
};
