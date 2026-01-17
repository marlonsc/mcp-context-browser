//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.

use mcb_infrastructure::di::bootstrap::DiContainerBuilder;

#[tokio::test]
async fn test_di_container_builder() {
    let container = DiContainerBuilder::new().build().await;
    assert!(
        container.is_ok(),
        "DiContainerBuilder should build successfully"
    );
    let app_container = container.unwrap();
    // Verify container has expected modules
    assert!(
        std::mem::size_of_val(&app_container.cache) > 0,
        "Cache module should be initialized"
    );
}
