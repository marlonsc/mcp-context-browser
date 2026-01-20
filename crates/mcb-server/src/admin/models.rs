//! Admin data models
//!
//! Request and response models for the admin API.

use serde::{Deserialize, Serialize};

/// Server information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server version
    pub version: String,
    /// Build timestamp
    pub build_time: Option<String>,
    /// Git commit hash
    pub git_hash: Option<String>,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: option_env!("BUILD_TIME").map(String::from),
            git_hash: option_env!("GIT_HASH").map(String::from),
        }
    }
}

/// Collection statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Collection name
    pub name: String,
    /// Number of vectors stored
    pub vector_count: u64,
    /// Total storage size in bytes
    pub storage_bytes: u64,
    /// Last modified timestamp
    pub last_modified: Option<u64>,
}

/// Admin action response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminActionResponse {
    /// Whether the action succeeded
    pub success: bool,
    /// Action result message
    pub message: String,
}

impl AdminActionResponse {
    /// Create a success response
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    /// Create a failure response
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

// ============================================================================
// Browse API Models
// ============================================================================

/// Response for listing all collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionListResponse {
    /// List of collections with metadata
    pub collections: Vec<CollectionInfoResponse>,
    /// Total number of collections
    pub total: usize,
}

/// Collection information for browse API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfoResponse {
    /// Collection name
    pub name: String,
    /// Number of vectors in the collection
    pub vector_count: u64,
    /// Number of unique files indexed
    pub file_count: u64,
    /// Unix timestamp of last indexing (if available)
    pub last_indexed: Option<u64>,
    /// Provider name (e.g., "milvus", "in_memory")
    pub provider: String,
}

/// Response for listing files in a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResponse {
    /// List of indexed files
    pub files: Vec<FileInfoResponse>,
    /// Total number of files
    pub total: usize,
    /// Collection name
    pub collection: String,
}

/// File information for browse API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoResponse {
    /// File path
    pub path: String,
    /// Number of code chunks from this file
    pub chunk_count: u32,
    /// Programming language
    pub language: String,
    /// File size in bytes (if available)
    pub size_bytes: Option<u64>,
}

/// Response for listing chunks in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkListResponse {
    /// List of code chunks
    pub chunks: Vec<ChunkDetailResponse>,
    /// File path
    pub file_path: String,
    /// Collection name
    pub collection: String,
    /// Total number of chunks
    pub total: usize,
}

/// Detailed chunk information for browse API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDetailResponse {
    /// Chunk ID
    pub id: String,
    /// Code content
    pub content: String,
    /// File path
    pub file_path: String,
    /// Starting line number
    pub start_line: u32,
    /// Ending line number (estimated from content)
    pub end_line: u32,
    /// Programming language
    pub language: String,
    /// Relevance score (1.0 for direct retrieval)
    pub score: f64,
}
