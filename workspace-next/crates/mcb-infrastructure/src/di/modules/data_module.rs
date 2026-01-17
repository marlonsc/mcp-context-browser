//! Data Module - Provides data persistence services
//!
//! This module provides vector store and data persistence implementations.
//! Uses null/memory providers as default for testing.

use shaku::module;

// Import data providers
use mcb_providers::vector_store::NullVectorStoreProvider;

// Import traits
use crate::di::modules::traits::DataModule;

/// Data module providing data persistence implementations
///
/// ## Services Provided
/// - VectorStoreProvider: For vector storage operations
///
/// ## Default Implementation
/// - NullVectorStoreProvider: In-memory vector store for testing
///
/// ## Production Override
/// Can be overridden with Milvus, EdgeVec, or other vector stores
module! {
    pub DataModuleImpl: DataModule {
        components = [
            // Default null vector store provider (testing fallback)
            NullVectorStoreProvider
        ],
        providers = []
    }
}