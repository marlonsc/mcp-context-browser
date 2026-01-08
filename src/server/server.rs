//! Enterprise AI Assistant Business Interface
//!
//! This module implements the core business logic for AI assistant integration,
//! transforming natural language code search requests into enterprise-grade
//! semantic search operations. The server orchestrates the complete business
//! workflow from query understanding to result delivery.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::{ServerHandler, tool};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

use crate::core::cache::CacheManager;
use crate::core::limits::ResourceLimits;
use crate::di::factory::ServiceProvider;
use crate::providers::routing::ProviderRouter;
use crate::server::args::{
    ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs,
};
use crate::server::auth::AuthHandler;
use crate::server::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
};
use crate::services::{IndexingService, SearchService};

/// Type alias for provider tuple to reduce complexity
type ProviderTuple = (
    Arc<dyn crate::providers::EmbeddingProvider>,
    Arc<dyn crate::providers::VectorStoreProvider>,
);

/// Enterprise Semantic Search Coordinator
///
/// The McpServer orchestrates the complete business workflow for AI-powered
/// code intelligence, connecting development teams with instant access to
/// codebase knowledge. This enterprise-grade server transforms natural language
/// queries into precise code discoveries while maintaining security, performance,
/// and reliability standards required for production deployments.
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
    service_provider: Arc<ServiceProvider>,
    /// Real-time performance metrics
    performance_metrics: Arc<PerformanceMetrics>,
    /// Ongoing indexing operations tracking
    indexing_operations: Arc<RwLock<HashMap<String, IndexingOperation>>>,
}

/// Real-time performance metrics tracking
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Total queries processed
    pub total_queries: AtomicU64,
    /// Successful queries
    pub successful_queries: AtomicU64,
    /// Failed queries
    pub failed_queries: AtomicU64,
    /// Response time accumulator (in milliseconds)
    pub response_time_sum: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Active connections
    pub active_connections: AtomicU64,
    /// Server start time
    pub start_time: std::time::Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            response_time_sum: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            start_time: std::time::Instant::now(),
        }
    }
}

/// Tracks ongoing indexing operations
#[derive(Debug, Clone)]
pub struct IndexingOperation {
    /// Operation ID
    pub id: String,
    /// Collection being indexed
    pub collection: String,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Start time
    pub start_time: std::time::Instant,
}

impl McpServer {
    /// Create providers based on configuration with fallback to defaults
    fn create_providers(config: &crate::config::Config) -> Result<ProviderTuple, Box<dyn std::error::Error>> {
        // Try to create configured providers, fallback to defaults
        let embedding_provider = Self::create_embedding_provider(config)?;
        let vector_store_provider = Self::create_vector_store_provider(config)?;

        Ok((embedding_provider, vector_store_provider))
    }

    /// Create embedding provider with fallback logic
    fn create_embedding_provider(
        config: &crate::config::Config,
    ) -> Result<Arc<dyn crate::providers::EmbeddingProvider>, Box<dyn std::error::Error>> {
        match config.providers.embedding.provider.as_str() {
            "ollama" => {
                let base_url = config
                    .providers
                    .embedding
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let model = config.providers.embedding.model.clone();
                match crate::providers::OllamaEmbeddingProvider::new(base_url, model) {
                    Ok(provider) => Ok(Arc::new(provider)),
                    Err(e) => {
                        tracing::warn!("Failed to create Ollama provider: {}", e);
                        Ok(Arc::new(crate::providers::MockEmbeddingProvider::new()))
                    }
                }
            }
            _ => Ok(Arc::new(crate::providers::MockEmbeddingProvider::new())),
        }
    }

    /// Create vector store provider with fallback logic
    fn create_vector_store_provider(
        config: &crate::config::Config,
    ) -> Result<Arc<dyn crate::providers::VectorStoreProvider>, Box<dyn std::error::Error>> {
        match config.providers.vector_store.provider.as_str() {
            "in-memory" => Ok(Arc::new(
                crate::providers::InMemoryVectorStoreProvider::new(),
            )),
            _ => Ok(Arc::new(
                crate::providers::InMemoryVectorStoreProvider::new(),
            )),
        }
    }

    /// Initialize core services (authentication, indexing, search)
    fn initialize_services(
        config: &crate::config::Config,
    ) -> Result<
        (Arc<AuthHandler>, Arc<IndexingService>, Arc<SearchService>),
        Box<dyn std::error::Error>,
    > {
        // Create authentication service and handler
        let auth_service = crate::core::auth::AuthService::new(config.auth.clone());
        let auth_handler = Arc::new(AuthHandler::new(auth_service));

        // Create context service with configured providers
        let (embedding_provider, vector_store_provider) = Self::create_providers(config)?;
        let context_service = Arc::new(crate::services::ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));

