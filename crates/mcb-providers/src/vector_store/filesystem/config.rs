//! Filesystem vector store configuration
//!
//! Configuration types for the filesystem-based vector store.

use crate::constants::{
    FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE, FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Filesystem vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemVectorStoreConfig {
    /// Base directory for storing vector data
    pub base_path: PathBuf,
    /// Maximum vectors per shard file
    pub max_vectors_per_shard: usize,
    /// Vector dimensions (must match embedding dimensions)
    pub dimensions: usize,
    /// Enable compression for stored vectors
    pub compression_enabled: bool,
    /// Index cache size (number of index entries to keep in memory)
    pub index_cache_size: usize,
    /// Enable memory mapping for better performance
    pub memory_mapping_enabled: bool,
}

/// Returns default FilesystemVectorStoreConfig with sensible defaults for local vector storage
impl Default for FilesystemVectorStoreConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./data/vectors"),
            max_vectors_per_shard: FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD,
            dimensions: 1536, // Default for text-embedding-3-small
            compression_enabled: false,
            index_cache_size: FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE,
            memory_mapping_enabled: true,
        }
    }
}
