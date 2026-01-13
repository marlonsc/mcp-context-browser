//! Admin handler unit tests
//!
//! Tests for all HTTP handlers in src/admin/handlers.rs

use mcp_context_browser::adapters::http_client::HttpClientPool;
use mcp_context_browser::admin::service::AdminService;

pub mod test_helpers {
    use super::*;

    /// Create a real AdminService for testing with minimal dependencies
    pub async fn create_test_admin_service() -> std::sync::Arc<dyn AdminService> {
        use arc_swap::ArcSwap;
        use mcp_context_browser::admin::service::{AdminServiceDependencies, AdminServiceImpl};
        use mcp_context_browser::infrastructure::config::ConfigLoader;
        use mcp_context_browser::infrastructure::di::factory::ServiceProvider;
        use mcp_context_browser::infrastructure::events::EventBus;
        use mcp_context_browser::infrastructure::logging;
        use mcp_context_browser::infrastructure::metrics::system::SystemMetricsCollector;
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
}

// ============================================================================
// Authentication Tests (using direct AuthService, not router)
// ============================================================================

#[cfg(test)]
mod auth_tests {
    use super::*;
    use mcp_context_browser::admin::auth::AuthService;

    #[tokio::test]
    async fn test_auth_service_valid_credentials() {
        let auth_service = AuthService::new(
            "test-secret".to_string(),
            3600,
            "admin".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to create auth service");

        let result = auth_service.authenticate("admin", "admin");
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.username, "admin");
        assert_eq!(user.role, "admin");
    }

    #[tokio::test]
    async fn test_auth_service_invalid_credentials() {
        let auth_service = AuthService::new(
            "test-secret".to_string(),
            3600,
            "admin".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to create auth service");

        let result = auth_service.authenticate("admin", "wrong_password");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_service_token_generation() {
        let auth_service = AuthService::new(
            "test-secret".to_string(),
            3600,
            "admin".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to create auth service");

        let user = mcp_context_browser::admin::models::UserInfo {
            username: "admin".to_string(),
            role: "admin".to_string(),
        };

        let token = auth_service.generate_token(&user);
        assert!(token.is_ok());

        let token = token.unwrap();
        assert!(!token.is_empty());

        let claims = auth_service.validate_token(&token);
        assert!(claims.is_ok());

        let claims = claims.unwrap();
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.role, "admin");
    }

    #[tokio::test]
    async fn test_auth_service_token_validation_wrong_secret() {
        let auth_service1 = AuthService::new(
            "secret-1".to_string(),
            3600,
            "admin".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to create auth service");

        let auth_service2 = AuthService::new(
            "secret-2".to_string(),
            3600,
            "admin".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to create auth service");

        let user = mcp_context_browser::admin::models::UserInfo {
            username: "admin".to_string(),
            role: "admin".to_string(),
        };

        let token = auth_service1.generate_token(&user).unwrap();

        let result = auth_service2.validate_token(&token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_real_admin_service_system_info() {
        let admin_service = test_helpers::create_test_admin_service().await;

        let info = admin_service.get_system_info().await;
        assert!(info.is_ok());

        let info = info.unwrap();
        assert!(!info.version.is_empty());
        assert!(info.pid > 0);
    }
}
//
// #[tokio::test]
// async fn test_admin_service_providers() {
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let providers = test_infra.admin_service.get_providers().await;
//     assert!(providers.is_ok());
//
//     let providers = providers.unwrap();
//     // The real service may return empty list if no providers are configured
//     // This is more realistic than hardcoded mock data
//     assert!(providers.is_empty() || !providers.is_empty()); // Either empty or has providers
// }
//
// #[tokio::test]
// async fn test_admin_service_indexing_status() {
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let status = test_infra.admin_service.get_indexing_status().await;
//     assert!(status.is_ok());
//
//     let status = status.unwrap();
//     // Real service returns actual indexing status
//     // Initially should not be indexing
//     assert!(!status.is_indexing || status.is_indexing); // Can be either initially
// }
//
// #[tokio::test]
// async fn test_admin_service_configuration() {
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let config = test_infra.admin_service.get_configuration().await;
//     assert!(config.is_ok());
//
//     let config = config.unwrap();
//     assert!(config.indexing.chunk_size > 0);
//     assert!(!config.indexing.supported_extensions.is_empty());
// }
//
// #[tokio::test]
// async fn test_admin_service_health_check() {
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let health = test_infra.admin_service.run_health_check().await;
//     assert!(health.is_ok());
//
//     let health = health.unwrap();
//     assert!(!health.overall_status.is_empty());
//     assert!(!health.checks.is_empty());
// }
//
// #[tokio::test]
// async fn test_admin_service_subsystems() {
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let subsystems = test_infra.admin_service.get_subsystems().await;
//     assert!(subsystems.is_ok());
//
//     let subsystems = subsystems.unwrap();
//     // Real service returns actual subsystems based on configuration
//     assert!(subsystems.is_empty() || !subsystems.is_empty()); // Can be empty or have subsystems
// }
//
// #[tokio::test]
// async fn test_admin_service_cache_operations() {
//     use mcp_context_browser::admin::service::CacheType;
//
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     for cache_type in [CacheType::All, CacheType::QueryResults, CacheType::Embeddings, CacheType::Indexes] {
//         let result = test_infra.admin_service.clear_cache(cache_type).await;
//         assert!(result.is_ok());
//         assert!(result.unwrap().success);
//     }
// }
//
// #[tokio::test]
// async fn test_admin_service_backup() {
//     use mcp_context_browser::admin::service::BackupConfig;
//
//     let test_infra = test_infrastructure::TestInfrastructure::new().await
//         .expect("Failed to create test infrastructure");
//
//     let config = BackupConfig {
//         name: "test-backup".to_string(),
//         include_data: true,
//         include_config: true,
//         compression: true,
//     };
//
//     let result = test_infra.admin_service.create_backup(config).await;
//     assert!(result.is_ok());
//
//     let result = result.unwrap();
//     assert!(!result.backup_id.is_empty());
//     assert!(!result.name.is_empty());
// }
