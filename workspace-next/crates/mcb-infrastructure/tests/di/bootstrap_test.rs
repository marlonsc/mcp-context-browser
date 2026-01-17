//! DI Container Bootstrap Tests
//!
//! Tests for the infrastructure container bootstrap process.

use mcb_infrastructure::config::ConfigBuilder;
use mcb_infrastructure::di::bootstrap::InfrastructureContainerBuilder;

#[tokio::test]
async fn test_infrastructure_container_creation() {
    let config = ConfigBuilder::new().build();
    let result = InfrastructureContainerBuilder::new(config).build().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_infrastructure_components() {
    let config = ConfigBuilder::new().build();
    let components = InfrastructureContainerBuilder::new(config)
        .build()
        .await
        .unwrap();

    // Test that components are accessible via public fields
    assert!(components
        .cache
        .get::<String>("test")
        .await
        .unwrap()
        .is_none());
    assert!(components
        .health
        .list_checks()
        .await
        .contains(&"system".to_string()));
}
