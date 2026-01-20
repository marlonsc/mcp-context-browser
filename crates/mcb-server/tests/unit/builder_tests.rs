//! Tests for McpServerBuilder

use mcb_server::builder::{BuilderError, McpServerBuilder};
use std::sync::Arc;

use crate::test_utils::mock_services::{
    MockContextService, MockIndexingService, MockSearchService,
};

#[test]
fn test_builder_all_services_provided() {
    let indexing_service = Arc::new(MockIndexingService::new());
    let context_service = Arc::new(MockContextService::new());
    let search_service = Arc::new(MockSearchService::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .try_build();

    assert!(result.is_ok());
}

#[test]
fn test_builder_missing_indexing_service() {
    let context_service = Arc::new(MockContextService::new());
    let search_service = Arc::new(MockSearchService::new());

    let result = McpServerBuilder::new()
        .with_context_service(context_service)
        .with_search_service(search_service)
        .try_build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "indexing service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_context_service() {
    let indexing_service = Arc::new(MockIndexingService::new());
    let search_service = Arc::new(MockSearchService::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_search_service(search_service)
        .try_build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "context service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_search_service() {
    let indexing_service = Arc::new(MockIndexingService::new());
    let context_service = Arc::new(MockContextService::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .try_build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "search service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_empty() {
    let result = McpServerBuilder::new().try_build();

    assert!(result.is_err());
}

#[test]
fn test_try_build_success() {
    let indexing_service = Arc::new(MockIndexingService::new());
    let context_service = Arc::new(MockContextService::new());
    let search_service = Arc::new(MockSearchService::new());

    let server = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .try_build();

    assert!(server.is_ok());
}

#[test]
fn test_builder_default() {
    let builder = McpServerBuilder::default();
    let result = builder.try_build();

    // Default builder has no services, so should fail
    assert!(result.is_err());
}
