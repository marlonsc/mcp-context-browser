//! Browse-related value objects for code navigation
//!
//! These value objects support the Admin UI code browser functionality,
//! providing structured representations of indexed collections and files.

use serde::{Deserialize, Serialize};

/// Information about an indexed collection
///
/// Represents metadata about a vector store collection, including
/// statistics useful for browsing and monitoring.
///
/// # Example
///
/// ```
/// use mcb_domain::value_objects::CollectionInfo;
///
/// let info = CollectionInfo {
///     name: "my-project".to_string(),
///     vector_count: 1500,
///     file_count: 42,
///     last_indexed: Some(1705680000),
///     provider: "milvus".to_string(),
/// };
///
/// assert_eq!(info.name, "my-project");
/// assert_eq!(info.vector_count, 1500);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Name of the collection
    pub name: String,

    /// Total number of vectors in the collection
    pub vector_count: u64,

    /// Number of unique files indexed in the collection
    pub file_count: u64,

    /// Unix timestamp of last indexing operation (if available)
    pub last_indexed: Option<u64>,

    /// Name of the vector store provider (e.g., "milvus", "in_memory")
    pub provider: String,
}

impl CollectionInfo {
    /// Create a new CollectionInfo instance
    pub fn new(
        name: impl Into<String>,
        vector_count: u64,
        file_count: u64,
        last_indexed: Option<u64>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            vector_count,
            file_count,
            last_indexed,
            provider: provider.into(),
        }
    }
}

/// Summary information about an indexed file
///
/// Provides metadata about a single file within a collection,
/// useful for file listing and navigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInfo {
    /// Relative path of the file within the indexed codebase
    pub path: String,

    /// Number of code chunks extracted from this file
    pub chunk_count: u32,

    /// Detected programming language
    pub language: String,

    /// File size in bytes (if available)
    pub size_bytes: Option<u64>,
}

impl FileInfo {
    /// Create a new FileInfo instance
    pub fn new(
        path: impl Into<String>,
        chunk_count: u32,
        language: impl Into<String>,
        size_bytes: Option<u64>,
    ) -> Self {
        Self {
            path: path.into(),
            chunk_count,
            language: language.into(),
            size_bytes,
        }
    }
}

// Tests moved to tests/unit/browse_tests.rs per test organization standards
