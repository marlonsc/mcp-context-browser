//! Health Check Tests

use mcb_infrastructure::health::{
    checkers, HealthCheck, HealthChecker, HealthRegistry, HealthResponse, HealthStatus,
};

#[tokio::test]
async fn test_health_check_creation() {
    let healthy_check = HealthCheck::healthy("test");
    assert_eq!(healthy_check.status, HealthStatus::Up);
    assert!(healthy_check.error.is_none());

    let failed_check = HealthCheck::failed("test", Some("error message".to_string()));
    assert_eq!(failed_check.status, HealthStatus::Down);
    assert_eq!(failed_check.error, Some("error message".to_string()));
}

#[tokio::test]
async fn test_health_response_aggregation() {
    let response = HealthResponse::new()
        .add_check(HealthCheck::healthy("check1"))
        .add_check(HealthCheck::healthy("check2"));

    assert_eq!(response.status, HealthStatus::Up);
    assert_eq!(response.checks.len(), 2);

    let degraded_response = response.add_check(HealthCheck::degraded("check3", None));
    assert_eq!(degraded_response.status, HealthStatus::Degraded);
}

#[tokio::test]
async fn test_health_registry() {
    let registry = HealthRegistry::new();

    // Register a simple checker
    registry
        .register_checker(
            "test".to_string(),
            checkers::ServiceHealthChecker::new("test", || Ok(())),
        )
        .await;

    let response = registry.perform_health_checks().await;
    assert_eq!(response.checks.len(), 1);
    assert!(response.checks["test"].status.is_healthy());

    let checks = registry.list_checks().await;
    assert_eq!(checks, vec!["test"]);
}

#[tokio::test]
async fn test_health_status_methods() {
    assert!(HealthStatus::Up.is_healthy());
    assert!(HealthStatus::Up.is_operational());
    assert!(HealthStatus::Degraded.is_operational());
    assert!(!HealthStatus::Down.is_healthy());
    assert!(!HealthStatus::Down.is_operational());
}

#[tokio::test]
async fn test_system_health_checker() {
    let checker = checkers::SystemHealthChecker::new();
    let result = checker.check_health().await;

    assert_eq!(result.name, "system");
    assert!(result.status.is_healthy());
    assert!(result.details.is_some());
}
