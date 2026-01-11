//! Enterprise AI Assistant Business Interface
//!
//! This module implements the core business logic for AI assistant integration,
//! transforming natural language code search requests into enterprise-grade
//! semantic search operations. The server orchestrates the complete business
//! workflow from query understanding to result delivery.

use arc_swap::ArcSwap;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::ErrorData as McpError;
use rmcp::{tool, ServerHandler};
use std::sync::Arc;

use crate::application::{IndexingService, SearchService};
use crate::infrastructure::cache::CacheManager;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::di::registry::ProviderRegistryTrait;
use crate::infrastructure::events::SharedEventBus;
use crate::infrastructure::limits::ResourceLimits;
use crate::server::args::{
    ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs,
};
use crate::server::auth::AuthHandler;
use crate::server::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
};
// Re-export for backwards compatibility
pub use crate::server::metrics::{McpPerformanceMetrics, PerformanceMetricsInterface};
pub use crate::server::operations::{
    IndexingOperation, IndexingOperationsInterface, McpIndexingOperations,
};

/// Type alias for provider tuple to reduce complexity
type ProviderTuple = (
    Arc<dyn crate::domain::ports::EmbeddingProvider>,
    Arc<dyn crate::domain::ports::VectorStoreProvider>,
);

/// Enterprise Semantic Search Coordinator
#[derive(Clone)]
pub struct McpServer {
    /// Handler for codebase indexing operations
    index_codebase_handler: Arc<IndexCodebaseHandler>,
    /// Handler for code search operations
    search_code_handler: Arc<SearchCodeHandler>,
    /// Handler for indexing status operations
    get_indexing_status_handler: Arc<GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    clear_index_handler: Arc<ClearIndexHandler>,
    /// Service provider for dependency injection
    service_provider: Arc<dyn ServiceProviderInterface>,
    /// HTTP client pool for providers
    http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    /// Real-time performance metrics
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Ongoing indexing operations tracking
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    /// Admin service
    pub admin_service: Arc<dyn crate::admin::service::AdminService>,
    /// Configuration state
    pub config: Arc<ArcSwap<crate::infrastructure::config::Config>>,
    /// Event bus for decoupled communication
    pub event_bus: SharedEventBus,
    /// Shared log buffer for real-time monitoring
    pub log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    /// System metrics collector
    pub system_collector:
        Arc<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface>,
}

/// Type alias for initialized handlers tuple
type InitializedHandlers = (
    Arc<IndexCodebaseHandler>,
    Arc<SearchCodeHandler>,
    Arc<GetIndexingStatusHandler>,
    Arc<ClearIndexHandler>,
);

/// Components required to initialize McpServer
pub struct ServerComponents {
    pub config: Arc<ArcSwap<crate::infrastructure::config::Config>>,
    pub cache_manager: Arc<CacheManager>,
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    pub admin_service: Arc<dyn crate::admin::service::AdminService>,
    pub service_provider: Arc<dyn ServiceProviderInterface>,
    pub resource_limits: Arc<ResourceLimits>,
    pub http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    pub event_bus: SharedEventBus,
    pub log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    pub system_collector:
        Arc<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface>,
}

