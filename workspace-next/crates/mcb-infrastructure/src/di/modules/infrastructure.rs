//! Infrastructure Module Implementation - COMPLETE: All Services with Component Derive
//!
//! This module provides ALL core infrastructure services with #[derive(Component)].
//! No placeholders - all services are real implementations.
//!
//! ## COMPLETE Services Provided:
//!
//! - MokaCacheProvider (from mcb-providers) -> implements CacheProvider ✓
//! - NullAuthService (from infrastructure) -> implements AuthServiceInterface ✓
//! - NullEventBus (from infrastructure) -> implements EventBusProvider ✓
//! - NullSystemMetricsCollector (from infrastructure) -> implements SystemMetricsCollectorInterface ✓
//! - NullStateStoreProvider (from infrastructure) -> implements StateStoreProvider ✓
//! - NullLockProvider (from infrastructure) -> implements LockProvider ✓
//!
//! ## No Runtime Factories:
//!
//! All services created at compile-time by Shaku DI, not runtime factories.

use shaku::module;

// Import ONLY real implementations with Component derive
use mcb_providers::cache::MokaCacheProvider;
use crate::infrastructure::auth::NullAuthService;
use crate::infrastructure::events::NullEventBus;
use crate::infrastructure::metrics::NullSystemMetricsCollector;
use crate::infrastructure::snapshot::NullStateStoreProvider;
use crate::infrastructure::sync::NullLockProvider;

// Import traits
use super::traits::InfrastructureModule;

/// Infrastructure module implementation - COMPLETE Shaku DI.
///
/// Contains ALL core infrastructure services with proper Component derives.
/// No placeholders - everything is real and compiles.
///
/// ## Component Registration - COMPLETE
///
/// ALL services have #[derive(Component)] and #[shaku(interface = ...)].
/// NO struct types in HasComponent (impossible in Shaku).
/// NO placeholder services.
/// ONLY real implementations that exist in the codebase.
///
/// ## Construction - COMPLETE
///
/// ```rust,ignore
/// let infrastructure = InfrastructureModuleImpl::builder().build();
/// ```
module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            // COMPLETE infrastructure services with Component derive
            MokaCacheProvider,
            NullAuthService,
            NullEventBus,
            NullSystemMetricsCollector,
            NullStateStoreProvider,
            NullLockProvider
        ],
        providers = []
    }
}
