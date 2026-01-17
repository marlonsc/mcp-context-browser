//! DI Module Organization - Clean Architecture Modules (Shaku Strict Pattern)
//!
//! This module provides Shaku modules for internal infrastructure services only.
//! External providers (embedding, vector_store, cache, language) are resolved
//! dynamically via the registry system in `di/resolver.rs`.
//!
//! ## Module Hierarchy
//!
//! ```text
//! AppContext (composition root)
//! ├── ResolvedProviders (from registry - embedding, vector_store, cache, language)
//! │
//! └── Infrastructure Modules (internal services via Shaku)
//!     ├── InfrastructureModule (auth, metrics, sync, snapshot, shutdown)
//!     ├── ServerModule (performance metrics, indexing operations)
//!     └── AdminModule (marker module for future admin services)
//! ```
//!
//! ## Design Notes
//!
//! - External providers are resolved via `di::resolver::resolve_providers()`
//! - Internal infrastructure services use Shaku DI
//! - No imports from mcb_providers in this module

/// Domain module traits (interfaces for Shaku HasComponent)
pub mod traits;

/// Admin services (marker module for future admin-specific services)
pub mod admin;

/// Core infrastructure services (auth, metrics, sync, snapshot, shutdown)
pub mod infrastructure;

/// MCP server components (performance metrics, indexing operations)
pub mod server;

/// Domain services factory (runtime service creation with DI)
pub mod domain_services;

// Re-export module implementations
pub use admin::AdminModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use server::ServerModuleImpl;

// Re-export module traits
pub use traits::{AdminModule, InfrastructureModule, ServerModule};

// Re-export Shaku for convenience
pub use shaku::{module, HasComponent};

// Re-export domain services
pub use domain_services::{DomainServicesContainer, DomainServicesFactory};