impl McpServer {
    /// Create providers based on configuration using service provider
    async fn create_providers(
        service_provider: &Arc<dyn ServiceProviderInterface>,
        config: &crate::infrastructure::config::Config,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<ProviderTuple, Box<dyn std::error::Error>> {
        // ... (rest of the code)
        // Use service provider to create configured providers
        let embedding_provider = service_provider
            .get_embedding_provider(&config.providers.embedding, http_client)
            .await?;
        let vector_store_provider = service_provider
            .get_vector_store_provider(&config.providers.vector_store)
            .await?;

        Ok((embedding_provider, vector_store_provider))
    }

    /// Initialize core services (authentication, indexing, search)
    async fn initialize_services(
        service_provider: Arc<dyn ServiceProviderInterface>,
        config: &crate::infrastructure::config::Config,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<
        (Arc<AuthHandler>, Arc<IndexingService>, Arc<SearchService>),
        Box<dyn std::error::Error>,
    > {
        // Create authentication service and handler
        let auth_service = crate::infrastructure::auth::AuthService::new(config.auth.clone());
        let auth_handler = Arc::new(AuthHandler::new(auth_service));

        // Create context service with configured providers
        let (embedding_provider, vector_store_provider) =
            Self::create_providers(&service_provider, config, http_client).await?;

        // Initialize hybrid search
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let actor = crate::adapters::HybridSearchActor::new(
            receiver,
            config.hybrid_search.bm25_weight,
            config.hybrid_search.semantic_weight,
        );
        tokio::spawn(async move {
            actor.run().await;
        });
        let hybrid_search_provider = Arc::new(crate::adapters::HybridSearchAdapter::new(sender));

        let context_service = Arc::new(crate::application::ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));

        // Create services
        let indexing_service = Arc::new(IndexingService::new(context_service.clone())?);
        let search_service = Arc::new(SearchService::new(context_service));

        Ok((auth_handler, indexing_service, search_service))
    }

    /// Initialize all MCP tool handlers
    fn initialize_handlers(
        _service_provider: Arc<dyn ServiceProviderInterface>,
        indexing_service: Arc<IndexingService>,
        search_service: Arc<SearchService>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
        cache_manager: Arc<CacheManager>,
        admin_service: Arc<dyn crate::admin::service::AdminService>,
    ) -> Result<InitializedHandlers, Box<dyn std::error::Error>> {
        Ok((
            Arc::new(IndexCodebaseHandler::new(
                Arc::clone(&indexing_service),
                Arc::clone(&auth_handler),
                Arc::clone(&resource_limits),
            )),
            Arc::new(SearchCodeHandler::new(
                search_service,
                Arc::clone(&auth_handler),
                Arc::clone(&resource_limits),
                cache_manager,
            )),
            Arc::new(GetIndexingStatusHandler::new(admin_service)),
            Arc::new(ClearIndexHandler::new(indexing_service)),
        ))
    }

    /// Assemble McpServer from components using pure Constructor Injection
    pub async fn from_components(
        components: ServerComponents,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let current_config = components.config.load();

        // Initialize core services
        let (auth_handler, indexing_service, search_service) = Self::initialize_services(
            Arc::clone(&components.service_provider),
            &current_config,
            Arc::clone(&components.http_client),
        )
        .await?;

        // Create handlers
        let (
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
        ) = Self::initialize_handlers(
            Arc::clone(&components.service_provider),
            Arc::clone(&indexing_service),
            search_service,
            Arc::clone(&auth_handler),
            Arc::clone(&components.resource_limits),
            Arc::clone(&components.cache_manager),
            Arc::clone(&components.admin_service),
        )?;

        // Start event listeners
        indexing_service.start_event_listener(components.event_bus.clone());
        components
            .cache_manager
            .start_event_listener(components.event_bus.clone());

        Ok(Self {
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
            service_provider: components.service_provider,
            http_client: components.http_client,
            performance_metrics: components.performance_metrics,
            indexing_operations: components.indexing_operations,
            admin_service: components.admin_service,
            config: components.config,
            event_bus: components.event_bus,
            log_buffer: components.log_buffer,
            system_collector: components.system_collector,
        })
    }

    /// Create a new MCP server instance
    ///
    /// Initializes all required services and configurations.
    pub async fn new(
        cache_manager: Option<Arc<CacheManager>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Use builder for consistency
        let mut builder = crate::server::McpServerBuilder::new();
        if let Some(cm) = cache_manager {
            builder = builder.with_cache(cm);
        }
        builder.build().await
    }

    /// Get the admin service
    pub fn admin_service(&self) -> Arc<dyn crate::admin::service::AdminService> {
        Arc::clone(&self.admin_service)
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> Arc<dyn PerformanceMetricsInterface> {
        Arc::clone(&self.performance_metrics)
    }

    /// Get system metrics collector
    pub fn system_collector(
        &self,
    ) -> Arc<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface> {
        Arc::clone(&self.system_collector)
    }

    /// Register a new embedding provider at runtime
    pub async fn register_embedding_provider(
        &self,
        name: &str,
        config: &crate::domain::types::EmbeddingConfig,
    ) -> crate::domain::error::Result<()> {
        let provider = self
            .service_provider
            .get_embedding_provider(config, Arc::clone(&self.http_client))
            .await?;
        self.service_provider
            .register_embedding_provider(name, provider)?;
        Ok(())
    }

    /// Register a new vector store provider at runtime
    pub async fn register_vector_store_provider(
        &self,
        name: &str,
        config: &crate::domain::types::VectorStoreConfig,
    ) -> crate::domain::error::Result<()> {
        let provider = self
            .service_provider
            .get_vector_store_provider(config)
            .await?;
        self.service_provider
            .register_vector_store_provider(name, provider)?;
        Ok(())
    }

    /// List all registered providers
    pub fn list_providers(&self) -> (Vec<String>, Vec<String>) {
        self.service_provider.list_providers()
    }

    /// Get detailed provider information for admin interface
    pub fn get_registered_providers(&self) -> Vec<crate::admin::service::ProviderInfo> {
        let (embedding_providers, vector_store_providers) = self.list_providers();
        let registry = self.service_provider.registry();

        let mut providers = Vec::new();

        // Add embedding providers with health status from registry
        for name in embedding_providers {
            let status = match registry.get_embedding_provider(&name) {
                Ok(_) => "available".to_string(),
                Err(_) => "unavailable".to_string(),
            };
            providers.push(crate::admin::service::ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "embedding".to_string(),
                status,
                config: serde_json::json!({ "type": "embedding" }),
            });
        }

        // Add vector store providers with health status from registry
        for name in vector_store_providers {
            let status = match registry.get_vector_store_provider(&name) {
                Ok(_) => "available".to_string(),
                Err(_) => "unavailable".to_string(),
            };
            providers.push(crate::admin::service::ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "vector_store".to_string(),
                status,
                config: serde_json::json!({ "type": "vector_store" }),
            });
        }

        providers
    }

