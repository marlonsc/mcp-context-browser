//! Admin Service DI Module Implementation
//!
//! Contains admin service with dependencies on infrastructure and server modules.

use shaku::module;

use super::traits::{AdaptersModule, AdminModule, InfrastructureModule, ServerModule};
use crate::adapters::http_client::HttpClientProvider;
use crate::admin::service::AdminServiceImpl;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::metrics::PerformanceMetricsInterface;
use crate::server::operations::IndexingOperationsInterface;

module! {
    pub AdminModuleImpl: AdminModule {
        components = [AdminServiceImpl],
        providers = [],

        use dyn InfrastructureModule {
            components = [dyn SystemMetricsCollectorInterface, dyn ServiceProviderInterface],
            providers = []
        },

        use dyn ServerModule {
            components = [dyn PerformanceMetricsInterface, dyn IndexingOperationsInterface],
            providers = []
        },

        use dyn AdaptersModule {
            components = [dyn HttpClientProvider],
            providers = []
        }
    }
}
