//! Dependency Injection System
//!
//! This module provides the complete dependency injection infrastructure
//! following SOLID principles and the provider pattern architecture.
//!
//! ## Hierarchical Module Organization
//!
//! The DI system uses Shaku's submodule pattern for domain-based organization:
//!
//! - `AdaptersModule` - External adapters (HTTP clients, providers)
//! - `InfrastructureModule` - Core infrastructure (metrics, service provider)
//! - `ServerModule` - Server components (performance, indexing)
//! - `AdminModule` - Admin service (depends on infrastructure and server)
//! - `McpModule` - Root module composing all domain modules

pub mod dispatch;
pub mod factory;
pub mod modules;
pub mod registry;

// Re-export main types for convenience
pub use dispatch::{
    create_embedding_provider_from_config, create_vector_store_provider_from_config,
    dispatch_embedding_provider, dispatch_vector_store_provider,
};
pub use factory::{DefaultProviderFactory, ProviderFactory, ServiceProvider};
pub use modules::{
    // Module traits for abstraction
    AdaptersModule,
    // Concrete module implementations
    AdaptersModuleImpl,
    AdminModule,
    AdminModuleImpl,
    InfrastructureModule,
    InfrastructureModuleImpl,
    McpModule,
    ServerModule,
    ServerModuleImpl,
};
pub use registry::ProviderRegistry;
