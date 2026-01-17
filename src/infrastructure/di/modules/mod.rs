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

/// Adapters module implementation
mod adapters;
/// Admin module implementation
mod admin;
/// Application module implementation
mod application;
/// Infrastructure module implementation
mod infrastructure;
/// Server module implementation
mod server;
/// Common traits and interfaces for DI modules
pub mod traits;

pub use adapters::AdaptersModuleImpl;
pub use admin::AdminModuleImpl;
pub use application::ApplicationModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use server::ServerModuleImpl;
pub use traits::{
    AdaptersModule, AdminModule, ApplicationModule, InfrastructureModule, ServerModule,
};

// Future module trait exports (v0.3.0+)
#[cfg(feature = "analysis")]
pub use traits::AnalysisModule;
#[cfg(feature = "git")]
pub use traits::GitModule;
#[cfg(feature = "quality")]
pub use traits::QualityModule;

use shaku::module;

use crate::adapters::http_client::HttpClientProvider;
use crate::application::admin::AdminService;
use crate::domain::ports::{
    ChunkRepository, ChunkingOrchestratorInterface, CodeChunker, ContextServiceInterface,
    EmbeddingProvider, IndexingOperationsInterface, IndexingServiceInterface,
    PerformanceMetricsInterface, SearchRepository, SearchServiceInterface, SnapshotProvider,
    SyncProvider, VectorStoreProvider,
};
use crate::infrastructure::auth::AuthServiceInterface;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::events::EventBusProvider;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;

// Root dependency injection module for the MCP Context Browser.
// This module composes all domain modules (adapters, infrastructure, application, server, admin)
// into a single cohesive dependency injection container.
//
// Dependencies:
// - `AdaptersModule`: HTTP clients, providers, repositories
// - `InfrastructureModule`: System metrics, service providers, event bus, auth
// - `ApplicationModule`: Business logic services (context, search, indexing)
// - `ServerModule`: Server-side components (performance metrics, indexing ops)
// - `AdminModule`: Administrative services and interfaces
module! {
    pub McpModule {
        components = [],
        providers = [],

        use dyn AdaptersModule {
            components = [
                dyn HttpClientProvider,
                dyn EmbeddingProvider,
                dyn VectorStoreProvider,
                dyn ChunkRepository,
                dyn SearchRepository
            ],
            providers = []
        },

        use dyn InfrastructureModule {
            components = [
                dyn SystemMetricsCollectorInterface,
                dyn ServiceProviderInterface,
                dyn EventBusProvider,
                dyn AuthServiceInterface,
                dyn SnapshotProvider,
                dyn SyncProvider
            ],
            providers = []
        },

        use dyn ServerModule {
            components = [dyn PerformanceMetricsInterface, dyn IndexingOperationsInterface],
            providers = []
        },

        use dyn AdminModule {
            components = [dyn AdminService],
            providers = []
        },

        use dyn ApplicationModule {
            components = [
                dyn ContextServiceInterface,
                dyn SearchServiceInterface,
                dyn IndexingServiceInterface,
                dyn ChunkingOrchestratorInterface,
                dyn CodeChunker
            ],
            providers = []
        }
    }
}
