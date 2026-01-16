//! Tests for argument validation

use mcb_server::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use validator::Validate;

#[test]
fn test_search_args_valid() {
    let args = SearchCodeArgs {
        query: "find authentication functions".to_string(),
        limit: 10,
        collection: Some("test".to_string()),
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_search_args_empty_query() {
    let args = SearchCodeArgs {
        query: "".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_search_args_query_too_long() {
    let args = SearchCodeArgs {
        query: "x".repeat(1001), // Exceeds 1000 character limit
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_search_args_limit_zero() {
    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 0,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_search_args_limit_too_high() {
    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 1001, // Exceeds 1000 limit
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_search_args_dangerous_content() {
    let args = SearchCodeArgs {
        query: "<script>alert('xss')</script>".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_index_args_valid() {
    let args = IndexCodebaseArgs {
        path: "/tmp/test".to_string(),
        collection: Some("test".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_index_args_empty_path() {
    let args = IndexCodebaseArgs {
        path: "".to_string(),
        collection: None,
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_index_args_path_traversal() {
    let args = IndexCodebaseArgs {
        path: "../../../etc/passwd".to_string(),
        collection: None,
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_clear_args_valid() {
    let args = ClearIndexArgs {
        collection: "my-collection".to_string(),
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_clear_args_empty_collection() {
    let args = ClearIndexArgs {
        collection: "".to_string(),
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_clear_args_invalid_characters() {
    let args = ClearIndexArgs {
        collection: "my/collection".to_string(),
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_clear_args_collection_too_long() {
    let args = ClearIndexArgs {
        collection: "a".repeat(101),
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_status_args_valid() {
    let args = GetIndexingStatusArgs {
        collection: "test".to_string(),
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_status_args_empty_collection() {
    let args = GetIndexingStatusArgs {
        collection: "".to_string(),
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_status_args_with_hyphen_underscore() {
    let args = GetIndexingStatusArgs {
        collection: "my-project_v2".to_string(),
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_status_args_alphanumeric() {
    let args = GetIndexingStatusArgs {
        collection: "project123".to_string(),
    };

    assert!(args.validate().is_ok());
}
