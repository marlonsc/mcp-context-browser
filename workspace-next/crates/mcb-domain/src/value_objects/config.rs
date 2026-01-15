//! Configuration Value Objects
//!
//! Value objects representing configuration for external providers
//! and system settings. These configurations define how the system
//! interacts with external services.

use crate::value_objects::types::{EmbeddingProviderKind, VectorStoreProviderKind};
use serde::{Deserialize, Serialize};

/// Value Object: Embedding Provider Configuration
///
/// Configuration for connecting to and using embedding providers.
/// Defines which provider to use and how to authenticate with it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingConfig {
    /// Provider name (openai, ollama, fastembed, etc.)
    pub provider: EmbeddingProviderKind,
    /// Model identifier specific to the provider
    pub model: String,
    /// API key for cloud providers
    pub api_key: Option<String>,
    /// Custom API endpoint URL
    pub base_url: Option<String>,
    /// Output embedding dimensions
    pub dimensions: Option<usize>,
    /// Maximum input token limit
    pub max_tokens: Option<usize>,
}

/// Value Object: Vector Store Configuration
///
/// Configuration for connecting to vector storage backends.
/// Defines which storage system to use and connection parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorStoreConfig {
    /// Provider name (filesystem, in-memory, milvus, edgevec)
    pub provider: VectorStoreProviderKind,
    /// Server address for remote providers (e.g., Milvus)
    pub address: Option<String>,
    /// Authentication token for remote providers
    pub token: Option<String>,
    /// Collection name for organizing vectors
    pub collection: Option<String>,
    /// Expected embedding dimensions
    pub dimensions: Option<usize>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
}

/// Value Object: Synchronization Batch
///
/// Represents a batch of files queued for synchronization/re-indexing.
/// Used by the file watcher daemon to batch file changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncBatch {
    /// Unique batch identifier
    pub id: String,
    /// Repository or collection identifier
    pub collection: String,
    /// List of file paths to process
    pub files: Vec<String>,
    /// Priority level (higher numbers = higher priority)
    pub priority: u8,
    /// Timestamp when batch was created
    pub created_at: i64,
}