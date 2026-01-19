//! Dependency Injection System - dill Catalog + linkme Registry
//!
//! This module implements dependency injection using:
//! - **dill Catalog**: Runtime DI for infrastructure services
//! - **linkme registry**: Compile-time discovery of external providers
//!
//! ## Architecture
//!
//! ```text
//! linkme (compile-time)          dill (runtime)
//! ─────────────────────          ──────────────────────
//! EMBEDDING_PROVIDERS     →      EmbeddingProviderResolver
//! (list of factories)                     ↓
//!                               EmbeddingProviderHandle (RwLock)
//!                                         ↓
//!                               EmbeddingAdminService
//!                               (switch_provider via API)
//! ```
//!
//! ## Key Principles
//!
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//! - **Composition Root**: Services composed in bootstrap.rs init_app()
//! - **Runtime Switching**: Providers can be changed via admin API
//! - **Testability**: Null providers enable isolated testing

pub mod admin;
pub mod bootstrap;
pub mod dispatch;
pub mod handles;
pub mod modules;
pub mod provider_resolvers;
pub mod resolver;

pub use admin::{
    CacheAdminService, EmbeddingAdminService, LanguageAdminService, ProviderInfo,
    VectorStoreAdminService,
};
pub use bootstrap::*;
pub use dispatch::*;
pub use handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
pub use modules::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
pub use provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
pub use resolver::{ResolvedProviders, resolve_providers};
