//! End-to-end workflow tests for admin interface
//!
//! These tests verify complete admin workflows using the mock service.

use mcp_context_browser::admin::service::AdminService;
use std::collections::HashMap;

// Import the mock from handler_tests
use super::handler_tests::test_helpers::MockAdminService;

// ============================================================================
// E2E: Login and Get System Info Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_login_and_system_info() {
    use mcp_context_browser::admin::auth::AuthService;
    use mcp_context_browser::admin::models::UserInfo;

    // 1. Create auth service and authenticate
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let result = auth_service.authenticate("admin", "admin");
    assert!(result.is_ok());

    // 2. Generate token
    let user = result.unwrap();
    let token = auth_service.generate_token(&user);
    assert!(token.is_ok());

    // 3. Validate token
    let token = token.unwrap();
    let claims = auth_service.validate_token(&token);
    assert!(claims.is_ok());

    // 4. Get system info using admin service
    let service = MockAdminService::default();
    let info = service.get_system_info().await;
    assert!(info.is_ok());

    let info = info.unwrap();
    assert!(!info.version.is_empty());
}

// ============================================================================
// E2E: Configuration Update Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_config_update_workflow() {
    let service = MockAdminService::default();

    // 1. Get current configuration
    let config = service.get_configuration().await;
    assert!(config.is_ok());
    let original_config = config.unwrap();

    // 2. Validate a configuration change
    let mut updates = HashMap::new();
    updates.insert("indexing.chunk_size".to_string(), serde_json::json!(1024));

    let warnings = service.validate_configuration(&updates).await;
    assert!(warnings.is_ok());

    // 3. Update configuration
    let result = service.update_configuration(updates.clone(), "test_user").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);

    // 4. Check configuration history
    let history = service.get_configuration_history(Some(10)).await;
    assert!(history.is_ok());
}

// ============================================================================
// E2E: Backup and Restore Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_backup_restore_workflow() {
    use mcp_context_browser::admin::service::BackupConfig;

    let service = MockAdminService::default();

    // 1. Create a backup
    let backup_config = BackupConfig {
        include_indexes: true,
        include_config: true,
        compression: true,
    };

    let backup_result = service.create_backup(backup_config).await;
    assert!(backup_result.is_ok());

    let backup = backup_result.unwrap();
    let backup_id = backup.backup_id.clone();
    assert!(!backup_id.is_empty());

    // 2. List backups
    let backups = service.list_backups().await;
    assert!(backups.is_ok());

    // 3. Restore from backup
    let restore_result = service.restore_backup(&backup_id).await;
    assert!(restore_result.is_ok());
    assert!(restore_result.unwrap().success);
}

// ============================================================================
// E2E: Cache Clear Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_cache_clear_workflow() {
    use mcp_context_browser::admin::service::CacheType;

    let service = MockAdminService::default();

    // Clear each cache type
    for cache_type in [
        CacheType::QueryResults,
        CacheType::Embeddings,
        CacheType::Indexes,
        CacheType::All,
    ] {
        let result = service.clear_cache(cache_type).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}

// ============================================================================
// E2E: Health and Diagnostics Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_health_diagnostics_workflow() {
    use mcp_context_browser::admin::service::PerformanceTestConfig;

    let service = MockAdminService::default();

    // 1. Run health check
    let health = service.run_health_check().await;
    assert!(health.is_ok());
    assert_eq!(health.unwrap().overall_status, "healthy");

    // 2. Test provider connectivity
    let connectivity = service.test_provider_connectivity("embedding-1").await;
    assert!(connectivity.is_ok());
    assert!(connectivity.unwrap().success);

    // 3. Run performance test
    let perf_config = PerformanceTestConfig {
        query: "test query".to_string(),
        iterations: 5,
    };

    let perf_result = service.run_performance_test(perf_config).await;
    assert!(perf_result.is_ok());
    assert_eq!(perf_result.unwrap().iterations_run, 5);
}

