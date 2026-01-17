//! DI Module Organization - Hierarchical by Domain (Shaku Strict Pattern)
//!
//! This module implements a strict Shaku-based hierarchical module system
//! following Clean Architecture and Domain-Driven Design principles.
//!
//! ## Shaku Module Hierarchy Pattern
//!
//! ```text
//! McpModule (Root - composes all modules)
//! ├── InfrastructureModule (core services - no dependencies)
//! ├── ServerModule (MCP server components - no dependencies)
//! ├── AdaptersModule (external integrations - no dependencies)
//! ├── ApplicationModule (business logic - placeholder)
//! └── AdminModule (admin services - placeholder)
//! ```
//!
//! ## Note on Current Implementation
//!
//! Many services are created via factory patterns at runtime (see domain_services.rs)
//! rather than through Shaku DI, because they require runtime configuration.
//! The Shaku modules here provide the foundation, with null providers as defaults.
//!
//! ## Module Construction Pattern
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use mcb_infrastructure::di::modules::*;
//!
//! // Build leaf modules
//! let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
//! let server = Arc::new(ServerModuleImpl::builder().build());
//! let adapters = Arc::new(AdaptersModuleImpl::builder().build());
//! let application = Arc::new(ApplicationModuleImpl::builder().build());
//! let admin = Arc::new(AdminModuleImpl::builder().build());
//!
//! // Build root module
//! let root = McpModule::builder(infrastructure, server, adapters, application, admin).build();
//! ```

/// Domain module traits (interfaces)
pub mod traits;

/// Infrastructure module implementation (core infrastructure)
mod infrastructure;
/// Server module implementation (MCP server components)
mod server;
/// Adapters module implementation (external integrations)
mod adapters;
/// Application module implementation (business logic)
mod application;
/// Admin module implementation (admin services)
mod admin;

/// Domain services factory (runtime service creation)
pub mod domain_services;

pub use adapters::AdaptersModuleImpl;
pub use admin::AdminModuleImpl;
pub use application::ApplicationModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use server::ServerModuleImpl;
pub use traits::{
    AdaptersModule, AdminModule, ApplicationModule, InfrastructureModule, ServerModule,
};

// Re-export Shaku for convenience
pub use shaku::module;

// Re-export domain services
pub use domain_services::{DomainServicesContainer, DomainServicesFactory};

// ============================================================================
// Root Module Definition (Shaku Strict Pattern)
// ============================================================================

use shaku::Interface;
use std::sync::Arc;

// Import provider traits from mcb-domain
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};

// ============================================================================
// Root Module Definition
// ============================================================================

/// Root dependency injection module following Shaku hierarchical pattern.
///
/// This module composes all domain modules into a single container.
/// It uses `use dyn ModuleTrait` to import services from submodules,
/// following Shaku's strict submodule composition pattern.
///
/// ## Note
///
/// Most services are created via runtime factories (DomainServicesFactory)
/// rather than through Shaku resolution, because they need runtime config.
/// The Shaku modules primarily hold null/default providers for testing.
///
/// ## Construction
///
/// ```rust,ignore
/// let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
/// let server = Arc::new(ServerModuleImpl::builder().build());
/// let adapters = Arc::new(AdaptersModuleImpl::builder().build());
/// let application = Arc::new(ApplicationModuleImpl::builder().build());
/// let admin = Arc::new(AdminModuleImpl::builder().build());
///
/// let root = McpModule::builder(infrastructure, server, adapters, application, admin).build();
/// ```
module! {
    pub McpModule {
        components = [],
        providers = [],

        // Infrastructure services (COMPLETE - all with Component derive)
        use dyn InfrastructureModule {
            components = [
                dyn mcb_domain::ports::providers::cache::CacheProvider,
                dyn mcb_domain::ports::infrastructure::AuthServiceInterface,
                dyn mcb_domain::ports::infrastructure::EventBusProvider,
                dyn mcb_domain::ports::infrastructure::SystemMetricsCollectorInterface,
                dyn mcb_domain::ports::infrastructure::StateStoreProvider,
                dyn mcb_domain::ports::infrastructure::LockProvider
            ],
            providers = []
        },

        // Server components (COMPLETE - all with Component derive)
        use dyn ServerModule {
            components = [
                dyn mcb_domain::ports::admin::PerformanceMetricsInterface,
                dyn mcb_domain::ports::admin::IndexingOperationsInterface
            ],
            providers = []
        },

        // External adapters (providers, repositories, no dependencies)
        use dyn AdaptersModule {
            components = [
                dyn EmbeddingProvider,
                dyn VectorStoreProvider
            ],
            providers = []
        }

        // NOTE: ApplicationModule and AdminModule are NOT included here.
        // They are placeholder modules for the hierarchy - services are created
        // at runtime via DomainServicesFactory, not through Shaku resolution.
    }
}
