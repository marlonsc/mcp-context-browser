//! Use Case Module - Provides application use cases
//!
//! This module provides application use cases implementations.
//! Depends on context modules for external services.

use shaku::module;

// Import use cases
use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};

// Import traits
use crate::di::modules::traits::UseCaseModule;

/// Use case module providing application business logic
///
/// ## Services Provided
/// - ContextServiceInterface: Code intelligence operations
/// - SearchServiceInterface: Semantic search operations
/// - IndexingServiceInterface: Code indexing operations
///
/// ## Dependencies
/// - CacheModule: For caching operations
/// - EmbeddingModule: For text embeddings
/// - DataModule: For vector storage
/// - LanguageModule: For code chunking
module! {
    pub UseCaseModuleImpl: UseCaseModule {
        components = [
            // Application use cases
            ContextServiceImpl,
            SearchServiceImpl,
            IndexingServiceImpl
        ],
        providers = []
    }
}