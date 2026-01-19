//! Tests for authentication infrastructure

use crate::infrastructure::auth::NullAuthService;
use mcb_application::ports::infrastructure::AuthServiceInterface;

#[test]
fn test_null_auth_service_creation() {
    let service = NullAuthService::new();
    let _service: Box<dyn AuthServiceInterface> = Box::new(service);
}

#[tokio::test]
async fn test_null_auth_service_validate_token() {
    let service = NullAuthService::new();
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
