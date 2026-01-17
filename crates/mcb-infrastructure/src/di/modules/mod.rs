//! DI Module Organization - Clean Architecture Modules (Shaku Strict Pattern)
//!
//! This module implements a strict Shaku-based hierarchical module system
//! following Clean Architecture and Domain-Driven Design principles.
//!
//! ## Module Hierarchy
//!
//! ```text
//! AppContainer (composition root)
//! ├── Context Modules (provider implementations)
//! │   ├── CacheModule (NullCacheProvider)
//! │   ├── EmbeddingModule (NullEmbeddingProvider)
//! │   ├── DataModule (NullVectorStoreProvider)
//! │   └── LanguageModule (UniversalLanguageChunkingProvider)
//! │
//! └── Infrastructure Modules (cross-cutting services)
//!     ├── InfrastructureModule (auth, events, metrics, sync, snapshot)
//!     ├── ServerModule (performance metrics, indexing operations)
//!     └── AdminModule (marker module for future admin services)
//! ```
//!
//! ## Design Notes
//!
//! - Services requiring runtime configuration are created via `DomainServicesFactory`
//! - Shaku modules provide null providers as defaults, overridable at runtime
//! - Context modules can be overridden with production providers via builder pattern

/// Domain module traits (interfaces for Shaku HasComponent)
pub mod traits;

/// Context modules (Clean Architecture - provider implementations)
pub mod cache_module;
pub mod data_module;
pub mod embedding_module;
pub mod language_module;

/// Admin services (marker module for future admin-specific services)
pub mod admin;
/// Infrastructure modules
/// Core infrastructure services (auth, events, metrics, sync, snapshot)
pub mod infrastructure;
/// MCP server components (performance metrics, indexing operations)
pub mod server;

/// Domain services factory (runtime service creation with DI)
pub mod domain_services;

// Re-export module implementations
pub use admin::AdminModuleImpl;
pub use cache_module::CacheModuleImpl;
pub use data_module::DataModuleImpl;
pub use embedding_module::EmbeddingModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use language_module::LanguageModuleImpl;
pub use server::ServerModuleImpl;

// Re-export module traits
pub use traits::{
    AdminModule, CacheModule, DataModule, EmbeddingModule, InfrastructureModule, LanguageModule,
    ServerModule,
};

// Re-export Shaku for convenience
pub use shaku::{module, HasComponent};

// Re-export domain services
pub use domain_services::{DomainServicesContainer, DomainServicesFactory};
