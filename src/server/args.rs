//! Tool argument types for MCP server
//!
//! This module contains all the argument types used by the MCP tools.
//! These are extracted to improve code organization and maintainability.

use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

/// Arguments for the index_codebase tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for indexing a codebase directory")]
pub struct IndexCodebaseArgs {
    /// Path to the codebase directory to index
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(
        description = "Absolute or relative path to the directory containing code to index"
    )]
    pub path: String,
    /// Collection name for the indexed data
    #[schemars(description = "Name of the collection to store indexed data")]
    pub collection: Option<String>,
    /// File extensions to include (e.g., ["rs", "py", "js"])
    #[schemars(description = "Only index files with these extensions")]
    pub extensions: Option<Vec<String>>,
    /// Patterns to ignore during indexing
    #[schemars(description = "Glob patterns for files/directories to exclude")]
    pub ignore_patterns: Option<Vec<String>>,
    /// Maximum file size to index (in bytes)
    #[schemars(description = "Maximum size of files to index")]
    pub max_file_size: Option<u64>,
    /// Whether to follow symbolic links
    #[schemars(description = "Follow symbolic links during indexing")]
    pub follow_symlinks: Option<bool>,
    /// Optional JWT token for authentication
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

/// Search filters for narrowing down search results
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Filters to narrow down search results")]
pub struct SearchFilters {
    /// Filter by file extensions (e.g., ["rs", "py", "js"])
    #[schemars(description = "Only include files with these extensions")]
    pub file_extensions: Option<Vec<String>>,
    /// Filter by programming languages
    #[schemars(description = "Only include files in these programming languages")]
    pub languages: Option<Vec<String>>,
    /// Exclude files matching these patterns
    #[schemars(description = "Exclude files matching these glob patterns")]
    pub exclude_patterns: Option<Vec<String>>,
    /// Minimum similarity score (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0, message = "Min score must be between 0.0 and 1.0"))]
    #[schemars(description = "Minimum similarity score threshold")]
    pub min_score: Option<f32>,
}

/// Arguments for the search_code tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for searching code using natural language")]
pub struct SearchCodeArgs {
    /// Natural language query to search for
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Query must be between 1 and 1000 characters"
    ))]
    #[validate(custom(function = "validate_search_query", message = "Invalid search query"))]
    #[schemars(
        description = "The search query in natural language (e.g., 'find functions that handle authentication')"
    )]
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[validate(range(min = 1, max = 1000, message = "Limit must be between 1 and 1000"))]
    #[schemars(description = "Maximum number of search results to return")]
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Collection name to search in
    #[schemars(description = "Name of the collection to search")]
    pub collection: Option<String>,
    /// File extensions to search in
    #[schemars(description = "Only search in files with these extensions")]
    pub extensions: Option<Vec<String>>,
    /// Optional search filters
    #[schemars(description = "Optional filters to narrow down search results")]
    pub filters: Option<SearchFilters>,
    /// Optional JWT token for authentication
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

/// Arguments for getting indexing status
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for checking indexing status")]
pub struct GetIndexingStatusArgs {
    /// Collection name (default: 'default')
    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    #[validate(custom(
        function = "validate_collection_name",
        message = "Invalid collection name"
    ))]
    #[schemars(description = "Name of the collection to check status for")]
    #[serde(default = "default_collection")]
    pub collection: String,
}

/// Arguments for clearing an index
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for clearing an index")]
pub struct ClearIndexArgs {
    /// Collection name to clear (default: 'default')
    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    #[validate(custom(
        function = "validate_collection_name",
        message = "Invalid collection name"
    ))]
    #[schemars(description = "Name of the collection to clear")]
    #[serde(default = "default_collection")]
    pub collection: String,
}

fn default_limit() -> usize {
    10
}

fn default_collection() -> String {
    "default".to_string()
}

// Custom validation functions

fn validate_file_path(path: &str) -> Result<(), validator::ValidationError> {
    if path.is_empty() {
        return Err(validator::ValidationError::new("Path cannot be empty"));
    }

    if path.contains("..") {
        return Err(validator::ValidationError::new(
            "Path cannot contain directory traversal",
        ));
    }

    // Check for sensitive system paths
    let sensitive_paths = ["/etc/", "/proc/", "/sys/", "/root/", "/home/"];
    for sensitive in &sensitive_paths {
        if path.starts_with(sensitive) && !path.starts_with("/tmp/") {
            return Err(validator::ValidationError::new(
                "Access to sensitive system paths is not allowed",
            ));
        }
    }

    Ok(())
}

fn validate_search_query(query: &str) -> Result<(), validator::ValidationError> {
    if query.is_empty() {
        return Err(validator::ValidationError::new(
            "Search query cannot be empty",
        ));
    }

    if query.len() > 1000 {
        return Err(validator::ValidationError::new(
            "Search query is too long (maximum 1000 characters)",
        ));
    }

    // Basic XSS protection
    let dangerous_patterns = ["<script", "javascript:", "onload=", "onerror="];
    for pattern in &dangerous_patterns {
        if query.to_lowercase().contains(pattern) {
            return Err(validator::ValidationError::new(
                "Search query contains potentially dangerous content",
            ));
        }
    }

    Ok(())
}

fn validate_collection_name(name: &str) -> Result<(), validator::ValidationError> {
    if name.is_empty() {
        return Err(validator::ValidationError::new(
            "Collection name cannot be empty",
        ));
    }

    if name.len() > 100 {
        return Err(validator::ValidationError::new(
            "Collection name is too long (maximum 100 characters)",
        ));
    }

    // Only allow alphanumeric, underscore, and hyphen
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(validator::ValidationError::new(
            "Collection name can only contain letters, numbers, underscores, and hyphens",
        ));
    }

    Ok(())
}
