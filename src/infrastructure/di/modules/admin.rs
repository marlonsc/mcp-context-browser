//! Admin Service DI Module Implementation
//!
//! Contains admin service with dependencies on infrastructure and server modules.


use shaku::module;

use super::traits::{
    AdaptersModule, AdminModule, ApplicationModule, InfrastructureModule, ServerModule,
};
use crate::adapters::http_client::HttpClientProvider;
use crate::application::admin::AdminServiceImpl;
use crate::domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::events::EventBusProvider;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;

// Implementation of the AdminModule trait providing administrative services.
// This module provides the main admin service with dependencies on infrastructure components.
//
// Generated components:
// - `AdminServiceImpl`: Core admin service providing configuration, monitoring, and control
//
// Dependencies (from InfrastructureModule):
// - `SystemMetricsCollectorInterface`: System resource monitoring for admin dashboard
// - `ServiceProviderInterface`: Provider management and registry access
// - `EventBusProvider`: Event system for admin notifications and updates
module! {
    pub AdminModuleImpl: AdminModule {
        components = [AdminServiceImpl],
        providers = [],

        use dyn InfrastructureModule {
            components = [dyn SystemMetricsCollectorInterface, dyn ServiceProviderInterface, dyn EventBusProvider],
            providers = []
        },

        use dyn ServerModule {
            components = [dyn PerformanceMetricsInterface, dyn IndexingOperationsInterface],
            providers = []
        },

        use dyn AdaptersModule {
            components = [dyn HttpClientProvider],
            providers = []
        },

        use dyn ApplicationModule {
            components = [dyn crate::domain::ports::SearchServiceInterface],
            providers = []
        }
    }
}
