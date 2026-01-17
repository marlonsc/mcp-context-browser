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
