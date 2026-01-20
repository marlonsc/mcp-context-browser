//! Tests for IndexCodebaseHandler

use mcb_server::args::IndexCodebaseArgs;
use mcb_server::handlers::IndexCodebaseHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use crate::test_utils::mock_services::MockIndexingService;
use crate::test_utils::test_fixtures::{
    create_temp_codebase, create_test_indexing_result, create_test_indexing_result_with_errors,
};

#[tokio::test]
async fn test_index_codebase_valid_path() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(10, 50, 2);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: codebase_path.to_string_lossy().to_string(),
        collection: Some("test".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_index_codebase_nonexistent_path() {
    let mock_service = MockIndexingService::new();
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: "/nonexistent/path/to/codebase".to_string(),
        collection: None,
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Should return a result indicating an error (path doesn't exist)
    assert!(result.is_ok());
    let response = result.expect("Expected response");
    // Nonexistent path should be reported as an error
    assert!(
        response.is_error.unwrap_or(false),
        "Nonexistent path should return is_error: true"
    );
}

#[tokio::test]
async fn test_index_codebase_empty_path() {
    let mock_service = MockIndexingService::new();
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: "".to_string(),
        collection: None,
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Empty path should fail validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_index_codebase_default_collection() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(5, 25, 0);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: codebase_path.to_string_lossy().to_string(),
        collection: None, // Should use "default"
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_index_codebase_service_error() {
    let (_temp_dir, codebase_path) = create_temp_codebase();

    let mock_service = MockIndexingService::new().with_failure("Storage quota exceeded");
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: codebase_path.to_string_lossy().to_string(),
        collection: Some("test".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Service errors are returned as successful responses with error content
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_index_codebase_with_errors_in_result() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result_with_errors(
        8,
        40,
        vec![
            "Failed to parse binary.bin".to_string(),
            "Unsupported file type: .xyz".to_string(),
        ],
    );

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: codebase_path.to_string_lossy().to_string(),
        collection: Some("test".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    // Response should include error count
}

#[tokio::test]
async fn test_index_codebase_file_instead_of_directory() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let file_path = codebase_path.join("lib.rs");

    let mock_service = MockIndexingService::new();
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: file_path.to_string_lossy().to_string(),
        collection: None,
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Should return success with error message (path is not a directory)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_index_codebase_with_extensions() {
    let (_temp_dir, codebase_path) = create_temp_codebase();
    let indexing_result = create_test_indexing_result(2, 10, 0);

    let mock_service = MockIndexingService::new().with_result(indexing_result);
    let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

    let args = IndexCodebaseArgs {
        path: codebase_path.to_string_lossy().to_string(),
        collection: Some("test".to_string()),
        extensions: Some(vec!["rs".to_string(), "py".to_string()]),
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}