// ============================================================================
// E2E: Subsystem Control Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_subsystem_control_workflow() {
    use mcp_context_browser::admin::service::SubsystemSignal;

    let service = MockAdminService::default();

    // 1. Get all subsystems
    let subsystems = service.get_subsystems().await;
    assert!(subsystems.is_ok());
    let subsystems = subsystems.unwrap();
    assert!(!subsystems.is_empty());

    // 2. Get subsystem ID
    let subsystem_id = &subsystems[0].id;

    // 3. Send status signal
    let result = service
        .send_subsystem_signal(subsystem_id, SubsystemSignal::Status)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

// ============================================================================
// E2E: Provider Management Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_provider_management_workflow() {
    let service = MockAdminService::default();

    // 1. List providers
    let providers = service.get_providers().await;
    assert!(providers.is_ok());

    // 2. Add a new provider
    let config = serde_json::json!({
        "name": "New Provider",
        "model": "test-model",
        "api_key": "test-key"
    });

    let new_provider = service.add_provider("embedding", config).await;
    assert!(new_provider.is_ok());

    let provider = new_provider.unwrap();
    assert_eq!(provider.provider_type, "embedding");

    // 3. Restart a provider
    let restart_result = service.restart_provider(&provider.id).await;
    assert!(restart_result.is_ok());
    assert!(restart_result.unwrap().success);

    // 4. Remove the provider
    let remove_result = service.remove_provider(&provider.id).await;
    assert!(remove_result.is_ok());
}

// ============================================================================
// E2E: Index Management Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_index_management_workflow() {
    let service = MockAdminService::default();

    // 1. Get indexing status
    let status = service.get_indexing_status().await;
    assert!(status.is_ok());

    // 2. Rebuild index
    let rebuild_result = service.rebuild_index("main-index").await;
    assert!(rebuild_result.is_ok());
    assert!(rebuild_result.unwrap().success);
}

// ============================================================================
// E2E: Logging Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_logging_workflow() {
    use mcp_context_browser::admin::service::{LogExportFormat, LogFilter};

    let service = MockAdminService::default();

    // 1. Get log stats
    let stats = service.get_log_stats().await;
    assert!(stats.is_ok());

    // 2. Get logs with filter
    let filter = LogFilter {
        level: Some("error".to_string()),
        module: None,
        start_time: None,
        end_time: None,
        limit: Some(100),
    };

    let logs = service.get_logs(filter.clone()).await;
    assert!(logs.is_ok());

    // 3. Export logs
    let filename = service.export_logs(filter, LogExportFormat::Json).await;
    assert!(filename.is_ok());
    assert!(!filename.unwrap().is_empty());
}

// ============================================================================
// E2E: Routes Management Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_routes_management_workflow() {
    let service = MockAdminService::default();

    // 1. Get all routes
    let routes = service.get_routes().await;
    assert!(routes.is_ok());
    assert!(!routes.unwrap().is_empty());

    // 2. Reload routes
    let reload_result = service.reload_routes().await;
    assert!(reload_result.is_ok());
    assert!(reload_result.unwrap().success);
}

// ============================================================================
// E2E: Full Admin Session Workflow
// ============================================================================

#[tokio::test]
async fn test_e2e_full_admin_session() {
    use mcp_context_browser::admin::auth::AuthService;
    use mcp_context_browser::admin::service::{BackupConfig, CacheType};

    let service = MockAdminService::default();

    // 1. Authenticate
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .unwrap();

    let user = auth_service.authenticate("admin", "admin").unwrap();
    let _token = auth_service.generate_token(&user).unwrap();

    // 2. Check system status
    let info = service.get_system_info().await.unwrap();
    assert!(!info.version.is_empty());

    // 3. Check health
    let health = service.run_health_check().await.unwrap();
    assert_eq!(health.overall_status, "healthy");

    // 4. Get configuration
    let config = service.get_configuration().await.unwrap();
    assert!(config.indexing.chunk_size > 0);

    // 5. Create backup
    let backup = service
        .create_backup(BackupConfig {
            include_indexes: true,
            include_config: true,
            compression: true,
        })
        .await
        .unwrap();
    assert!(!backup.backup_id.is_empty());

    // 6. Clear cache
    let clear_result = service.clear_cache(CacheType::All).await.unwrap();
    assert!(clear_result.success);

    // 7. Get performance metrics
    let metrics = service.get_performance_metrics().await.unwrap();
    assert!(metrics.total_queries >= 0);
}
