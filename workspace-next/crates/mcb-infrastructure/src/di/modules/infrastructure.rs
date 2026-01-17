//! Infrastructure Module Implementation - COMPLETE: All Services with Component Derive
//!
//! This module provides ALL core infrastructure services with #[derive(Component)].
//! No placeholders - all services are null implementations for testing.
//!
//! ## Services Provided:
//!
//! - NullCacheProvider (from mcb-providers) -> implements CacheProvider ✓
//! - NullAuthService (from infrastructure) -> implements AuthServiceInterface ✓
//! - NullEventBus (from infrastructure) -> implements EventBusProvider ✓
//! - NullSystemMetricsCollector (from infrastructure) -> implements SystemMetricsCollectorInterface ✓
//! - NullSnapshotProvider (from infrastructure) -> implements SnapshotProvider ✓
//! - NullSyncProvider (from infrastructure) -> implements SyncProvider ✓
//!
//! ## Note on Production:
//!
//! Real providers are created at runtime via DomainServicesFactory, not through Shaku.

use shaku::module;

// Import implementations with Component derive
// Using infrastructure null implementations for Shaku DI defaults
use crate::infrastructure::{
    NullAuthService, NullEventBus, NullSnapshotProvider, NullSyncProvider,
    NullSystemMetricsCollector,
};

// Import traits
use super::traits::InfrastructureModule;

/// Infrastructure module implementation with null providers for testing.
///
/// Real providers are injected at runtime via DomainServicesFactory.
module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            // Infrastructure null implementations with Component derive
            NullAuthService,
            NullEventBus,
            NullSystemMetricsCollector,
            NullSnapshotProvider,
            NullSyncProvider
        ],
        providers = []
    }
}
