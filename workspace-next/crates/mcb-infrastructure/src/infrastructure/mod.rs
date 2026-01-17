//! Infrastructure Services
//!
//! Null implementations of infrastructure port traits for testing.
//! The actual port traits are defined in mcb-domain/ports/infrastructure.

pub mod auth;
pub mod events;
pub mod metrics;
pub mod snapshot;
pub mod sync;

// Re-export Null implementations
pub use auth::NullAuthService;
pub use events::NullEventBus;
pub use metrics::NullSystemMetricsCollector;
pub use snapshot::NullSnapshotProvider;
pub use sync::NullSyncProvider;
