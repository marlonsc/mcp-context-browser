//! Configuration key constants for type-safe configuration management
//!
//! Centralizes all configuration key strings used in the admin interface.
//! This eliminates magic strings and provides IDE autocompletion support.
//!
//! # Example
//!
//! ```rust
//! use mcp_context_browser::server::admin::config_keys::{indexing, security, cache};
//!
//! // Access typed configuration keys
//! assert_eq!(indexing::CHUNK_SIZE, "indexing.chunk_size");
//! assert_eq!(security::ENABLE_AUTH, "security.enable_auth");
//! assert_eq!(cache::ENABLED, "cache.enabled");
//! ```

/// Indexing configuration keys
pub mod indexing {
    /// Maximum size of each indexed chunk
    pub const CHUNK_SIZE: &str = "indexing.chunk_size";
    /// Overlap between consecutive chunks
    pub const CHUNK_OVERLAP: &str = "indexing.chunk_overlap";
    /// Maximum file size to index
    pub const MAX_FILE_SIZE: &str = "indexing.max_file_size";
    /// File extensions that should be indexed
    pub const SUPPORTED_EXTENSIONS: &str = "indexing.supported_extensions";
    /// Patterns for files/directories to exclude from indexing
    pub const EXCLUDE_PATTERNS: &str = "indexing.exclude_patterns";
}

/// Security configuration keys
pub mod security {
    /// Enable/disable authentication for admin interface
    pub const ENABLE_AUTH: &str = "security.enable_auth";
    /// Enable/disable rate limiting
    pub const RATE_LIMITING: &str = "security.rate_limiting";
    /// Maximum requests per minute allowed
    pub const MAX_REQUESTS_PER_MINUTE: &str = "security.max_requests_per_minute";
}

/// Metrics configuration keys
pub mod metrics {
    /// Enable/disable metrics collection
    pub const ENABLED: &str = "metrics.enabled";
    /// Interval between metrics collection in seconds
    pub const COLLECTION_INTERVAL: &str = "metrics.collection_interval";
    /// Number of days to retain metrics data
    pub const RETENTION_DAYS: &str = "metrics.retention_days";
}

/// Cache configuration keys
pub mod cache {
    /// Enable/disable caching
    pub const ENABLED: &str = "cache.enabled";
    /// Cache backend type (moka, redis, etc.)
    pub const BACKEND_TYPE: &str = "cache.backend_type";
    /// Maximum number of entries in cache
    pub const MAX_ENTRIES: &str = "cache.max_entries";
    /// Time-to-live for cache entries in seconds
    pub const TTL_SECONDS: &str = "cache.ttl_seconds";
}

/// Embedding provider configuration keys
pub mod embedding {
    /// Embedding model to use
    pub const MODEL: &str = "embedding.model";
    /// API key for embedding provider
    pub const API_KEY: &str = "embedding.api_key";
    /// Base URL for embedding provider API
    pub const BASE_URL: &str = "embedding.base_url";
}

/// Vector store configuration keys
pub mod vector_store {
    /// Type of vector store to use
    pub const TYPE_NAME: &str = "vector_store.type";
    /// Host address for vector store
    pub const HOST: &str = "vector_store.host";
    /// Port number for vector store
    pub const PORT: &str = "vector_store.port";
    /// Collection/table name for vector store
    pub const COLLECTION: &str = "vector_store.collection";
}
