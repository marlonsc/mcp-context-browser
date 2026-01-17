//! DI Factory Implementation Tests
//!
//! Tests for factory implementations that create infrastructure components.

use mcb_infrastructure::di::factory::implementation::{
    DefaultCryptoServiceFactory, DefaultHealthRegistryFactory,
};
use mcb_infrastructure::di::factory::traits::{CryptoServiceFactory, HealthRegistryFactory};

#[tokio::test]
async fn test_crypto_service_factory() {
    let factory = DefaultCryptoServiceFactory::new();
    let service = factory.create_crypto_service().await.unwrap();

    // Test that the service can encrypt/decrypt
    let data = b"test data";
    let encrypted = service.encrypt(data).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(data.to_vec(), decrypted);
}

#[tokio::test]
async fn test_crypto_service_factory_with_custom_key() {
    let custom_key = vec![0u8; 32]; // 256-bit key
    let factory = DefaultCryptoServiceFactory::with_master_key(custom_key);
    let service = factory.create_crypto_service().await.unwrap();

    // Test that the service can encrypt/decrypt with custom key
    let data = b"test data with custom key";
    let encrypted = service.encrypt(data).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(data.to_vec(), decrypted);
}

#[tokio::test]
async fn test_health_registry_factory() {
    let factory = DefaultHealthRegistryFactory::new();
    let registry = factory.create_health_registry().await.unwrap();

    let checks = registry.list_checks().await;
    assert!(checks.contains(&"system".to_string()));
}

#[tokio::test]
async fn test_health_registry_factory_default() {
    let factory = DefaultHealthRegistryFactory::default();
    let registry = factory.create_health_registry().await.unwrap();

    // Verify that system health checker is registered
    let checks = registry.list_checks().await;
    assert!(checks.contains(&"system".to_string()));
}
