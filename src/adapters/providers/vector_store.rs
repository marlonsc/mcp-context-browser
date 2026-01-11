//! Vector store provider implementations

pub mod edgevec;
pub mod encrypted;
pub mod filesystem;
pub mod in_memory;
#[cfg(feature = "milvus")]
pub mod milvus;
pub mod null;

// Re-export for convenience
pub use edgevec::EdgeVecVectorStoreProvider;
pub use encrypted::EncryptedVectorStoreProvider;
pub use filesystem::FilesystemVectorStore;
pub use in_memory::InMemoryVectorStoreProvider;
#[cfg(feature = "milvus")]
pub use milvus::MilvusVectorStoreProvider;
pub use null::NullVectorStoreProvider;
