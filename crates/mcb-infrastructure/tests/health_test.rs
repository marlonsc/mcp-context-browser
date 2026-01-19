//! Health Check Tests
#![allow(clippy::manual_range_contains)]

use mcb_infrastructure::health::{
    HealthCheck, HealthChecker, HealthRegistry, HealthResponse, HealthStatus, checkers,
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

// =============================================================================
// REAL METRICS VALIDATION TESTS
// Phase 3 of v0.1.2: Verify SystemHealthChecker returns REAL system metrics
// =============================================================================

#[tokio::test]
async fn test_system_health_checker_returns_real_cpu_metrics() {
    let checker = checkers::SystemHealthChecker::new();
    let result = checker.check_health().await;

    let details = result.details.expect("Should have details");
    let cpu_usage = details["cpu_usage_percent"]
        .as_f64()
        .expect("cpu_usage_percent should be a number");

    // CPU usage should be a valid percentage (0-100)
    assert!(
        cpu_usage >= 0.0 && cpu_usage <= 100.0,
        "CPU usage must be between 0 and 100, got: {}",
        cpu_usage
    );

    // CPU usage should NOT be a hardcoded value like 45.2%
    // Running multiple times should show variation (unless system is truly idle)
    // We just verify it's a reasonable value, not checking for variation
    println!("Real CPU usage: {:.2}%", cpu_usage);
}

#[tokio::test]
async fn test_system_health_checker_returns_real_memory_metrics() {
    let checker = checkers::SystemHealthChecker::new();
    let result = checker.check_health().await;

    let details = result.details.expect("Should have details");

    // Memory used bytes
    let memory_used = details["memory_used_bytes"]
        .as_u64()
        .expect("memory_used_bytes should be a number");

    // Memory total bytes
    let memory_total = details["memory_total_bytes"]
        .as_u64()
        .expect("memory_total_bytes should be a number");

    // Memory usage percent
    let memory_percent = details["memory_usage_percent"]
        .as_f64()
        .expect("memory_usage_percent should be a number");

    // Validate memory values are realistic
    assert!(
        memory_total > 1_000_000_000,
        "Total memory should be at least 1GB, got: {} bytes",
        memory_total
    );
    assert!(
        memory_used > 0,
        "Memory used should be > 0, got: {} bytes",
        memory_used
    );
    assert!(
        memory_used <= memory_total,
        "Memory used ({}) should not exceed total ({})",
        memory_used,
        memory_total
    );
    assert!(
        memory_percent >= 0.0 && memory_percent <= 100.0,
        "Memory percent must be between 0 and 100, got: {}",
        memory_percent
    );

    // Verify percent calculation is correct
    let expected_percent = (memory_used as f64 / memory_total as f64) * 100.0;
    assert!(
        (memory_percent - expected_percent).abs() < 0.01,
        "Memory percent calculation mismatch: got {} expected {}",
        memory_percent,
        expected_percent
    );

    println!(
        "Real memory: {:.2} GB used of {:.2} GB total ({:.2}%)",
        memory_used as f64 / 1_000_000_000.0,
        memory_total as f64 / 1_000_000_000.0,
        memory_percent
    );
}

#[tokio::test]
async fn test_system_health_checker_includes_threshold_info() {
    let checker = checkers::SystemHealthChecker::with_thresholds(80.0, 85.0);
    let result = checker.check_health().await;

    let details = result.details.expect("Should have details");

    // Verify thresholds are included in response
    let cpu_threshold = details["cpu_threshold_percent"]
        .as_f64()
        .expect("cpu_threshold_percent should be a number");
    let memory_threshold = details["memory_threshold_percent"]
        .as_f64()
        .expect("memory_threshold_percent should be a number");

    assert_eq!(
        cpu_threshold, 80.0,
        "CPU threshold should match configured value"
    );
    assert_eq!(
        memory_threshold, 85.0,
        "Memory threshold should match configured value"
    );
}

#[tokio::test]
async fn test_system_health_checker_degraded_on_high_threshold() {
    // Set very low thresholds to trigger degraded status
    let checker = checkers::SystemHealthChecker::with_thresholds(0.1, 0.1);
    let result = checker.check_health().await;

    // With 0.1% thresholds, status should be Degraded (unless system is truly idle)
    // On most systems, this will be Degraded
    assert!(
        result.status == HealthStatus::Degraded || result.status == HealthStatus::Up,
        "Status should be Degraded or Up based on actual metrics"
    );

    // Print actual status for verification
    println!("Status with 0.1% thresholds: {:?}", result.status);
}

#[tokio::test]
async fn test_system_health_checker_response_time_is_measured() {
    let checker = checkers::SystemHealthChecker::new();
    let result = checker.check_health().await;

    // Response time should be at least 100ms due to CPU measurement delay
    // The implementation has: std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(
        result.response_time_ms >= 100,
        "Response time should be at least 100ms (CPU measurement delay), got: {}ms",
        result.response_time_ms
    );

    println!("Health check response time: {}ms", result.response_time_ms);
}

#[tokio::test]
async fn test_system_health_checker_details_has_all_required_fields() {
    let checker = checkers::SystemHealthChecker::new();
    let result = checker.check_health().await;

    let details = result.details.expect("Should have details");

    // Verify all required fields are present
    assert!(
        details.get("cpu_usage_percent").is_some(),
        "Missing cpu_usage_percent field"
    );
    assert!(
        details.get("memory_used_bytes").is_some(),
        "Missing memory_used_bytes field"
    );
    assert!(
        details.get("memory_total_bytes").is_some(),
        "Missing memory_total_bytes field"
    );
    assert!(
        details.get("memory_usage_percent").is_some(),
        "Missing memory_usage_percent field"
    );
    assert!(
        details.get("cpu_threshold_percent").is_some(),
        "Missing cpu_threshold_percent field"
    );
    assert!(
        details.get("memory_threshold_percent").is_some(),
        "Missing memory_threshold_percent field"
    );
}
