//! End-to-end workflow tests for admin interface
//!
//! These tests verify complete admin workflows using real services.

use std::collections::HashMap;

use mcp_context_browser::server::admin::service::AdminService;

/// Create a real AdminService for testing with minimal dependencies
async fn create_test_admin_service() -> std::sync::Arc<dyn AdminService> {
    use arc_swap::ArcSwap;
    use mcp_context_browser::adapters::http_client::HttpClientPool;
    use mcp_context_browser::infrastructure::config::ConfigLoader;
    use mcp_context_browser::infrastructure::di::factory::ServiceProvider;
    use mcp_context_browser::infrastructure::events::EventBus;
    use mcp_context_browser::infrastructure::logging;
    use mcp_context_browser::infrastructure::metrics::system::SystemMetricsCollector;
    use mcp_context_browser::server::admin::service::{AdminServiceDependencies, AdminServiceImpl};
    use mcp_context_browser::server::metrics::McpPerformanceMetrics;
    use mcp_context_browser::server::operations::McpIndexingOperations;
    use std::sync::Arc;

    // Create minimal test dependencies
    let performance_metrics = Arc::new(McpPerformanceMetrics::default());
    let indexing_operations = Arc::new(McpIndexingOperations::default());
    let service_provider = Arc::new(ServiceProvider::new());
    let system_collector = Arc::new(SystemMetricsCollector::new());
    let http_client = Arc::new(HttpClientPool::new().expect("Failed to create HTTP client"));

    // Create event bus and log buffer
    let event_bus = Arc::new(EventBus::with_default_capacity());
    let log_buffer = logging::create_shared_log_buffer(1000);

    // Load config from file instead of using Config::default()
    let loader = ConfigLoader::new();
    let loaded_config = loader
        .load()
        .await
        .expect("Failed to load config for tests");
    let config = Arc::new(ArcSwap::from_pointee(loaded_config));

    // Create the real admin service
    let deps = AdminServiceDependencies {
        performance_metrics,
        indexing_operations,
        service_provider,
        system_collector,
        http_client,
        event_bus,
        log_buffer,
        config,
    };
    let admin_service = AdminServiceImpl::new(deps);

    Arc::new(admin_service)
}

/// Test infrastructure for setting up real services

#[tokio::test]
async fn test_e2e_login_and_system_info() {
    use mcp_context_browser::server::admin::auth::AuthService;

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
    let service = create_test_admin_service().await;
    let info = service.get_system_info().await;
    assert!(info.is_ok());

    let info = info.unwrap();
    assert!(!info.version.is_empty());
}

#[tokio::test]
async fn test_e2e_config_update_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let service = create_test_admin_service().await;

    // 1. Get current configuration
    let config = service.get_configuration().await;
    assert!(config.is_ok());
    let _original_config = config.unwrap();

    // 2. Validate a configuration change
    let mut updates = HashMap::new();
    updates.insert("indexing.chunk_size".to_string(), serde_json::json!(1024));

    let warnings = service.validate_configuration(&updates).await;
    assert!(warnings.is_ok());

    // 3. Update configuration
    let result = service
        .update_configuration(updates.clone(), "test_user")
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);

    // 4. Check configuration history
    let history = service.get_configuration_history(Some(10)).await;
    assert!(history.is_ok());

    Ok(())
}

// Test service creation function
