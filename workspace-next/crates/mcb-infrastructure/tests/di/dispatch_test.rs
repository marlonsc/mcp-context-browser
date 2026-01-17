//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.

use mcb_infrastructure::config::ConfigBuilder;
use mcb_infrastructure::di::bootstrap::{DiContainerBuilder, FullContainer};

#[tokio::test]
async fn test_di_container_builder() {
    let container = DiContainerBuilder::new().build().await.unwrap();
    // Container builds successfully with null providers
    drop(container);
}

#[tokio::test]
async fn test_full_container_creation() {
    let config = ConfigBuilder::new().build();
    let container = FullContainer::new(config).await.unwrap();

    // Test that services are accessible
    let _indexing = container.indexing_service();
    let _context = container.context_service();
    let _search = container.search_service();
}
