//! Dependency Injection System
//!
//! This module provides the complete dependency injection infrastructure
//! following SOLID principles and the provider pattern architecture.

pub mod factory;
pub mod registry;

// Re-export main types for convenience
pub use factory::{DefaultProviderFactory, ProviderFactory, ServiceProvider};
pub use registry::ProviderRegistry;
