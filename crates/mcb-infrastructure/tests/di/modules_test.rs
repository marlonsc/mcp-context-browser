//! DI Module Tests
//!
//! Tests for Shaku-based dependency injection modules.
//!
//! Note: Full DI module integration tests require complex setup
//! with multiple infrastructure components. These tests verify
//! basic module imports and availability.

// Verify modules are exported correctly
use mcb_infrastructure::di::modules::{DomainServicesContainer, DomainServicesFactory};

#[test]
fn test_domain_services_factory_exists() {
    // Verify the factory type is accessible
    let _ = std::any::type_name::<DomainServicesFactory>();
}

#[test]
fn test_domain_services_container_exists() {
    // Verify the container type is accessible
    let _ = std::any::type_name::<DomainServicesContainer>();
}
