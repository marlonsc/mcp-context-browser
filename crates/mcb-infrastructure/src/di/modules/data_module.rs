//! Data Module - Provides data persistence services
//!
//! This module provides vector store and data persistence implementations.
//! Uses null/memory providers as default for testing.

use shaku::module;

// Import data providers
use mcb_providers::vector_store::NullVectorStoreProvider;

// Import traits
use crate::di::modules::traits::DataModule;

module! {
    pub DataModuleImpl: DataModule {
        components = [
            // Default null vector store provider (testing fallback)
            NullVectorStoreProvider
        ],
        providers = []
    }
}
