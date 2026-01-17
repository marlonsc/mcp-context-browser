//! # MCP Context Browser - Provider Implementations
//!
//! This crate contains all user-selectable provider implementations following
//! Clean Architecture principles. Each provider implements a port (trait)
//! defined in `mcb-domain`.
//!
//! ## Provider Categories
//!
//! | Category | Port | Implementations |
//! |----------|------|-----------------|
//! | Embedding | `EmbeddingProvider` | OpenAI, Ollama, VoyageAI, Gemini, FastEmbed, Null |
//! | Vector Store | `VectorStoreProvider` | InMemory, Encrypted, Null |
//! | Cache | `CacheProvider` | Moka, Redis, Null |
//! | Events | `EventPublisher` | Tokio, Nats, Null |
//! | Language | `LanguageChunkingProvider` | Rust, Python, Go, Java, etc. |
//!
//! ## Feature Flags
//!
//! Each provider can be enabled/disabled via feature flags for minimal builds:
//!
//! ```toml
//! [dependencies]
//! mcb-providers = { version = "0.1", default-features = false, features = ["embedding-ollama", "cache-moka"] }
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use mcb_providers::embedding::OllamaEmbeddingProvider;
//! use mcb_providers::cache::MokaCacheProvider;
//! use mcb_providers::language::RustProcessor;
//! ```

// Re-export mcb-domain types commonly used with providers
pub use mcb_domain::error::{Error, Result};
pub use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};

// Re-export CryptoProvider from domain (for encrypted vector store)
#[cfg(feature = "vectorstore-encrypted")]
pub use mcb_domain::ports::providers::{CryptoProvider, EncryptedData};

/// Provider-specific constants
pub mod constants;

/// Shared utilities for provider implementations
pub mod utils;

/// Embedding provider implementations
///
/// Implements `EmbeddingProvider` trait for various embedding APIs.
pub mod embedding;

/// Vector store provider implementations
///
/// Implements `VectorStoreProvider` trait for vector storage backends.
pub mod vector_store;

/// Cache provider implementations
///
/// Implements `CacheProvider` trait for caching backends.
pub mod cache;

/// Event publisher implementations
///
/// Implements `EventPublisher` trait for event bus backends.
pub mod events;

/// HTTP client abstractions
///
/// Provides `HttpClientProvider` trait and configuration for API-based providers.
pub mod http;

/// Code chunking provider implementations
///
/// Implements `CodeChunker` trait for intelligent code chunking.
/// Provides `IntelligentChunker` using tree-sitter and language-specific processors.
pub mod chunking;

/// Language chunking provider implementations
///
/// Implements `LanguageChunkingProvider` trait for AST-based code parsing.
/// Also provides `IntelligentChunker` that implements `CodeChunker` trait.
pub mod language;

/// Admin provider implementations
///
/// Implements `PerformanceMetricsInterface` and `IndexingOperationsInterface` ports.
pub mod admin;
