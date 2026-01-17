//! Tests for GetIndexingStatusHandler

use mcb_server::args::GetIndexingStatusArgs;
use mcb_server::handlers::GetIndexingStatusHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use crate::test_utils::mock_services::MockIndexingService;
use crate::test_utils::test_fixtures::{create_idle_status, create_in_progress_status};

#[tokio::test]
async fn test_get_status_idle() {
    let status = create_idle_status();
    let mock_service = MockIndexingService::new().with_status(status);
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "test".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_get_status_in_progress() {
    let status = create_in_progress_status(0.5, "src/main.rs");
    let mock_service = MockIndexingService::new().with_status(status);
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "test".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_status_nearly_complete() {
    let status = create_in_progress_status(0.95, "src/lib.rs");
    let mock_service = MockIndexingService::new().with_status(status);
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "test".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_status_collection_validation() {
    let mock_service = MockIndexingService::new();
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Empty collection should fail validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_status_default_collection() {
    let status = create_idle_status();
    let mock_service = MockIndexingService::new().with_status(status);
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "default".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_status_invalid_collection_name() {
    let mock_service = MockIndexingService::new();
    let handler = GetIndexingStatusHandler::new(Arc::new(mock_service));

    let args = GetIndexingStatusArgs {
        collection: "../escape".to_string(),
    };

    let result = handler.handle(Parameters(args)).await;

    // Path traversal should be invalid
    assert!(result.is_err());
}
