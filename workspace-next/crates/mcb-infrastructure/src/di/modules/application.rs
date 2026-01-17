//! Application Module Implementation - Use Cases and Business Logic
//!
//! This module provides application services (use cases) that orchestrate
//! business logic and coordinate between domain entities and external ports.
//!
//! ## Services Provided
//!
//! - ContextService: Manages embeddings and vector storage for semantic understanding
//! - SearchService: Provides semantic search capabilities
//! - IndexingService: Handles code indexing and ingestion operations
//!
//! ## Dependencies
//!
//! Application services depend on:
//! - Cache providers (from AdaptersModule)
//! - Embedding providers (from AdaptersModule)
//! - Vector store providers (from AdaptersModule)
//! - Language chunking providers (from AdaptersModule)

use shaku::module;

// Import application use cases
use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};

// Import traits
use super::traits::ApplicationModule;

/// Application module implementation - Use Cases and Business Logic
///
/// This module provides application services that orchestrate business logic.
/// Services depend on providers from the adapters module.
///
/// ## Dependencies
///
/// - Cache providers (from AdaptersModule)
/// - Embedding providers (from AdaptersModule)
/// - Vector store providers (from AdaptersModule)
/// - Language chunking providers (from AdaptersModule)
///
/// ## Construction
///
/// ```rust,ignore
/// let application = ApplicationModuleImpl::builder().build();
/// ```
module! {
    pub ApplicationModuleImpl: ApplicationModule {
        components = [
            // Application use cases - business logic orchestration
            ContextServiceImpl,
            SearchServiceImpl,
            IndexingServiceImpl
        ],
        providers = []
    }
}
