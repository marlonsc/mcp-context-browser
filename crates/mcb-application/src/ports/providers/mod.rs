//! External Provider Ports
//!
//! Ports for external services and providers that the domain depends on.
//! These interfaces define contracts for embedding providers, vector stores,
//! language chunking, caching, cryptography, and other external services.
//!
//! ## Note
//!
//! Provider port traits are now defined in `mcb-domain` following Clean Architecture
//! principles. This module re-exports them for backward compatibility.
//!
//! ## Provider Ports
//!
//! | Port | Description |
//! |------|-------------|
//! | [`EmbeddingProvider`] | Text embedding generation services |
//! | [`VectorStoreProvider`] | Vector storage and similarity search |
//! | [`HybridSearchProvider`] | Combined semantic and keyword search |
//! | [`LanguageChunkingProvider`] | Language-specific code chunking |
//! | [`CacheProvider`] | Caching backend services |
//! | [`CryptoProvider`] | Encryption/decryption services |

// Re-export submodules from domain for backward compatibility with paths like `cache::CacheProvider`
pub use mcb_domain::ports::providers::cache;
pub use mcb_domain::ports::providers::config;
pub use mcb_domain::ports::providers::crypto;
pub use mcb_domain::ports::providers::embedding;
pub use mcb_domain::ports::providers::hybrid_search;
pub use mcb_domain::ports::providers::language_chunking;
pub use mcb_domain::ports::providers::vector_store;

// Re-export commonly used traits directly for convenience
pub use mcb_domain::ports::providers::{
    // Cache
    CacheEntryConfig,
    CacheProvider,
    CacheProviderFactoryInterface,
    CacheStats,
    // Crypto
    CryptoProvider,
    // Embedding
    EmbeddingProvider,
    EncryptedData,
    // Hybrid Search
    HybridSearchProvider,
    HybridSearchResult,
    // Language Chunking
    LanguageChunkingProvider,
    // Config
    ProviderConfigManagerInterface,
    // Vector Store
    VectorStoreAdmin,
    VectorStoreProvider,
};
