//! Application Module Implementation - Placeholder for Shaku Module Hierarchy
//!
//! This module is a **placeholder** for the Shaku module hierarchy.
//!
//! ## Why Services Are NOT Registered Here
//!
//! Application services (ContextService, SearchService, IndexingService) have complex
//! dependencies that cannot be resolved through Shaku's static module system:
//!
//! - They depend on runtime-configured providers (embedding, vector store)
//! - They depend on infrastructure components that require async initialization
//! - They need dependencies from multiple modules that can't be wired at compile-time
//!
//! ## Correct Pattern
//!
//! Services are created at runtime via `DomainServicesFactory::create_services()` in
//! `domain_services.rs`. This allows:
//!
//! - Async initialization of dependencies
//! - Runtime configuration of providers
//! - Proper dependency injection without Shaku limitations
//!
//! ## Usage
//!
//! ```rust,ignore
//! // 1. Build Shaku modules (placeholder only)
//! let application = ApplicationModuleImpl::builder().build();
//!
//! // 2. Create services via factory (with runtime dependencies)
//! let services = DomainServicesFactory::create_services(
//!     cache, crypto, config, embedding_provider, vector_store_provider
//! ).await?;
//! ```

use shaku::module;

// Import traits
use super::traits::ApplicationModule;

/// Application module implementation - **Placeholder only**.
///
/// This module exists to satisfy the Shaku module hierarchy but
/// does NOT register any components. Application services are
/// created via `DomainServicesFactory` at runtime.
///
/// ## Construction
///
/// ```rust,ignore
/// let application = ApplicationModuleImpl::builder().build();
/// ```
module! {
    pub ApplicationModuleImpl: ApplicationModule {
        components = [],
        providers = []
    }
}
