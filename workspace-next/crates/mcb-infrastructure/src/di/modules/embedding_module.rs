//! Embedding Module - Provides embedding services
//!
//! This module provides embedding provider implementations.
//! Uses null provider as default for testing.

use shaku::module;

// Import embedding providers
use mcb_providers::embedding::NullEmbeddingProvider;

// Import traits
use crate::di::modules::traits::EmbeddingModule;

/// Embedding module providing embedding provider implementations
///
/// ## Services Provided
/// - EmbeddingProvider: For text embedding operations
///
/// ## Default Implementation
/// - NullEmbeddingProvider: Deterministic hash-based embeddings for testing
///
/// ## Production Override
/// Can be overridden with OpenAI, Ollama, Gemini, or other embedding providers
module! {
    pub EmbeddingModuleImpl: EmbeddingModule {
        components = [
            // Default null embedding provider (testing fallback)
            NullEmbeddingProvider
        ],
        providers = []
    }
}