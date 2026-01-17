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
    resolve_cache_provider, list_cache_providers,
    CacheProviderConfig, CacheProviderEntry,
};
pub use embedding::{
    resolve_embedding_provider, list_embedding_providers,
    EmbeddingProviderConfig, EmbeddingProviderEntry,
};
pub use language::{
    resolve_language_provider, list_language_providers,
    LanguageProviderConfig, LanguageProviderEntry,
};
pub use vector_store::{
    resolve_vector_store_provider, list_vector_store_providers,
    VectorStoreProviderConfig, VectorStoreProviderEntry,
};
