//! Comprehensive input validation tests
//!
//! Tests for validator crate integration and business logic validation
//! throughout the MCP Context Browser application.

use mcp_context_browser::core::error::Error;
use mcp_context_browser::core::types::{CodeChunk, Language, Embedding};
use mcp_context_browser::server::args::{IndexCodebaseArgs, SearchCodeArgs, GetIndexingStatusArgs, ClearIndexArgs};
use validator::{Validate, ValidationErrors};

/// Test validation of server request arguments
#[cfg(test)]
mod server_args_validation_tests {
    use super::*;

    #[test]
    fn test_index_codebase_args_validation() {
        // Valid args should pass
        let valid_args = IndexCodebaseArgs {
            path: "/tmp/test".to_string(),
            collection: None,
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: Some("valid_token".to_string()),
        };
        assert!(valid_args.validate().is_ok());

        // Empty path should fail
        let invalid_args = IndexCodebaseArgs {
            path: "".to_string(),
            collection: None,
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        };
        assert!(invalid_args.validate().is_err());
    }

    #[test]
    fn test_search_code_args_validation() {
        // Valid search args should pass
        let valid_args = SearchCodeArgs {
            query: "find authentication".to_string(),
            limit: Some(10),
        };
        assert!(valid_args.validate().is_ok());

        // Empty query should fail
        let invalid_args = SearchCodeArgs {
            query: "".to_string(),
            limit: Some(5),
        };
        assert!(invalid_args.validate().is_err());

        // Invalid limit should fail
        let invalid_limit_args = SearchCodeArgs {
            query: "test query".to_string(),
            limit: Some(0), // Limit must be positive
        };
        assert!(invalid_limit_args.validate().is_err());
    }

    #[test]
    fn test_get_indexing_status_args_validation() {
        // Valid args should pass (may be empty)
        let valid_args = GetIndexingStatusArgs {};
        assert!(valid_args.validate().is_ok());
    }

    #[test]
    fn test_clear_index_args_validation() {
        // Valid collection name should pass
        let valid_args = ClearIndexArgs {
            collection: Some("test_collection".to_string()),
        };
        assert!(valid_args.validate().is_ok());

        // Empty collection name should fail
        let invalid_args = ClearIndexArgs {
            collection: Some("".to_string()),
        };
        assert!(invalid_args.validate().is_err());
    }
}

/// Test validation of core data structures
#[cfg(test)]
mod core_types_validation_tests {
    use super::*;

    #[test]
    fn test_code_chunk_validation() {
        // Valid code chunk should pass
        let valid_chunk = CodeChunk {
            id: "test_id".to_string(),
            content: "fn hello() { println!(\"world\"); }".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"test": "value"}),
        };
        assert!(valid_chunk.validate().is_ok());

        // Empty content should fail
        let invalid_chunk = CodeChunk {
            id: "test_id".to_string(),
            content: "".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };
        assert!(invalid_chunk.validate().is_err());

        // Invalid line numbers should fail
        let invalid_lines_chunk = CodeChunk {
            id: "test_id".to_string(),
            content: "test content".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 5,
            end_line: 3, // Start > End
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };
        assert!(invalid_lines_chunk.validate().is_err());
    }

    #[test]
    fn test_embedding_validation() {
        // Valid embedding should pass
        let valid_embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3, 0.4],
            model: "text-embedding-ada-002".to_string(),
        };
        assert!(valid_embedding.validate().is_ok());

        // Empty vector should fail
        let invalid_embedding = Embedding {
            vector: vec![],
            model: "test".to_string(),
        };
        assert!(invalid_embedding.validate().is_err());

        // Empty model should fail
        let invalid_model_embedding = Embedding {
            vector: vec![0.1, 0.2],
            model: "".to_string(),
        };
        assert!(invalid_model_embedding.validate().is_err());
    }
}

/// Test custom validation rules and business logic
#[cfg(test)]
mod custom_validation_tests {
    use super::*;

    #[test]
    fn test_path_validation_rules() {
        // Valid paths should pass
        assert!(validate_file_path("/tmp/test.rs").is_ok());
        assert!(validate_file_path("src/main.rs").is_ok());
        assert!(validate_file_path("./relative/path/file.py").is_ok());

        // Invalid paths should fail
        assert!(validate_file_path("").is_err()); // Empty
        assert!(validate_file_path("../escape").is_err()); // Directory traversal
        assert!(validate_file_path("/etc/passwd").is_err()); // Sensitive path
    }

    #[test]
    fn test_query_validation_rules() {
        // Valid queries should pass
        assert!(validate_search_query("find authentication").is_ok());
        assert!(validate_search_query("user login function").is_ok());

        // Invalid queries should fail
        assert!(validate_search_query("").is_err()); // Empty
        assert!(validate_search_query(&"x".repeat(1001)).is_err()); // Too long
        assert!(validate_search_query("query with <script>").is_err()); // XSS attempt
    }

