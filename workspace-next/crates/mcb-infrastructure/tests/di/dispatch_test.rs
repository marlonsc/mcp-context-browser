//! DI Component Dispatch Tests
//!
//! Tests for the component dispatcher and infrastructure initializer.

use mcb_infrastructure::config::ConfigBuilder;
use mcb_infrastructure::di::bootstrap::{ConfigHealthAccess, StorageComponentsAccess};
use mcb_infrastructure::di::dispatch::{ComponentDispatcher, InfrastructureInitializer};

#[tokio::test]
async fn test_component_dispatcher() {
    let config = ConfigBuilder::new().build();
    let dispatcher = ComponentDispatcher::new(config);

    let container = dispatcher.dispatch().await.unwrap();

    // Verify components are accessible
    assert!(container
        .cache()
        .get::<String>("test")
        .await
        .unwrap()
        .is_none());
    assert!(container
        .health()
        .list_checks()
        .await
        .contains(&"system".to_string()));
}

#[tokio::test]
async fn test_infrastructure_initializer() {
    let config = ConfigBuilder::new().build();
    let initializer = InfrastructureInitializer::new(config);

    let container = initializer.initialize().await.unwrap();

    // Test that initialization succeeded
    assert!(container
        .health()
        .list_checks()
        .await
        .contains(&"system".to_string()));
}