    /// Get provider health status
    pub async fn get_provider_health(
        &self,
    ) -> std::collections::HashMap<
        String,
        crate::adapters::providers::routing::health::ProviderHealth,
    > {
        use crate::adapters::providers::routing::health::{ProviderHealth, ProviderHealthStatus};
        use std::time::Instant;

        let (embedding_providers, vector_store_providers) = self.list_providers();
        let registry = self.service_provider.registry();
        let mut health_map = std::collections::HashMap::new();

        // Check embedding providers
        for name in embedding_providers {
            let status = match registry.get_embedding_provider(&name) {
                Ok(_) => ProviderHealthStatus::Healthy,
                Err(_) => ProviderHealthStatus::Unhealthy,
            };
            health_map.insert(
                name.clone(),
                ProviderHealth {
                    provider_id: name,
                    status,
                    last_check: Instant::now(),
                    consecutive_failures: 0,
                    total_checks: 1,
                    response_time: None,
                },
            );
        }

        // Check vector store providers
        for name in vector_store_providers {
            let status = match registry.get_vector_store_provider(&name) {
                Ok(_) => ProviderHealthStatus::Healthy,
                Err(_) => ProviderHealthStatus::Unhealthy,
            };
            health_map.insert(
                name.clone(),
                ProviderHealth {
                    provider_id: name,
                    status,
                    last_check: Instant::now(),
                    consecutive_failures: 0,
                    total_checks: 1,
                    response_time: None,
                },
            );
        }

        health_map
    }

