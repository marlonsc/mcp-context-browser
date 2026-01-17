//! Provider Factories
//!
//! Factory pattern for creating providers via DI wiring.
//! All provider implementations come from mcb-providers crate.
//!
//! **ARCHITECTURE**: This module contains ONLY wiring logic.
//! No concrete implementations - those are in mcb-providers.

pub mod implementation;
pub mod providers;
pub mod traits;

pub use providers::{
    embedding_providers, vector_store_providers, EmbeddingProviderFactory,
    VectorStoreProviderFactory,
};

pub use implementation::DefaultCryptoServiceFactory;
pub use traits::CryptoServiceFactory;
