//! Admin service unit tests
//!
//! These tests verify the AdminService trait implementation contract

use mcp_context_browser::admin::service::AdminService;

// Import the mock from handler_tests
use super::handler_tests::test_helpers::MockAdminService;

// ============================================================================
// System Information Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_system_info() {
    let service = MockAdminService::default();
    let result = service.get_system_info().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_providers() {
    let service = MockAdminService::default();
    let result = service.get_providers().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_indexing_status() {
    let service = MockAdminService::default();
    let result = service.get_indexing_status().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_performance_metrics() {
    let service = MockAdminService::default();
    let result = service.get_performance_metrics().await;
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.cache_hit_rate >= 0.0 && metrics.cache_hit_rate <= 1.0);
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_configuration() {
    let service = MockAdminService::default();
    let result = service.get_configuration().await;
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.indexing.chunk_size > 0);
}

#[tokio::test]
async fn test_service_update_configuration() {
    use std::collections::HashMap;

    let service = MockAdminService::default();
    let mut updates = HashMap::new();
    updates.insert("test.key".to_string(), serde_json::json!("value"));

    let result = service.update_configuration(updates, "test_user").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_service_validate_configuration() {
    use std::collections::HashMap;

    let service = MockAdminService::default();
    let updates = HashMap::new();
    let result = service.validate_configuration(&updates).await;
    assert!(result.is_ok());
}

// ============================================================================
// Logging Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_logs() {
    use mcp_context_browser::admin::service::LogFilter;

    let service = MockAdminService::default();
    let filter = LogFilter {
        level: None,
        module: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
    };
    let result = service.get_logs(filter).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_log_stats() {
    let service = MockAdminService::default();
    let result = service.get_log_stats().await;
    assert!(result.is_ok());
}

// ============================================================================
// Maintenance Tests
// ============================================================================

#[tokio::test]
async fn test_service_clear_cache() {
    use mcp_context_browser::admin::service::CacheType;

    let service = MockAdminService::default();
    let result = service.clear_cache(CacheType::All).await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_service_restart_provider() {
    let service = MockAdminService::default();
    let result = service.restart_provider("test-provider").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_service_rebuild_index() {
    let service = MockAdminService::default();
    let result = service.rebuild_index("main-index").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_service_cleanup_data() {
    use mcp_context_browser::admin::service::CleanupConfig;

    let service = MockAdminService::default();
    let config = CleanupConfig { older_than_days: 30 };
    let result = service.cleanup_data(config).await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

// ============================================================================
// Diagnostic Tests
// ============================================================================

#[tokio::test]
async fn test_service_run_health_check() {
    let service = MockAdminService::default();
    let result = service.run_health_check().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().overall_status, "healthy");
}

#[tokio::test]
async fn test_service_test_connectivity() {
    let service = MockAdminService::default();
    let result = service.test_provider_connectivity("test-provider").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_service_run_performance_test() {
    use mcp_context_browser::admin::service::PerformanceTestConfig;

    let service = MockAdminService::default();
    let config = PerformanceTestConfig {
        query: "test".to_string(),
        iterations: 5,
    };
    let result = service.run_performance_test(config).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().iterations_run, 5);
}

// ============================================================================
// Backup Tests
// ============================================================================

#[tokio::test]
async fn test_service_create_backup() {
    use mcp_context_browser::admin::service::BackupConfig;

    let service = MockAdminService::default();
    let config = BackupConfig {
        include_indexes: true,
        include_config: true,
        compression: true,
    };
    let result = service.create_backup(config).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().backup_id.is_empty());
}

#[tokio::test]
async fn test_service_list_backups() {
    let service = MockAdminService::default();
    let result = service.list_backups().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_restore_backup() {
    let service = MockAdminService::default();
    let result = service.restore_backup("backup-123").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

// ============================================================================
// Subsystem Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_subsystems() {
    let service = MockAdminService::default();
    let result = service.get_subsystems().await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[tokio::test]
async fn test_service_send_signal() {
    use mcp_context_browser::admin::service::SubsystemSignal;

    let service = MockAdminService::default();
    let result = service
        .send_subsystem_signal("indexing", SubsystemSignal::Status)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

// ============================================================================
// Routes Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_routes() {
    let service = MockAdminService::default();
    let result = service.get_routes().await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[tokio::test]
async fn test_service_reload_routes() {
    let service = MockAdminService::default();
    let result = service.reload_routes().await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}
