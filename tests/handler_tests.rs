//! Handler tests for MCP server tools
//!
//! Tests for clear_index, search_code, index_codebase, and get_indexing_status handlers.

use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

use mcp_context_browser::adapters::hybrid_search::{HybridSearchAdapter, HybridSearchMessage};
use mcp_context_browser::admin::service::AdminService;

// Test service creation function (copied from admin tests)
async fn create_test_admin_service() -> std::sync::Arc<dyn AdminService> {
    use arc_swap::ArcSwap;
    use mcp_context_browser::adapters::http_client::test_utils::NullHttpClientPool;
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
    let http_client = Arc::new(NullHttpClientPool::new());

    // Create event bus with a dummy subscriber to prevent channel closure
    let event_bus: std::sync::Arc<
        dyn mcp_context_browser::infrastructure::events::EventBusProvider,
    > = Arc::new(EventBus::with_default_capacity());
    // Keep a receiver alive to prevent the channel from being considered closed
    let _receiver = event_bus.subscribe().await;
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
use mcp_context_browser::application::{ContextService, IndexingService, SearchService};
use mcp_context_browser::domain::ports::{
    EmbeddingProvider, HybridSearchProvider, VectorStoreProvider,
};
use mcp_context_browser::infrastructure::auth::{AuthConfig, AuthService};
use mcp_context_browser::infrastructure::limits::{ResourceLimits, ResourceLimitsConfig};
use mcp_context_browser::server::args::{
    ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs,
};
use mcp_context_browser::server::auth::AuthHandler;
use mcp_context_browser::server::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
};

// ============================================================================
// Test Provider Setup
// ============================================================================

async fn create_indexing_service() -> Arc<IndexingService> {
    let (embedding_provider, vector_store_provider, hybrid_search_provider) =
        create_test_providers();
    let context_service = Arc::new(ContextService::new_with_providers(
        embedding_provider,
        vector_store_provider,
        hybrid_search_provider,
    ));
    Arc::new(IndexingService::new(context_service, None).unwrap())
}

fn create_test_providers() -> (
    Arc<dyn EmbeddingProvider>,
    Arc<dyn VectorStoreProvider>,
    Arc<dyn HybridSearchProvider>,
) {
    let embedding_provider = Arc::new(
        mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider::new(),
    );
    let vector_store_provider = Arc::new(
        mcp_context_browser::adapters::providers::vector_store::null::NullVectorStoreProvider::new(
        ),
    );
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    tokio::spawn(async move {
        let mut receiver = receiver;
        while let Some(msg) = receiver.recv().await {
            match msg {
                HybridSearchMessage::Search { respond_to, .. } => {
                    let _ = respond_to.send(Ok(Vec::new()));
                }
                HybridSearchMessage::GetStats { respond_to } => {
                    let _ = respond_to.send(std::collections::HashMap::new());
                }
                _ => {}
            }
        }
    });
    let hybrid_search_provider = Arc::new(HybridSearchAdapter::new(sender));
    (
        embedding_provider,
        vector_store_provider,
        hybrid_search_provider,
    )
}

// ============================================================================
// Real Service Implementations
// ============================================================================

// ============================================================================
// Test Utilities
// ============================================================================

fn create_auth_handler_disabled() -> AuthHandler {
    let config = AuthConfig {
        enabled: false,
        ..Default::default()
    };
    let auth_service = AuthService::new(config);
    AuthHandler::new(auth_service)
}

fn create_resource_limits_disabled() -> ResourceLimits {
    let config = ResourceLimitsConfig {
        enabled: false,
        ..Default::default()
    };
    ResourceLimits::new(config)
}

