//! Tests for authentication infrastructure

use mcb_application::ports::infrastructure::AuthServiceInterface;
use mcb_infrastructure::infrastructure::auth::NullAuthService;

#[test]
fn test_null_auth_service_creation() {
    let service = NullAuthService::new();
    // Test that service can be created without panicking
    let _service: Box<dyn AuthServiceInterface> = Box::new(service);
}

#[tokio::test]
async fn test_null_auth_service_validate_token() {
    let service = NullAuthService::new();

    // Null implementation always returns true
    let result = service.validate_token("any-token").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_null_auth_service_generate_token() {
    let service = NullAuthService::new();

    let result = service.generate_token("test-subject").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null-token");
}

#[tokio::test]
async fn test_null_auth_service_new() {
    let service = NullAuthService::new();

    // Test that new implementation works
    let result = service.validate_token("token").await;
    assert!(result.is_ok());
}
