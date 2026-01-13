//! DI Module Organization - Hierarchical by Domain
//!
//! This module organizes DI components by domain following Clean Architecture:
//!
//! - `AdaptersModule` - External adapters (HTTP clients, providers)
//! - `InfrastructureModule` - Core infrastructure (metrics, service provider)
//! - `ServerModule` - Server components (performance, indexing)
//! - `AdminModule` - Admin service (depends on infrastructure and server)
//!
//! The root `McpModule` composes all domain modules for application use.
//!
//! ## Building the Module
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use mcp_context_browser::infrastructure::di::modules::*;
//!
//! // Build submodules first (those without dependencies)
//! let adapters = Arc::new(AdaptersModuleImpl::builder().build());
//! let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
//! let server = Arc::new(ServerModuleImpl::builder().build());
//!
//! // Build admin module with its dependencies
//! let admin = Arc::new(
//!     AdminModuleImpl::builder(infrastructure.clone(), server.clone(), adapters.clone())
//!         .build()
//! );
//!
//! // Build root module with all submodules
//! let module = McpModule::builder(adapters, infrastructure, server, admin).build();
//! ```

mod adapters;
mod admin;
mod infrastructure;
mod server;
pub mod traits;

pub use adapters::AdaptersModuleImpl;
pub use admin::AdminModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use server::ServerModuleImpl;
pub use traits::{AdaptersModule, AdminModule, InfrastructureModule, ServerModule};

use shaku::module;

use crate::adapters::http_client::HttpClientProvider;
use crate::admin::service::AdminService;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::metrics::PerformanceMetricsInterface;
use crate::server::operations::IndexingOperationsInterface;

// Root module that composes all domain modules.
//
// Use this module for production initialization. Submodules can be used
// independently for testing or partial initialization.
module! {
    pub McpModule {
        components = [],
        providers = [],

        use dyn AdaptersModule {
            components = [dyn HttpClientProvider],
            providers = []
        },

        use dyn InfrastructureModule {
            components = [dyn SystemMetricsCollectorInterface, dyn ServiceProviderInterface],
            providers = []
        },

        use dyn ServerModule {
            components = [dyn PerformanceMetricsInterface, dyn IndexingOperationsInterface],
            providers = []
        },

        use dyn AdminModule {
            components = [dyn AdminService],
            providers = []
        }
    }
}