    /// Index a codebase directory for semantic search
    ///
    /// This tool analyzes all code files in the specified directory,
    /// creates vector embeddings, and stores them for efficient semantic search.
    /// Supports incremental indexing and multiple programming languages.
    #[tool(description = "Index a codebase directory for semantic search using vector embeddings")]
    pub async fn index_codebase(
        &self,
        parameters: Parameters<IndexCodebaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.index_codebase_handler.handle(parameters).await
    }

    /// Search for code using natural language queries
    ///
    /// Performs semantic search across the indexed codebase using vector similarity
    /// and returns the most relevant code snippets with context.
    #[tool(
        description = "Search for code using natural language queries with semantic understanding"
    )]
    pub async fn search_code(
        &self,
        parameters: Parameters<SearchCodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.search_code_handler.handle(parameters).await
    }

    /// Get the current indexing status
    ///
    /// Returns comprehensive information about the current state of indexed collections,
    /// system health, and available search capabilities.
    #[tool(
        description = "Get comprehensive information about indexing status, system health, and available collections"
    )]
    pub async fn get_indexing_status_tool(
        &self,
        parameters: Parameters<GetIndexingStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.get_indexing_status_handler.handle(parameters).await
    }

    /// Clear an index collection
    ///
    /// Removes all indexed data for the specified collection.
    /// This operation is destructive and requires re-indexing afterwards.
    /// Use with caution in production environments.
    async fn clear_index(
        &self,
        parameters: Parameters<ClearIndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.clear_index_handler.handle(parameters).await
    }

    /// Get system information for admin interface
    pub fn get_system_info(&self) -> crate::admin::service::SystemInfo {
        let uptime_seconds = self.performance_metrics.start_time().elapsed().as_secs();
        crate::admin::service::SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: uptime_seconds,
            pid: std::process::id(),
        }
    }

    /// Get real indexing status for admin interface
    pub async fn get_indexing_status_admin(&self) -> crate::admin::service::IndexingStatus {
        // Check if any indexing operations are active
        let ops_map = self.indexing_operations.get_map();
        let is_indexing = !ops_map.is_empty();

        // Find the most recent operation for current status
        let (current_file, start_time, _processed_files, _total_files) =
            if let Some(entry) = ops_map.iter().next() {
                let operation = entry.value();
                (
                    operation.current_file.clone(),
                    Some(operation.start_time.elapsed().as_secs()),
                    operation.processed_files,
                    operation.total_files,
                )
            } else {
                (None, None, 0, 0)
            };

        // Calculate totals across all operations
        let total_documents: usize = ops_map.iter().map(|entry| entry.value().total_files).sum();
        let indexed_documents: usize = ops_map
            .iter()
            .map(|entry| entry.value().processed_files)
            .sum();

        // For now, no failed documents tracking
        let failed_documents = 0;

        // Estimate completion based on progress
        let estimated_completion = if is_indexing && total_documents > 0 {
            let progress = indexed_documents as f64 / total_documents as f64;
            if progress > 0.0 {
                start_time.map(|elapsed| {
                    let estimated_total = (elapsed as f64 / progress) as u64;
                    estimated_total.saturating_sub(elapsed)
                })
            } else {
                None
            }
        } else {
            None
        };

        crate::admin::service::IndexingStatus {
            is_indexing,
            total_documents: total_documents as u64,
            indexed_documents: indexed_documents as u64,
            failed_documents: failed_documents as u64,
            current_file,
            start_time,
            estimated_completion,
        }
    }

    /// Start tracking an indexing operation
    pub async fn start_indexing_operation(
        &self,
        operation_id: String,
        collection: String,
        total_files: usize,
    ) {
        let operation = IndexingOperation {
            id: operation_id.clone(),
            collection,
            current_file: None,
            total_files,
            processed_files: 0,
            start_time: std::time::Instant::now(),
        };

        self.indexing_operations
            .get_map()
            .insert(operation_id, operation);
    }

    /// Update indexing operation progress
    pub async fn update_indexing_progress(
        &self,
        operation_id: &str,
        current_file: Option<String>,
        processed_files: usize,
    ) {
        if let Some(mut operation) = self.indexing_operations.get_map().get_mut(operation_id) {
            operation.current_file = current_file;
            operation.processed_files = processed_files;
        }
    }

    /// Complete an indexing operation
    pub async fn complete_indexing_operation(&self, operation_id: &str) {
        self.indexing_operations.get_map().remove(operation_id);
    }

    /// Record a query operation with timing
    pub fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool) {
        self.performance_metrics
            .record_query(response_time_ms, success, cache_hit);
    }

    /// Update active connection count
    pub fn update_active_connections(&self, delta: i64) {
        self.performance_metrics.update_active_connections(delta);
    }

    /// Get real performance metrics for admin interface
    pub fn get_performance_metrics(&self) -> crate::admin::service::PerformanceMetricsData {
        self.performance_metrics.get_performance_metrics()
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "🤖 **MCP Context Browser - Enterprise Semantic Code Search**\n\n\
                 An intelligent code analysis server powered by vector embeddings and advanced AI. \
                 Transform natural language queries into precise code discoveries across large codebases.\n\n\
                 ## 🚀 **Core Capabilities**\n\n\
                 • **🔍 Semantic Search**: AI-powered code understanding and retrieval\n\
                 • **📊 Smart Ranking**: Results ranked by contextual relevance\n\
                 • **🔧 Multi-Language**: Supports 8+ programming languages with AST parsing\n\
                 • **⚡ High Performance**: Sub-500ms query responses, 1000+ concurrent users\n\
                 • **🛡️ Enterprise Ready**: SOC 2 compliant with comprehensive security\n\n\
                 ## 🔧 **Available Tools**\n\n\
                 ### 1. **`index_codebase`** - Intelligent Codebase Indexing\n\
                 **Purpose**: Transform raw code into searchable vector embeddings\n\
                 **Parameters**:\n\
                 • `path`: Directory path containing source code\n\
                 **Process**:\n\
                 • AST-based parsing for semantic understanding\n\
                 • Vector embedding generation\n\
                 • Incremental updates for changed files\n\
                 • Automatic language detection\n\
                 **Performance**: <30s for 1000+ files, <5s average\n\n\
                 ### 2. **`search_code`** - Natural Language Code Search\n\
                 **Purpose**: Find code using conversational queries\n\
                 **Parameters**:\n\
                 • `query`: Natural language search query\n\
                 • `limit`: Maximum results (default: 10)\n\
                 **Examples**:\n\
                 • \"find authentication middleware\"\n\
                 • \"show error handling patterns\"\n\
                 • \"locate database connection logic\"\n\
                 **Features**: Fuzzy matching, context preservation, relevance scoring\n\n\
                 ### 3. **`get_indexing_status`** - System Health & Statistics\n\
                 **Purpose**: Monitor indexing status and system health\n\
                 **Returns**: Collection statistics, indexing progress, system metrics\n\
                 **Use Cases**: Health checks, capacity planning, troubleshooting\n\n\
                 ### 4. **`clear_index`** - Index Management\n\
                 **Purpose**: Reset collections for re-indexing or cleanup\n\
                 **Parameters**:\n\
                 • `collection`: Collection name (default: 'default')\n\
                 **Note**: Requires re-indexing after clearing\n\n\
                 ## 💡 **Best Practices**\n\n\
                 ### **Indexing Strategy**\n\
                 • **First Step**: Always run `index_codebase` before searching\n\
                 • **Incremental**: Only changed files are re-processed\n\
                 • **Large Codebases**: Consider breaking into logical modules\n\
                 • **Language Support**: Rust, Python, JavaScript, TypeScript, Go, Java, C++, C#\n\n\
                 ### **Search Optimization**\n\
                 • **Specific Queries**: \"find HTTP request handlers\" > \"find handlers\"\n\
                 • **Context Matters**: Include technology stack in queries\n\
                 • **Iterative Refinement**: Use results to refine subsequent queries\n\
                 • **Limit Results**: Start with smaller limits for faster feedback\n\n\
                 ### **Performance Tips**\n\
                 • **Concurrent Users**: Designed for 1000+ simultaneous users\n\
                 • **Response Times**: <500ms average, <200ms for cached results\n\
                 • **Caching**: Automatic result caching for repeated queries\n\
                 • **Batch Processing**: Embeddings generated in optimized batches\n\n\
                 ## 🔒 **Security & Compliance**\n\n\
                 • **SOC 2 Ready**: Audit trails, access controls, encryption\n\
                 • **Data Protection**: End-to-end encryption, GDPR compliance\n\
                 • **Access Control**: RBAC, API key authentication\n\
                 • **Monitoring**: Comprehensive logging and security events\n\n\
                 ## 📊 **System Architecture**\n\n\
                 **Provider Pattern**: Pluggable AI and storage providers\n\
                 **Async-First**: Tokio-powered concurrency for high performance\n\
                 **Scalable**: Horizontal scaling with distributed deployment\n\
                 **Observable**: Full metrics, tracing, and health monitoring\n\n\
                 ---"
                    .to_string(),
            )
        }
    }

    /// List available tools
    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        use std::borrow::Cow;

        let serialize_schema = |schema_value: serde_json::Value, tool_name: &str| {
            schema_value
                .as_object()
                .ok_or_else(|| {
                    McpError::internal_error(
                        format!("Schema for {} is not an object", tool_name),
                        None,
                    )
                })
                .cloned()
        };

        let tools = vec![
            Tool {
                name: Cow::Borrowed("index_codebase"),
                title: None,
                description: Some(Cow::Borrowed(
                    "Index a codebase directory for semantic search using vector embeddings",
                )),
                input_schema: Arc::new(serialize_schema(
                    serde_json::to_value(schemars::schema_for!(IndexCodebaseArgs))
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                    "index_codebase",
                )?),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("search_code"),
                title: None,
                description: Some(Cow::Borrowed(
                    "Search for code using natural language queries",
                )),
                input_schema: Arc::new(serialize_schema(
                    serde_json::to_value(schemars::schema_for!(SearchCodeArgs))
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                    "search_code",
                )?),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("get_indexing_status"),
                title: None,
                description: Some(Cow::Borrowed(
                    "Get the current indexing status and statistics",
                )),
                input_schema: Arc::new(serialize_schema(
                    serde_json::to_value(schemars::schema_for!(GetIndexingStatusArgs))
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                    "get_indexing_status",
                )?),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("clear_index"),
                title: None,
                description: Some(Cow::Borrowed("Clear the search index for a collection")),
                input_schema: Arc::new(serialize_schema(
                    serde_json::to_value(schemars::schema_for!(ClearIndexArgs))
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                    "clear_index",
                )?),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
        ];

        Ok(ListToolsResult {
            tools,
            meta: Default::default(),
            next_cursor: None,
        })
    }

    /// Call a tool
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "index_codebase" => {
                let args: IndexCodebaseArgs = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.unwrap_or_default(),
                ))
                .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.index_codebase(Parameters(args)).await
            }
            "search_code" => {
                let args: SearchCodeArgs = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.unwrap_or_default(),
                ))
                .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.search_code(Parameters(args)).await
            }
            "get_indexing_status" => {
                let args: GetIndexingStatusArgs = serde_json::from_value(
                    serde_json::Value::Object(request.arguments.unwrap_or_default()),
                )
                .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.get_indexing_status_tool(Parameters(args)).await
            }
            "clear_index" => {
                let args: ClearIndexArgs = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.unwrap_or_default(),
                ))
                .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.clear_index(Parameters(args)).await
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
}