    #[test]
    fn test_collection_name_validation() {
        // Valid collection names should pass
        assert!(validate_collection_name("default").is_ok());
        assert!(validate_collection_name("my_collection").is_ok());
        assert!(validate_collection_name("collection_123").is_ok());

        // Invalid collection names should fail
        assert!(validate_collection_name("").is_err()); // Empty
        assert!(validate_collection_name(&"a".repeat(101)).is_err()); // Too long
        assert!(validate_collection_name("invalid name").is_err()); // Spaces
        assert!(validate_collection_name("invalid@name").is_err()); // Special chars
    }

    #[test]
    fn test_limit_validation() {
        // Valid limits should pass
        assert!(validate_result_limit(1).is_ok());
        assert!(validate_result_limit(100).is_ok());
        assert!(validate_result_limit(1000).is_ok());

        // Invalid limits should fail
        assert!(validate_result_limit(0).is_err()); // Zero
        assert!(validate_result_limit(1001).is_err()); // Too high
    }
}

/// Test validation error handling and propagation
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_validation_error_propagation() {
        // Test that validation errors are properly wrapped
        let invalid_chunk = CodeChunk {
            id: "test".to_string(),
            content: "".to_string(), // Invalid: empty content
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            embedding: None,
        };

        let result = invalid_chunk.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("content"));
    }

    #[test]
    fn test_multiple_validation_errors() {
        // Test multiple validation failures
        let invalid_chunk = CodeChunk {
            id: "".to_string(), // Invalid: empty ID
            content: "".to_string(), // Invalid: empty content
            file_path: "".to_string(), // Invalid: empty path
            start_line: 0, // Invalid: zero line
            end_line: 0, // Invalid: zero line
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        let result = invalid_chunk.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        // Should have multiple field errors
        assert!(errors.field_errors().len() > 1);
    }
}

/// Test validation middleware integration
#[cfg(test)]
mod middleware_integration_tests {
    use super::*;

    #[test]
    fn test_request_validation_middleware() {
        // Test that validation middleware properly validates incoming requests
        let valid_request = IndexCodebaseArgs {
            path: "/tmp/valid/path".to_string(),
            collection: None,
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: Some("valid_token".to_string()),
        };

        let invalid_request = IndexCodebaseArgs {
            path: "".to_string(), // Invalid
            collection: None,
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        };

        // Simulate middleware validation
        assert!(validate_request(&valid_request).is_ok());
        assert!(validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validation_middleware_error_formatting() {
        // Test that validation errors are properly formatted for API responses
        let invalid_request = SearchCodeArgs {
            query: "", // Invalid: empty query
            limit: Some(0), // Invalid: zero limit
        };

        let result = validate_request(&invalid_request);
        assert!(result.is_err());

        let error_message = format_validation_errors(&result.unwrap_err());
        assert!(error_message.contains("query"));
        assert!(error_message.contains("limit"));
    }
}

// Helper functions for custom validation

fn validate_file_path(path: &str) -> Result<(), Error> {
    if path.is_empty() {
        return Err(Error::validation("File path cannot be empty"));
    }

    if path.contains("..") {
        return Err(Error::validation("File path cannot contain directory traversal"));
    }

    // Check for sensitive system paths
    let sensitive_paths = ["/etc/", "/proc/", "/sys/", "/root/", "/home/"];
    for sensitive in &sensitive_paths {
        if path.starts_with(sensitive) && !path.starts_with("/tmp/") {
            return Err(Error::validation("Access to sensitive system paths is not allowed"));
        }
    }

    Ok(())
}

fn validate_search_query(query: &str) -> Result<(), Error> {
    if query.is_empty() {
        return Err(Error::validation("Search query cannot be empty"));
    }

    if query.len() > 1000 {
        return Err(Error::validation("Search query is too long (maximum 1000 characters)"));
    }

    // Basic XSS protection
    let dangerous_patterns = ["<script", "javascript:", "onload=", "onerror="];
    for pattern in &dangerous_patterns {
        if query.to_lowercase().contains(pattern) {
            return Err(Error::validation("Search query contains potentially dangerous content"));
        }
    }

    Ok(())
}

fn validate_collection_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::validation("Collection name cannot be empty"));
    }

    if name.len() > 100 {
        return Err(Error::validation("Collection name is too long (maximum 100 characters)"));
    }

    // Only allow alphanumeric, underscore, and hyphen
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::validation("Collection name can only contain letters, numbers, underscores, and hyphens"));
    }

    Ok(())
}

fn validate_result_limit(limit: usize) -> Result<(), Error> {
    if limit == 0 {
        return Err(Error::validation("Result limit must be greater than zero"));
    }

    if limit > 1000 {
        return Err(Error::validation("Result limit cannot exceed 1000"));
    }

    Ok(())
}

fn validate_request<T: Validate>(request: &T) -> Result<(), ValidationErrors> {
    request.validate()
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            if let Some(message) = &error.message {
                messages.push(format!("{}: {}", field, message));
            } else {
                messages.push(format!("{}: {}", field, error.code));
            }
        }
    }

    messages.join("; ")
}