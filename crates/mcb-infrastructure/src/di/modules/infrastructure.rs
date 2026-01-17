//! Infrastructure Module Implementation - Production Defaults with DI
//!
//! This module provides core infrastructure services with production-ready defaults.
//! Use `with_component_override` to substitute implementations for testing.
//!
//! ## Services Provided (Production Defaults):
//!
//! | Service | Default | Use Case |
//! |---------|---------|----------|
//! | EventBus | `TokioBroadcastEventBus` | High-performance in-process events |
//! | Auth | `NullAuthService` | Override with real auth in production |
//! | Metrics | `NullSystemMetricsCollector` | Override for monitoring |
//! | Snapshot | `NullSnapshotProvider` | Override for persistence |
//! | Sync | `NullSyncProvider` | Override for distributed sync |
//!
//! ## Testing Override Example:
//!
//! ```ignore
//! use crate::infrastructure::NullEventBusProvider;
//!
//! let module = InfrastructureModuleImpl::builder()
//!     .with_component_override::<dyn EventBusProvider>(Box::new(NullEventBusProvider::new()))
//!     .build();
//! ```

use shaku::module;

// Import event bus from local infrastructure (no mcb_providers dependency)
use crate::infrastructure::TokioBroadcastEventBus;

// Import infrastructure-specific implementations (lifecycle, etc.)
use crate::infrastructure::DefaultShutdownCoordinator;

// Import null implementations for services without production defaults yet
use crate::infrastructure::{
    NullAuthService, NullSnapshotProvider, NullSyncProvider, NullSystemMetricsCollector,
};

// Import traits
use super::traits::InfrastructureModule;

module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            // Production event bus (local infrastructure)
            TokioBroadcastEventBus,
            // Local infrastructure services
            DefaultShutdownCoordinator,
            // Testing defaults (override in production config)
            NullAuthService,
            NullSystemMetricsCollector,
            NullSnapshotProvider,
            NullSyncProvider
        ],
        providers = []
    }
}
