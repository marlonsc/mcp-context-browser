//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.

use mcb_infrastructure::di::bootstrap::DiContainerBuilder;

#[tokio::test]
async fn test_di_container_builder() {
    let container = DiContainerBuilder::new().build().await.unwrap();
    // Container builds successfully with null providers
    drop(container);
}
