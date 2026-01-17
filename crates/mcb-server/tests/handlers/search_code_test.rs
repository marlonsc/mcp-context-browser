//! Tests for SearchCodeHandler

use mcb_server::args::SearchCodeArgs;
use mcb_server::handlers::SearchCodeHandler;
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use crate::test_utils::mock_services::MockSearchService;
use crate::test_utils::test_fixtures::create_test_search_results;

#[tokio::test]
async fn test_search_code_valid_query() {
    let results = create_test_search_results(3);
    let mock_service = MockSearchService::new().with_results(results);
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "find authentication functions".to_string(),
        limit: 10,
        collection: Some("test".to_string()),
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_search_code_empty_query() {
    let mock_service = MockSearchService::new();
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Empty query should fail validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_code_whitespace_only_query() {
    let mock_service = MockSearchService::new();
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "   ".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Whitespace-only query should fail validation after trimming
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_code_default_limit() {
    let results = create_test_search_results(15);
    let mock_service = MockSearchService::new().with_results(results);
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 10, // default limit
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_code_custom_limit() {
    let results = create_test_search_results(50);
    let mock_service = MockSearchService::new().with_results(results);
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 25,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_code_service_error() {
    let mock_service = MockSearchService::new().with_failure("Database connection failed");
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    // Service error should propagate as MCP error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_code_no_results() {
    let mock_service = MockSearchService::new().with_results(vec![]);
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "nonexistent code pattern".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    // Response should contain empty results message
}

#[tokio::test]
async fn test_search_code_with_collection() {
    let results = create_test_search_results(5);
    let mock_service = MockSearchService::new().with_results(results);
    let handler = SearchCodeHandler::new(Arc::new(mock_service));

    let args = SearchCodeArgs {
        query: "test query".to_string(),
        limit: 10,
        collection: Some("my-project".to_string()),
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
}
