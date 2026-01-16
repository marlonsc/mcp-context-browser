//! External Provider Ports
//!
//! Ports for external services and providers that the domain depends on.
//! These interfaces define contracts for embedding providers, vector stores,
//! language chunking, caching, cryptography, and other external services.
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

/// Cache provider port
pub mod cache;
/// Config provider port
pub mod config;
/// Crypto provider port
pub mod crypto;
/// Embedding provider port
pub mod embedding;
/// Hybrid search provider port
pub mod hybrid_search;
/// Language chunking provider port
pub mod language_chunking;
/// Vector store provider port
pub mod vector_store;

// Re-export provider ports
pub use cache::CacheProvider;
pub use config::ProviderConfigManagerInterface;
pub use crypto::{CryptoProvider, EncryptedData};
pub use embedding::EmbeddingProvider;
pub use hybrid_search::HybridSearchProvider;
pub use language_chunking::LanguageChunkingProvider;
pub use vector_store::{VectorStoreAdmin, VectorStoreProvider};
