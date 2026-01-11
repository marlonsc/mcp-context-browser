use crate::infrastructure::cache::CacheManager;
use crate::infrastructure::config::Config;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::events::SharedEventBus;
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::logging::SharedLogBuffer;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::mcp_server::{
    IndexingOperationsInterface, McpServer, PerformanceMetricsInterface,
};
use arc_swap::ArcSwap;
use std::sync::Arc;

/// Builder for McpServer to handle complex dependency injection
#[derive(Default)]
pub struct McpServerBuilder {
    config: Option<Arc<ArcSwap<Config>>>,
    cache_manager: Option<Arc<CacheManager>>,
    event_bus: Option<SharedEventBus>,
    log_buffer: Option<SharedLogBuffer>,
    performance_metrics: Option<Arc<dyn PerformanceMetricsInterface>>,
    indexing_operations: Option<Arc<dyn IndexingOperationsInterface>>,
    service_provider: Option<Arc<dyn ServiceProviderInterface>>,
    system_collector: Option<Arc<dyn SystemMetricsCollectorInterface>>,
    resource_limits: Option<Arc<ResourceLimits>>,
    http_client: Option<Arc<dyn crate::adapters::http_client::HttpClientProvider>>,
}

impl McpServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Arc<ArcSwap<Config>>) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_cache(mut self, cache_manager: Arc<CacheManager>) -> Self {
        self.cache_manager = Some(cache_manager);
        self
    }

    pub fn with_event_bus(mut self, event_bus: SharedEventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn with_log_buffer(mut self, log_buffer: SharedLogBuffer) -> Self {
        self.log_buffer = Some(log_buffer);
        self
    }

    pub fn with_performance_metrics(
        mut self,
        metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        self.performance_metrics = Some(metrics);
        self
    }

    pub fn with_indexing_operations(mut self, ops: Arc<dyn IndexingOperationsInterface>) -> Self {
        self.indexing_operations = Some(ops);
        self
    }

    pub fn with_service_provider(mut self, provider: Arc<dyn ServiceProviderInterface>) -> Self {
        self.service_provider = Some(provider);
        self
    }

    pub fn with_system_collector(
        mut self,
        collector: Arc<dyn SystemMetricsCollectorInterface>,
    ) -> Self {
        self.system_collector = Some(collector);
        self
    }

    pub fn with_resource_limits(mut self, limits: Arc<ResourceLimits>) -> Self {
        self.resource_limits = Some(limits);
        self
    }

    pub fn with_http_client(
        mut self,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Self {
        self.http_client = Some(http_client);
        self
    }

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

        let event_bus = self
            .event_bus
            .unwrap_or_else(|| Arc::new(crate::infrastructure::events::EventBus::default()));
        let log_buffer = self
            .log_buffer
            .unwrap_or_else(|| crate::infrastructure::logging::create_shared_log_buffer(1000));

        let performance_metrics = self.performance_metrics.unwrap_or_else(|| {
            Arc::new(crate::server::mcp_server::McpPerformanceMetrics::default())
        });

        let indexing_operations = self.indexing_operations.unwrap_or_else(|| {
            Arc::new(crate::server::mcp_server::McpIndexingOperations::default())
        });

        let service_provider = self.service_provider.unwrap_or_else(|| {
            Arc::new(crate::infrastructure::di::factory::ServiceProvider::new())
        });

        let system_collector = self.system_collector.unwrap_or_else(|| {
            Arc::new(crate::infrastructure::metrics::system::SystemMetricsCollector::new())
        });

        // Initialize resource limits from config if not provided
        let resource_limits = if let Some(rl) = self.resource_limits {
            rl
        } else {
            let config = config_arc.load();
            Arc::new(ResourceLimits::new(config.resource_limits.clone()))
        };

        // Initialize HTTP client if not provided
        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                Arc::new(
                    crate::adapters::http_client::HttpClientPool::new()
                        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client pool: {}", e))?,
                )
            }
        };

        // Initialize cache manager if not provided
        let cache_manager = match self.cache_manager {
            Some(cm) => cm,
            None => {
                let config = config_arc.load().cache.clone();
                Arc::new(CacheManager::new(config, Some(event_bus.clone())).await?)
            }
        };

        // Create admin service with all dependencies
        let admin_service = Arc::new(crate::admin::service::AdminServiceImpl::new(
            Arc::clone(&performance_metrics),
            Arc::clone(&indexing_operations),
            Arc::clone(&service_provider),
            Arc::clone(&system_collector),
            event_bus.clone(),
            log_buffer.clone(),
            Arc::clone(&config_arc),
        )) as Arc<dyn crate::admin::service::AdminService>;

        // Use from_components to assemble the server
        McpServer::from_components(crate::server::mcp_server::ServerComponents {
            config: config_arc,
            cache_manager,
            performance_metrics,
            indexing_operations,
            admin_service,
            service_provider,
            resource_limits,
            http_client,
            event_bus,
            log_buffer,
            system_collector,
        })
        .await
    }
}