/// Helper to extract text from CallToolResult
fn extract_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| {
            if let rmcp::model::RawContent::Text(text_content) = &c.raw {
                Some(text_content.text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

// ============================================================================
// GetIndexingStatusHandler Tests
// ============================================================================

mod get_indexing_status_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_indexing_status_ready() {
        let admin_service = create_test_admin_service().await;
        let handler = GetIndexingStatusHandler::new(admin_service);

        let args = GetIndexingStatusArgs {
            collection: "default".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("System Status"));
        assert!(text.contains("Ready for search"));
        assert!(text.contains("default"));
    }

    #[tokio::test]
    async fn test_get_indexing_status_with_collection() {
        let admin_service = create_test_admin_service().await;
        let handler = GetIndexingStatusHandler::new(admin_service);

        let args = GetIndexingStatusArgs {
            collection: "test-collection".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        // With real services, indexing status depends on actual state
        // The response should contain status information
        assert!(
            text.contains("System Status") || text.contains("Ready") || text.contains("collection")
        );
    }

    #[tokio::test]
    async fn test_get_indexing_status_shows_metrics() {
        let admin_service = create_test_admin_service().await;
        let handler = GetIndexingStatusHandler::new(admin_service);

        let args = GetIndexingStatusArgs {
            collection: "default".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        // Check metrics are included
        assert!(text.contains("Performance"));
        assert!(text.contains("Total Queries"));
        assert!(text.contains("Cache Hit Rate"));
    }
}

// ============================================================================
// ClearIndexHandler Tests
// ============================================================================

mod clear_index_tests {
    use super::*;

    #[tokio::test]
    async fn test_clear_index_empty_collection_name() {
        let indexing_service = create_indexing_service().await;
        let handler = ClearIndexHandler::new(indexing_service);

        let args = ClearIndexArgs {
            collection: "   ".to_string(), // Empty after trim
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_clear_index_system_collection_blocked() {
        let indexing_service = create_indexing_service().await;
        let handler = ClearIndexHandler::new(indexing_service);

        let args = ClearIndexArgs {
            collection: "system".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("system collections"));
    }

    #[tokio::test]
    async fn test_clear_index_admin_collection_blocked() {
        let indexing_service = create_indexing_service().await;
        let handler = ClearIndexHandler::new(indexing_service);

        let args = ClearIndexArgs {
            collection: "admin".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("system collections"));
    }

    #[tokio::test]
    async fn test_clear_index_valid_collection() {
        let indexing_service = create_indexing_service().await;
        let handler = ClearIndexHandler::new(indexing_service);

        let args = ClearIndexArgs {
            collection: "test-collection".to_string(),
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        // Should either succeed or report an error (not validation error)
        assert!(
            text.contains("Completed Successfully") || text.contains("test-collection"),
            "Expected success message or collection reference, got: {}",
            text
        );
    }
}

// ============================================================================
// IndexCodebaseHandler Tests
// ============================================================================

mod index_codebase_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_index_codebase_path_is_file() {
        let indexing_service = create_indexing_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let handler = IndexCodebaseHandler::new(indexing_service, auth_handler, resource_limits);

        // Create a temporary file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let args = IndexCodebaseArgs {
            path: file_path.to_string_lossy().to_string(),
            token: None,
            collection: None,
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("not a directory"));
    }

    #[tokio::test]
    async fn test_index_codebase_valid_directory() {
        let indexing_service = create_indexing_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let handler = IndexCodebaseHandler::new(indexing_service, auth_handler, resource_limits);

        // Create a temporary directory with source files
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("main.rs");
        fs::write(&file_path, "fn main() {\n    println!(\"Hello\");\n}").unwrap();

        let args = IndexCodebaseArgs {
            path: temp_dir.path().to_string_lossy().to_string(),
            token: None,
            collection: Some("test".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        // Should succeed or show indexing result
        assert!(
            text.contains("Completed") || text.contains("chunks"),
            "Expected indexing result, got: {}",
            text
        );
    }

    #[tokio::test]
    async fn test_index_codebase_uses_default_collection() {
        let indexing_service = create_indexing_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let handler = IndexCodebaseHandler::new(indexing_service, auth_handler, resource_limits);

        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("test.rs"), "fn test() {}").unwrap();

        let args = IndexCodebaseArgs {
            path: temp_dir.path().to_string_lossy().to_string(),
            token: None,
            collection: None, // No collection specified
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
        };

        let result = handler.handle(Parameters(args)).await;

        // Should succeed - default collection "default" should be used
        assert!(result.is_ok());
    }
}

// ============================================================================
// SearchCodeHandler Tests
// ============================================================================

mod search_code_tests {
    use super::*;
    use mcp_context_browser::infrastructure::cache::{
        create_cache_provider, CacheBackendConfig, CacheConfig, SharedCacheProvider,
    };

    async fn create_search_service() -> Arc<SearchService> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new_with_providers(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        Arc::new(SearchService::new(context_service))
    }

    async fn create_cache_provider_test() -> SharedCacheProvider {
        let config = CacheConfig {
            enabled: true,
            backend: CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 3600,
            },
            namespaces: Default::default(),
        };
        create_cache_provider(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_search_code_empty_query() {
        let search_service = create_search_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let cache_provider = Some(create_cache_provider_test().await);
        let handler = SearchCodeHandler::new(
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        );

        let args = SearchCodeArgs {
            query: "   ".to_string(), // Empty after trim
            limit: 10,
            token: None,
            filters: None,
            collection: None,
            extensions: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("empty") || text.contains("cannot"));
    }

    #[tokio::test]
    async fn test_search_code_query_too_short() {
        let search_service = create_search_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let cache_provider = Some(create_cache_provider_test().await);
        let handler = SearchCodeHandler::new(
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        );

        let args = SearchCodeArgs {
            query: "ab".to_string(), // Less than 3 characters
            limit: 10,
            token: None,
            filters: None,
            collection: None,
            extensions: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("short") || text.contains("3 characters"));
    }

    #[tokio::test]
    async fn test_search_code_valid_query() {
        let search_service = create_search_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let cache_provider = Some(create_cache_provider_test().await);
        let handler = SearchCodeHandler::new(
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        );

        let args = SearchCodeArgs {
            query: "find error handling functions".to_string(),
            limit: 10,
            token: None,
            filters: None,
            collection: None,
            extensions: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        // Should show search results (even if empty)
        assert!(
            text.contains("Search") || text.contains("Results"),
            "Expected search response, got: {}",
            text
        );
    }

    #[tokio::test]
    async fn test_search_code_limit_clamped() {
        let search_service = create_search_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let cache_provider = Some(create_cache_provider_test().await);
        let handler = SearchCodeHandler::new(
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        );

        // Test with limit above maximum (should be clamped to 50)
        let args = SearchCodeArgs {
            query: "test query".to_string(),
            limit: 1000, // Above max
            token: None,
            filters: None,
            collection: None,
            extensions: None,
        };

        let result = handler.handle(Parameters(args)).await;

        // Should succeed - limit is silently clamped
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_code_query_too_long() {
        let search_service = create_search_service().await;
        let auth_handler = Arc::new(create_auth_handler_disabled());
        let resource_limits = Arc::new(create_resource_limits_disabled());
        let cache_provider = Some(create_cache_provider_test().await);
        let handler = SearchCodeHandler::new(
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        );

        // Create a query longer than 1000 characters
        let long_query = "a".repeat(1001);
        let args = SearchCodeArgs {
            query: long_query,
            limit: 10,
            token: None,
            filters: None,
            collection: None,
            extensions: None,
        };

        let result = handler.handle(Parameters(args)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let text = extract_text(&response);

        assert!(text.contains("Error"));
        assert!(text.contains("long") || text.contains("1000"));
    }
}
