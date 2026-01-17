//! Vector Store Provider Implementations
//!
//! Provides storage backends for vector embeddings.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | [`NullVectorStoreProvider`] | Testing | No-op stub for testing |
//! | [`InMemoryVectorStoreProvider`] | Local | In-memory storage (non-persistent) |
//! | [`EncryptedVectorStoreProvider`] | Secure | AES-256-GCM encryption wrapper |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `NullVectorStoreProvider` for unit tests
//! - **Development with data**: Use `InMemoryVectorStoreProvider`
//! - **Production with encryption**: Use `EncryptedVectorStoreProvider` wrapper

#[cfg(feature = "vectorstore-encrypted")]
pub mod encrypted;
pub mod in_memory;
pub mod null;

// Re-export for convenience
#[cfg(feature = "vectorstore-encrypted")]
pub use encrypted::EncryptedVectorStoreProvider;
pub use in_memory::InMemoryVectorStoreProvider;
pub use null::NullVectorStoreProvider;
