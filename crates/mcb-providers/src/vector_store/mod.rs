//! Vector Store Provider Implementations
//!
//! Provides storage backends for vector embeddings.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | NullVectorStoreProvider | Testing | No-op stub for testing |
//! | InMemoryVectorStoreProvider | Local | In-memory storage (non-persistent) |
//! | EncryptedVectorStoreProvider | Secure | AES-256-GCM encryption wrapper |
//! | FilesystemVectorStore | Local | Persistent filesystem-based storage |
//! | EdgeVecVectorStoreProvider | Embedded | High-performance HNSW vector store |
//! | MilvusVectorStoreProvider | Cloud | Production-scale cloud vector database |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `NullVectorStoreProvider` for unit tests
//! - **Development with data**: Use `InMemoryVectorStoreProvider`
//! - **Production with encryption**: Use `EncryptedVectorStoreProvider` wrapper
//! - **Production local storage**: Use `FilesystemVectorStore` for persistent local storage
//! - **High-performance embedded**: Use `EdgeVecVectorStoreProvider` for sub-ms search
//! - **Cloud production**: Use `MilvusVectorStoreProvider` for distributed cloud deployments

#[cfg(feature = "vectorstore-edgevec")]
pub mod edgevec;
#[cfg(feature = "vectorstore-encrypted")]
pub mod encrypted;
#[cfg(feature = "vectorstore-filesystem")]
pub mod filesystem;
pub mod in_memory;
#[cfg(feature = "vectorstore-milvus")]
pub mod milvus;
pub mod null;

// Re-export for convenience
#[cfg(feature = "vectorstore-edgevec")]
pub use edgevec::{
    EdgeVecConfig, EdgeVecVectorStoreProvider, HnswConfig, MetricType, QuantizerConfig,
};
#[cfg(feature = "vectorstore-encrypted")]
pub use encrypted::EncryptedVectorStoreProvider;
#[cfg(feature = "vectorstore-filesystem")]
pub use filesystem::{FilesystemVectorStore, FilesystemVectorStoreConfig};
pub use in_memory::InMemoryVectorStoreProvider;
#[cfg(feature = "vectorstore-milvus")]
pub use milvus::MilvusVectorStoreProvider;
pub use null::NullVectorStoreProvider;
