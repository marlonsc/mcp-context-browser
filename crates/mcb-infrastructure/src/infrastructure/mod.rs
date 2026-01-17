//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-application.
//!
//! ## ARCHITECTURE RULE
//!
//! **CONCRETE TYPES ARE INTERNAL ONLY.**
//!
//! All implementations in this module are registered in Shaku DI modules.
//! External code MUST resolve dependencies via `HasComponent::resolve()`.
//! NEVER import concrete types directly - use the trait abstractions.
//!
//! ## Shaku Registration
//!
//! Implementations are registered in `di/modules/`:
//! - `TokioBroadcastEventBus` → `InfrastructureModuleImpl`
//! - `DefaultShutdownCoordinator` → `InfrastructureModuleImpl`
//! - `ServiceManager` → Created via factory with DI-resolved dependencies

// Internal modules - implementations NOT exported
pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod events;
pub(crate) mod lifecycle;
pub(crate) mod metrics;
pub(crate) mod snapshot;
pub(crate) mod sync;

// Re-export ONLY for Shaku module registration (crate-internal)
// These types are NEVER exposed outside the crate - external code MUST use DI
pub(crate) use admin::{NullIndexingOperations, NullPerformanceMetrics};
pub(crate) use auth::NullAuthService;
pub(crate) use events::TokioBroadcastEventBus;
pub(crate) use lifecycle::DefaultShutdownCoordinator;
pub(crate) use metrics::NullSystemMetricsCollector;
pub(crate) use snapshot::NullSnapshotProvider;
pub(crate) use sync::NullSyncProvider;

// NullEventBus is exported for tests and override scenarios within crate
#[cfg(test)]
pub(crate) use events::NullEventBus;

// Public data types (NOT implementations) - these are pure DTOs
pub use lifecycle::{ServiceInfo, ServiceManagerError};
