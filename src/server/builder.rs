//! # MCP Server Builder
//!
//! Fluent builder pattern for configuring and constructing the MCP server.
//! Provides a type-safe way to configure all server components before startup.

use crate::application::admin::traits::AdminService;
use crate::domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use crate::infrastructure::cache::{create_cache_provider, SharedCacheProvider};
use crate::infrastructure::config::Config;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::di::DiContainer;
use crate::infrastructure::events::SharedEventBusProvider;
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::logging::SharedLogBuffer;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::mcp_server::McpServer;
use arc_swap::ArcSwap;
use std::sync::Arc;

/// Builder for McpServer to handle complex dependency injection
#[derive(Default)]
pub struct McpServerBuilder {
    config: Option<Arc<ArcSwap<Config>>>,
    cache_provider: Option<SharedCacheProvider>,
    event_bus: Option<SharedEventBusProvider>,
    log_buffer: Option<SharedLogBuffer>,
    performance_metrics: Option<Arc<dyn PerformanceMetricsInterface>>,
    indexing_operations: Option<Arc<dyn IndexingOperationsInterface>>,
    service_provider: Option<Arc<dyn ServiceProviderInterface>>,
    system_collector: Option<Arc<dyn SystemMetricsCollectorInterface>>,
    resource_limits: Option<Arc<ResourceLimits>>,
    http_client: Option<Arc<dyn crate::adapters::http_client::HttpClientProvider>>,
}

impl McpServerBuilder {
    /// Create a new MCP server builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration for the MCP server
    pub fn with_config(mut self, config: Arc<ArcSwap<Config>>) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the cache provider for the MCP server
    pub fn with_cache_provider(mut self, cache_provider: Option<SharedCacheProvider>) -> Self {
        self.cache_provider = cache_provider;
        self
    }

    /// Set the event bus for system-wide communication
    pub fn with_event_bus(mut self, event_bus: SharedEventBusProvider) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Set the log buffer for structured logging
    pub fn with_log_buffer(mut self, log_buffer: SharedLogBuffer) -> Self {
        self.log_buffer = Some(log_buffer);
        self
    }

    /// Set the performance metrics collector
    pub fn with_performance_metrics(
        mut self,
        metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        self.performance_metrics = Some(metrics);
        self
    }

    /// Set the indexing operations tracker
    pub fn with_indexing_operations(mut self, ops: Arc<dyn IndexingOperationsInterface>) -> Self {
        self.indexing_operations = Some(ops);
        self
    }

    /// Set the service provider for dependency injection
    pub fn with_service_provider(mut self, provider: Arc<dyn ServiceProviderInterface>) -> Self {
        self.service_provider = Some(provider);
        self
    }

    /// Set the system metrics collector
    pub fn with_system_collector(
        mut self,
        collector: Arc<dyn SystemMetricsCollectorInterface>,
    ) -> Self {
        self.system_collector = Some(collector);
        self
    }

    /// Set resource limits for the server
    /// Set the resource limits for the MCP server
    pub fn with_resource_limits(mut self, limits: Arc<ResourceLimits>) -> Self {
        self.resource_limits = Some(limits);
        self
    }

    /// Set the HTTP client provider
    pub fn with_http_client(
        mut self,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Build the MCP server with the configured components
    pub async fn build(self) -> Result<McpServer, Box<dyn std::error::Error>> {
        // Load configuration if not provided
        let config_arc = if let Some(c) = self.config {
            c
        } else {
            let loader = crate::infrastructure::config::ConfigLoader::new();
            let home_dir = dirs::home_dir().ok_or("Cannot determine home directory")?;
            let config_path = home_dir.join(".context").join("config.toml");
            let config = loader.load_with_file(&config_path).await?;
            Arc::new(ArcSwap::from_pointee(config))
        };

        // Check if we're in stdio-only mode (used for testing)
        let is_stdio_only = std::env::var("MCP__TRANSPORT__MODE")
            .map(|s| s.to_lowercase() == "stdio")
            .unwrap_or(false);

        // Resolve HTTP client first (needed for build_with_config)
        // Use a temporary container to get the default HTTP client if not provided
        let temp_container = DiContainer::build()
            .map_err(|e| anyhow::anyhow!("Failed to build temp DI container: {}", e))?;
        let http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider> =
            self.http_client.unwrap_or_else(|| temp_container.resolve());

        // Build DI container
        let container = if is_stdio_only {
            // For stdio-only mode (tests), use null providers to avoid external dependencies
            tracing::info!("📡 Stdio-only mode detected in builder, using null providers for testing");
            DiContainer::build()
                .map_err(|e| anyhow::anyhow!("Failed to build DI container: {}", e))?
        } else {
            // For production/server mode, build with config-based providers
            // This uses actual providers based on configuration instead of null providers
            let current_config = config_arc.load();
            DiContainer::build_with_config(&current_config, Arc::clone(&http_client))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to build DI container with config: {}", e))?
        };

        // Resolve EventBus from DI container if not explicitly provided
        let event_bus: crate::infrastructure::events::SharedEventBusProvider =
            self.event_bus.unwrap_or_else(|| container.resolve());
        let log_buffer = self
            .log_buffer
            .unwrap_or_else(|| crate::infrastructure::logging::create_shared_log_buffer(1000));

        // Resolve from DI container if not explicitly provided
        let performance_metrics: Arc<dyn PerformanceMetricsInterface> = self
            .performance_metrics
            .unwrap_or_else(|| container.resolve());

        let indexing_operations: Arc<dyn IndexingOperationsInterface> = self
            .indexing_operations
            .unwrap_or_else(|| container.resolve());

        let service_provider: Arc<dyn ServiceProviderInterface> =
            self.service_provider.unwrap_or_else(|| container.resolve());

        let system_collector: Arc<dyn SystemMetricsCollectorInterface> =
            self.system_collector.unwrap_or_else(|| container.resolve());

        // Initialize resource limits from config if not provided
        // ResourceLimits not yet in DI modules
        let resource_limits = if let Some(rl) = self.resource_limits {
            rl
        } else {
            let config = config_arc.load();
            Arc::new(ResourceLimits::new(config.resource_limits.clone()))
        };

        // HTTP client was already resolved above for build_with_config
        // No need to resolve again

        // Initialize cache provider if not provided
        let cache_provider = match self.cache_provider {
            Some(cp) => cp,
            None => {
                let config = config_arc.load().cache.clone();
                create_cache_provider(&config).await?
            }
        };

        // Resolve admin service from DI container
        // AdminService is now fully wired through Shaku DI with proper submodule dependencies
        let admin_service: Arc<dyn AdminService> = container.resolve();

        // Resolve application services from DI container
        let indexing_service: Arc<dyn crate::domain::ports::IndexingServiceInterface> =
            container.resolve();
        let search_service: Arc<dyn crate::domain::ports::SearchServiceInterface> =
            container.resolve();

        // Use from_components to assemble the server with DI-resolved services
        McpServer::from_components(crate::server::ServerComponents {
            config: config_arc,
            cache_provider: Some(cache_provider),
            performance_metrics,
            indexing_operations,
            admin_service,
            service_provider,
            resource_limits,
            http_client,
            event_bus,
            log_buffer,
            system_collector,
            indexing_service: Some(indexing_service),
            search_service: Some(search_service),
        })
        .await
    }
}
