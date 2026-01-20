//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-application.
//!
//! ## ARCHITECTURE RULE
//!
//! **CONCRETE TYPES ARE INTERNAL ONLY.**
//!
//! All implementations are composed in the DI bootstrap module.
//! External code SHOULD use `init_app()` to get an `AppContext` with resolved services.
//! NEVER import concrete types directly from here - use the trait abstractions.
//!
//! ## Exception: Admin Types
//!
//! `AtomicPerformanceMetrics` and `DefaultIndexingOperations` are exported
//! because mcb-server needs them for AdminState. These implement traits from
//! mcb-application but are infrastructure concerns, not external providers.

// Internal modules - implementations NOT exported
pub(crate) mod auth;
pub(crate) mod events;
pub(crate) mod lifecycle;
pub(crate) mod metrics;
pub(crate) mod snapshot;
pub(crate) mod sync;

// Admin module - partially exported for mcb-server
pub mod admin;

// Public data types (NOT implementations) - these are pure DTOs
pub use lifecycle::{ServiceInfo, ServiceManager, ServiceManagerError};

// Admin types - exported for mcb-server AdminState
pub use admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};

// Test utilities - exported only when test-utils feature is enabled
// These are null implementations used for testing infrastructure services
#[cfg(feature = "test-utils")]
pub use auth::NullAuthService;
#[cfg(feature = "test-utils")]
pub use snapshot::NullSnapshotProvider;
#[cfg(feature = "test-utils")]
pub use sync::NullSyncProvider;
