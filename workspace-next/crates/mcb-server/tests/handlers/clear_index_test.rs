//! Tests for ClearIndexHandler

use mcb_server::args::ClearIndexArgs;
use mcb_server::handlers::ClearIndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use crate::test_utils::mock_services::MockIndexingService;

#[tokio::test]
async fn test_clear_index_valid_collection() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "test-collection".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_clear_index_default_collection() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "default".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_index_empty_collection() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Empty collection should fail validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clear_index_service_error() {
    let mock_service = MockIndexingService::new().with_failure("Permission denied");
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "test-collection".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Service error should propagate as MCP error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clear_index_with_special_characters() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "my-project_v2".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Hyphens and underscores should be allowed
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_index_invalid_characters() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "my/project".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Forward slash should be invalid
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clear_index_very_long_name() {
    let mock_service = MockIndexingService::new();
    let handler = ClearIndexHandler::new(Arc::new(mock_service));

    let args = ClearIndexArgs {
        collection: "a".repeat(101), // Exceeds 100 character limit
    };

    let result = handler.handle(Parameters(args)).await;

    // Too long name should fail validation
    assert!(result.is_err());
}
