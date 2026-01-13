//! Test infrastructure for setting up real dependencies
//!
//! This module provides utilities to create real service instances
//! with test-appropriate configurations instead of mocks.

use arc_swap::ArcSwap;
use std::sync::Arc;

use mcp_context_browser::adapters::http_client::{HttpClientPool, HttpClientProvider};
use mcp_context_browser::application::search::SearchService;
use mcp_context_browser::infrastructure::config::Config;
use mcp_context_browser::infrastructure::di::factory::{ServiceProvider, ServiceProviderInterface};
use mcp_context_browser::infrastructure::events::{EventBus, SharedEventBusProvider};
use mcp_context_browser::infrastructure::logging::SharedLogBuffer;
use mcp_context_browser::infrastructure::metrics::system::{
    SystemMetricsCollector, SystemMetricsCollectorInterface,
};
use mcp_context_browser::server::admin::service::{
    AdminService, AdminServiceDependencies, AdminServiceImpl,
};
use mcp_context_browser::server::metrics::{McpPerformanceMetrics, PerformanceMetricsInterface};
use mcp_context_browser::server::operations::{IndexingOperationsInterface, McpIndexingOperations};

/// Test infrastructure for setting up real services
pub struct TestInfrastructure {
    pub admin_service: Arc<dyn AdminService>,
    pub config: Arc<ArcSwap<Config>>,
    pub event_bus: SharedEventBusProvider,
    pub log_buffer: SharedLogBuffer,
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    pub service_provider: Arc<dyn ServiceProviderInterface>,
    pub system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    pub http_client: Arc<dyn HttpClientProvider>,
    pub search_service: Option<Arc<SearchService>>,
}

impl TestInfrastructure {
    /// Create a new test infrastructure with real services
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create test configuration
        let config = Self::create_test_config();
        let config_arc = Arc::new(ArcSwap::from_pointee(config));

        // Create shared components
        let event_bus: SharedEventBusProvider = Arc::new(EventBus::with_default_capacity());
        let log_buffer =
            mcp_context_browser::infrastructure::logging::create_shared_log_buffer(1000);

        // Create service components
        let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
            Arc::new(McpPerformanceMetrics::default());
        let indexing_operations: Arc<dyn IndexingOperationsInterface> =
            Arc::new(McpIndexingOperations::default());
        let service_provider: Arc<dyn ServiceProviderInterface> = Arc::new(ServiceProvider::new());
        let system_collector: Arc<dyn SystemMetricsCollectorInterface> =
            Arc::new(SystemMetricsCollector::new());

        // Create HTTP client
        let http_client: Arc<dyn HttpClientProvider> = Arc::new(
            HttpClientPool::new()
                .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?,
        );

        // Create admin service with all dependencies
        let deps = AdminServiceDependencies {
            performance_metrics: Arc::clone(&performance_metrics),
            indexing_operations: Arc::clone(&indexing_operations),
            service_provider: Arc::clone(&service_provider),
            system_collector: Arc::clone(&system_collector),
            http_client: Arc::clone(&http_client),
            event_bus: event_bus.clone(),
            log_buffer: log_buffer.clone(),
            config: Arc::clone(&config_arc),
        };
        let admin_service = Arc::new(AdminServiceImpl::new(deps)) as Arc<dyn AdminService>;

        Ok(Self {
            admin_service,
            config: config_arc,
            event_bus,
            log_buffer,
            performance_metrics,
            indexing_operations,
            service_provider,
            system_collector,
            http_client,
            search_service: None,
        })
    }

    /// Create a test configuration
    fn create_test_config() -> Config {
        use mcp_context_browser::domain::types::{EmbeddingConfig, VectorStoreConfig};
        use mcp_context_browser::infrastructure::config::{DataConfig, ProviderConfig};

        Config {
            name: "Test MCP Context Browser".to_string(),
            version: "0.1.0-test".to_string(),
            server: Default::default(),
            providers: ProviderConfig {
                embedding: EmbeddingConfig {
                    provider: "null".to_string(),
                    model: "test-model".to_string(),
                    api_key: None,
                    base_url: None,
                    dimensions: Some(384),
                    max_tokens: Some(512),
                },
                vector_store: VectorStoreConfig {
                    provider: "in_memory".to_string(),
                    address: None,
                    token: None,
                    collection: Some("test-collection".to_string()),
                    dimensions: Some(384),
                    timeout_secs: Some(10),
                },
            },
            metrics: Default::default(),
            admin: Default::default(),
            auth: Default::default(),
            database: Default::default(),
            sync: Default::default(),
            daemon: Default::default(),
            resource_limits: Default::default(),
            cache: Default::default(),
            hybrid_search: Default::default(),
            data: DataConfig {
                base_dir: "~/.local/share/mcp-context-browser".to_string(),
                snapshots_dir: None,
                config_history_dir: None,
                encryption_keys_dir: None,
                circuit_breakers_dir: None,
            },
        }
    }

    /// Set up a basic search service for testing
    pub async fn setup_search_service(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use mcp_context_browser::application::context::ContextService;
        use mcp_context_browser::domain::types::{EmbeddingConfig, VectorStoreConfig};

        // Create providers for testing
        let embedding_config = EmbeddingConfig {
            provider: "null".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: Some(512),
        };

        let embedding_provider = self
            .service_provider
            .get_embedding_provider(&embedding_config, Arc::clone(&self.http_client))
            .await?;

        let vector_store_config = VectorStoreConfig {
            provider: "in_memory".to_string(),
            address: None,
            token: None,
            collection: Some("test-search".to_string()),
            dimensions: Some(384),
            timeout_secs: Some(10),
        };

        let vector_store_provider = self
            .service_provider
            .get_vector_store_provider(&vector_store_config)
            .await?;

        // Create context service
        let context_service = Arc::new(ContextService::new_with_providers(
            embedding_provider,
            vector_store_provider,
        ));

        // Create search service
        let search_service = Arc::new(SearchService::new(context_service));

        self.search_service = Some(search_service.clone());

        Ok(())
    }
}

impl Default for TestInfrastructure {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::new())
            .unwrap()
    }
}
