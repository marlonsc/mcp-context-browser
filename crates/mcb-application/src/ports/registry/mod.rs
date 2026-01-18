//! Provider Registry System
//!
//! Defines the auto-registration infrastructure for plugin providers.
//! Uses the `inventory` crate for compile-time registration of providers
//! that can be discovered and instantiated at runtime.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Provider Registration Flow                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  1. Provider defines:  inventory::submit! { Entry { ... } }     │
//! │                              ↓                                  │
//! │  2. Registry collects: inventory::collect!(Entry)               │
//! │                              ↓                                  │
//! │  3. Resolver queries:  inventory::iter::<Entry>                 │
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
//! use mcb_application::ports::registry::EmbeddingProviderEntry;
//!
//! inventory::submit! {
//!     EmbeddingProviderEntry {
//!         name: "ollama",
//!         description: "Ollama local embedding provider",
//!         factory: |config| Ok(Arc::new(OllamaProvider::from_config(config)?)),
//!     }
//! }
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
    list_cache_providers, resolve_cache_provider, CacheProviderConfig, CacheProviderEntry,
    CACHE_PROVIDERS_LINKME,
};
pub use embedding::{
    list_embedding_providers, resolve_embedding_provider, EmbeddingProviderConfig,
    EmbeddingProviderEntry, EMBEDDING_PROVIDERS_LINKME,
};
pub use language::{
    list_language_providers, resolve_language_provider, LanguageProviderConfig,
    LanguageProviderEntry, LANGUAGE_PROVIDERS_LINKME,
};
pub use vector_store::{
    list_vector_store_providers, resolve_vector_store_provider, VectorStoreProviderConfig,
    VectorStoreProviderEntry, VECTOR_STORE_PROVIDERS_LINKME,
};
