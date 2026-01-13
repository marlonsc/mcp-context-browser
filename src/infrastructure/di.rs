//! Dependency Injection System
//!
//! This module provides the complete dependency injection infrastructure
//! following SOLID principles and the provider pattern architecture.

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
pub use modules::McpModule;
pub use registry::ProviderRegistry;
