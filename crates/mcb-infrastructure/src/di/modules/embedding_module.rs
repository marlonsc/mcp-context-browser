//! Embedding Module - Provides embedding services
//!
//! This module provides embedding provider implementations.
//! Uses null provider as default for testing.

use shaku::module;

// Import embedding providers
use mcb_providers::embedding::NullEmbeddingProvider;

// Import traits
use crate::di::modules::traits::EmbeddingModule;

module! {
    pub EmbeddingModuleImpl: EmbeddingModule {
        components = [
            // Default null embedding provider (testing fallback)
            NullEmbeddingProvider
        ],
        providers = []
    }
}