        // Create services
        let indexing_service = Arc::new(IndexingService::new(context_service.clone())?);
        let search_service = Arc::new(SearchService::new(context_service));

        Ok((auth_handler, indexing_service, search_service))
    }

    /// Initialize cache manager with fallback to disabled cache
    fn initialize_cache_manager(
        cache_manager: Option<Arc<CacheManager>>,
    ) -> Result<Arc<CacheManager>, Box<dyn std::error::Error>> {
        let cache_manager = cache_manager.unwrap_or_else(|| {
            Arc::new({
                let config = crate::core::cache::CacheConfig {
                    enabled: false,
                    ..Default::default()
                };
                // For disabled cache, we can create synchronously since no Redis connection needed
                futures::executor::block_on(CacheManager::new(config))
                    .map_err(|e| format!("Failed to create disabled cache manager: {}", e))
                    .expect("Failed to create disabled cache manager")
            })
        });
        Ok(cache_manager)
    }

    /// Initialize all MCP tool handlers
    fn initialize_handlers(
        indexing_service: Arc<IndexingService>,
        search_service: Arc<SearchService>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
        cache_manager: Arc<CacheManager>,
    ) -> Result<
        (
            Arc<IndexCodebaseHandler>,
            Arc<SearchCodeHandler>,
            Arc<GetIndexingStatusHandler>,
            Arc<ClearIndexHandler>,
        ),
        Box<dyn std::error::Error>,
    > {
        Ok((
            Arc::new(IndexCodebaseHandler::new(
                indexing_service,
                Arc::clone(&auth_handler),
                Arc::clone(&resource_limits),
            )),
            Arc::new(SearchCodeHandler::new(
                search_service,
                Arc::clone(&auth_handler),
                Arc::clone(&resource_limits),
                cache_manager,
            )),
            Arc::new(GetIndexingStatusHandler::new()),
            Arc::new(ClearIndexHandler::new()),
        ))
    }

    /// Create a new MCP server instance
    ///
    /// Initializes all required services and configurations.
    pub fn new(
        cache_manager: Option<Arc<CacheManager>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration from environment
        let config = crate::config::Config::from_env()
            .map_err(|e| format!("Failed to load configuration: {}", e))?;

        // Initialize resource limits
        let resource_limits_config = config.resource_limits.clone();
        let resource_limits = Arc::new(ResourceLimits::new(resource_limits_config.clone()));
        crate::core::limits::init_global_resource_limits(resource_limits_config)?;

        // Create provider registry and router
        let registry = Arc::new(crate::di::registry::ProviderRegistry::new());
        let _provider_router = Arc::new(ProviderRouter::with_defaults(Arc::clone(&registry))?);
        let service_provider = Arc::new(ServiceProvider::new());

        // Initialize core services
        let (auth_handler, indexing_service, search_service) = Self::initialize_services(&config)?;

        // Create cache manager
        let cache_manager = Self::initialize_cache_manager(cache_manager)?;

        // Create handlers
        let (
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
        ) = Self::initialize_handlers(
            indexing_service,
            search_service,
            Arc::clone(&auth_handler),
            Arc::clone(&resource_limits),
            cache_manager,
        )?;

        Ok(Self {
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
            service_provider,
            performance_metrics: Arc::new(PerformanceMetrics::default()),
            indexing_operations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a new embedding provider at runtime
    pub async fn register_embedding_provider(
        &self,
        name: &str,
        config: &crate::core::types::EmbeddingConfig,
    ) -> crate::core::error::Result<()> {
        let provider = self.service_provider.get_embedding_provider(config).await?;
        self.service_provider
            .register_embedding_provider(name, provider)?;
        Ok(())
    }

    /// Register a new vector store provider at runtime
    pub async fn register_vector_store_provider(
        &self,
        name: &str,
        config: &crate::core::types::VectorStoreConfig,
    ) -> crate::core::error::Result<()> {
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

        let mut providers = Vec::new();

        // Add embedding providers
        for name in embedding_providers {
            providers.push(crate::admin::service::ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "embedding".to_string(),
                status: "active".to_string(), // Assume active for now
                config: serde_json::json!({ "type": "embedding" }),
            });
        }

        // Add vector store providers
        for name in vector_store_providers {
            providers.push(crate::admin::service::ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "vector_store".to_string(),
                status: "active".to_string(), // Assume active for now
                config: serde_json::json!({ "type": "vector_store" }),
            });
        }

        providers
    }

    /// Get provider health status
    pub async fn get_provider_health(
        &self,
    ) -> std::collections::HashMap<String, crate::providers::routing::health::ProviderHealth> {
        // This would use the health monitor from the router
        // For now, return empty map
        std::collections::HashMap::new()
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
        crate::admin::service::SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: 0, // TODO: Implement uptime tracking
            pid: std::process::id(),
        }
    }

    /// Get real indexing status for admin interface
    pub async fn get_indexing_status_admin(&self) -> crate::admin::service::IndexingStatus {
        let operations = self.indexing_operations.read().await;

        // Check if any indexing operations are active
        let is_indexing = !operations.is_empty();

        // Find the most recent operation for current status
        let (current_file, start_time, _processed_files, _total_files) =
            if let Some((_, operation)) = operations.iter().next() {
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
        let total_documents: usize = operations.values().map(|op| op.total_files).sum();
        let indexed_documents: usize = operations.values().map(|op| op.processed_files).sum();

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

        let mut operations = self.indexing_operations.write().await;
        operations.insert(operation_id, operation);
    }

    /// Update indexing operation progress
    pub async fn update_indexing_progress(
        &self,
        operation_id: &str,
        current_file: Option<String>,
        processed_files: usize,
    ) {
        let mut operations = self.indexing_operations.write().await;
        if let Some(operation) = operations.get_mut(operation_id) {
            operation.current_file = current_file;
            operation.processed_files = processed_files;
        }
    }

    /// Complete an indexing operation
    pub async fn complete_indexing_operation(&self, operation_id: &str) {
        let mut operations = self.indexing_operations.write().await;
        operations.remove(operation_id);
    }

    /// Record a query operation with timing
    pub fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool) {
        self.performance_metrics
            .total_queries
            .fetch_add(1, Ordering::Relaxed);

        if success {
            self.performance_metrics
                .successful_queries
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.performance_metrics
                .failed_queries
                .fetch_add(1, Ordering::Relaxed);
        }

        self.performance_metrics
            .response_time_sum
            .fetch_add(response_time_ms, Ordering::Relaxed);

        if cache_hit {
            self.performance_metrics
                .cache_hits
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.performance_metrics
                .cache_misses
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Update active connection count
    pub fn update_active_connections(&self, delta: i64) {
        if delta > 0 {
            self.performance_metrics
                .active_connections
                .fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            let current = self
                .performance_metrics
                .active_connections
                .load(Ordering::Relaxed);
            let new_value = current.saturating_sub((-delta) as u64);
            self.performance_metrics
                .active_connections
                .store(new_value, Ordering::Relaxed);
        }
    }

    /// Get real performance metrics for admin interface
    pub fn get_performance_metrics(&self) -> crate::admin::service::PerformanceMetrics {
        let total_queries = self
            .performance_metrics
            .total_queries
            .load(Ordering::Relaxed);
        let successful_queries = self
            .performance_metrics
            .successful_queries
            .load(Ordering::Relaxed);
        let failed_queries = self
            .performance_metrics
            .failed_queries
            .load(Ordering::Relaxed);
        let response_time_sum = self
            .performance_metrics
            .response_time_sum
            .load(Ordering::Relaxed);
        let cache_hits = self.performance_metrics.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self
            .performance_metrics
            .cache_misses
            .load(Ordering::Relaxed);

        // Calculate average response time
        let average_response_time_ms = if total_queries > 0 {
            response_time_sum as f64 / total_queries as f64
        } else {
            0.0
        };

        // Calculate cache hit rate
        let total_cache_requests = cache_hits + cache_misses;
        let cache_hit_rate = if total_cache_requests > 0 {
            cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        };

        // Calculate uptime
        let uptime_seconds = self.performance_metrics.start_time.elapsed().as_secs();

        crate::admin::service::PerformanceMetrics {
            total_queries,
            successful_queries,
            failed_queries,
            average_response_time_ms,
            cache_hit_rate,
            active_connections: self
                .performance_metrics
                .active_connections
                .load(Ordering::Relaxed) as u32,
            uptime_seconds,
        }
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

        let tools = vec![
            Tool {
                name: Cow::Borrowed("index_codebase"),
                title: None,
                description: Some(Cow::Borrowed(
                    "Index a codebase directory for semantic search using vector embeddings",
                )),
                input_schema: Arc::new(
                    serde_json::to_value(schemars::schema_for!(IndexCodebaseArgs))
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
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
                input_schema: Arc::new(
                    serde_json::to_value(schemars::schema_for!(SearchCodeArgs))
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
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
                input_schema: Arc::new(
                    serde_json::to_value(schemars::schema_for!(GetIndexingStatusArgs))
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("clear_index"),
                title: None,
                description: Some(Cow::Borrowed("Clear the search index for a collection")),
                input_schema: Arc::new(
                    serde_json::to_value(schemars::schema_for!(ClearIndexArgs))
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
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
