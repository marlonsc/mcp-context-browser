//! Infrastructure DI Module Implementation
//!
//! Contains system metrics, service providers, event bus, auth, and core infrastructure.


use shaku::module;

use super::traits::InfrastructureModule;
use crate::infrastructure::auth::AuthService;
use crate::infrastructure::di::factory::ServiceProvider;
use crate::infrastructure::events::EventBus;
use crate::infrastructure::metrics::system::SystemMetricsCollector;
use crate::infrastructure::snapshot::SnapshotManager;
use crate::infrastructure::sync::SyncManager;

// Implementation of the InfrastructureModule trait providing core infrastructure services.
// This module provides system metrics, service providers, event bus, authentication,
// snapshot tracking, and synchronization services.
//
// Generated components:
// - `SystemMetricsCollector`: Monitors CPU, memory, disk, and network usage
// - `ServiceProvider`: Factory for creating provider instances with dependency injection
// - `EventBus`: Asynchronous event communication system for system-wide notifications
// - `AuthService`: JWT-based authentication and role-based access control
// - `SnapshotManager`: Codebase snapshot tracking for incremental indexing
// - `SyncManager`: Cross-process synchronization with debouncing
module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            SystemMetricsCollector,
            ServiceProvider,
            EventBus,
            AuthService,
            SnapshotManager,
            SyncManager
        ],
        providers = []
    }
}
