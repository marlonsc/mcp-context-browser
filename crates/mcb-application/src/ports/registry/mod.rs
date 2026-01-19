//! Provider Registry System
//!
//! Defines the auto-registration infrastructure for plugin providers.
//! Uses the `linkme` crate for compile-time registration of providers
//! that can be discovered and instantiated at runtime.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Provider Registration Flow                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  1. Provider defines:  #[linkme::distributed_slice(PROVIDERS)]  │
//! │                        static ENTRY: ProviderEntry = ...        │
//! │                              ↓                                  │
//! │  2. Registry declares: #[linkme::distributed_slice]             │
//! │                        pub static PROVIDERS: [Entry] = [..]     │
//! │                              ↓                                  │
//! │  3. Resolver queries:  PROVIDERS.iter()                         │
//! │                              ↓                                  │
//! │  4. Config selects:    "provider = ollama" → OllamaProvider     │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Registering a Provider (in mcb-providers)
//!
//! ```ignore
//! use mcb_application::ports::registry::{EmbeddingProviderEntry, EMBEDDING_PROVIDERS};
//!
//! #[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
//! static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
//!     name: "ollama",
//!     description: "Ollama local embedding provider",
//!     factory: |config| Ok(Arc::new(OllamaProvider::from_config(config)?)),
//! };
//! ```
//!
//! ### Resolving a Provider (in mcb-infrastructure)
//!
//! ```ignore
//! use mcb_application::ports::registry::resolve_embedding_provider;
//!
//! let config = EmbeddingProviderConfig { provider: "ollama".into(), .. };
//! let provider = resolve_embedding_provider(&config)?;
//! ```

pub mod cache;
pub mod embedding;
pub mod language;
pub mod vector_store;

// Re-export all registry types and functions
pub use cache::{
    CACHE_PROVIDERS, CacheProviderConfig, CacheProviderEntry, list_cache_providers,
    resolve_cache_provider,
};
pub use embedding::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry, list_embedding_providers,
    resolve_embedding_provider,
};
pub use language::{
    LANGUAGE_PROVIDERS, LanguageProviderConfig, LanguageProviderEntry, list_language_providers,
    resolve_language_provider,
};
pub use vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
    list_vector_store_providers, resolve_vector_store_provider,
};
