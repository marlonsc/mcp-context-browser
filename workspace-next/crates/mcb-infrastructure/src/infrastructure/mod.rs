//! Infrastructure Adapters
//!
//! Null implementations of infrastructure port traits for testing.
//! The actual port traits are defined in mcb-domain/ports/infrastructure.

pub mod auth;
pub mod events;
pub mod metrics;
pub mod snapshot;
pub mod sync;

// Re-export Null adapters
pub use auth::NullAuthService;
pub use events::NullEventBus;
pub use metrics::NullSystemMetricsCollector;
pub use snapshot::NullStateStoreProvider;
pub use sync::NullLockProvider;

// Re-export port traits from mcb-domain for convenience
pub use mcb_domain::ports::infrastructure::{
    AuthServiceInterface, EventBusProvider, LockGuard, LockProvider, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SyncOptions, SyncProvider, SyncResult, SystemMetrics,
    SystemMetricsCollectorInterface,
};
