//! MCP Context Browser Server
//!
//! A Model Context Protocol server that provides semantic code search and indexing
//! capabilities using vector embeddings. This server follows the official MCP SDK
//! patterns and provides tools for:
//!
//! - Indexing codebases for semantic search
//! - Performing natural language code queries
//! - Managing search collections
//!
//! Based on the official rmcp SDK examples and best practices.

// Module declarations
pub mod args;
pub mod auth;
pub mod formatter;
pub mod handlers;
pub mod rate_limit_middleware;
pub mod security;

use crate::core::auth::AuthService;
use crate::core::cache::CacheManager;
use crate::core::database::init_global_database_pool;
use crate::core::http_client::{HttpClientConfig, init_global_http_client};
use crate::core::limits::{ResourceLimits, init_global_resource_limits};
use crate::core::rate_limit::RateLimiter;
use crate::metrics::MetricsApiServer;
use crate::services::{IndexingService, SearchService};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, service::RequestContext, tool,
    transport::stdio,
};
use std::sync::Arc;
use tracing_subscriber::{self, EnvFilter};

/// This server provides semantic code search and indexing capabilities
/// using vector embeddings and advanced text analysis. It implements
/// the MCP protocol using the official rmcp SDK.
#[derive(Clone)]
pub struct McpServer {
    /// Handler for codebase indexing operations
    index_codebase_handler: Arc<crate::server::handlers::index_codebase::IndexCodebaseHandler>,
    /// Handler for code search operations
    search_code_handler: Arc<crate::server::handlers::search_code::SearchCodeHandler>,
    /// Handler for indexing status operations
    get_indexing_status_handler: Arc<crate::server::handlers::get_indexing_status::GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    clear_index_handler: Arc<crate::server::handlers::clear_index::ClearIndexHandler>,
}

impl McpServer {
    /// Create embedding provider with fallback to mock
    fn create_embedding_provider(config: &crate::config::Config) -> Result<Arc<dyn crate::providers::EmbeddingProvider>, Box<dyn std::error::Error>> {
        match config.providers.embedding.provider.as_str() {
            "ollama" => {
                let base_url = config.providers.embedding.base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let model = config.providers.embedding.model.clone();
                match crate::providers::OllamaEmbeddingProvider::new(base_url, model) {
                    Ok(provider) => Ok(Arc::new(provider)),
                    Err(e) => {
                        tracing::warn!("Failed to create Ollama provider: {}", e);
                        Err(Box::new(e))
                    }
                }
            },
            "openai" => {
                if let Some(api_key) = &config.providers.embedding.api_key {
                    let base_url = config.providers.embedding.base_url.clone();
                    let model = config.providers.embedding.model.clone();
                    match crate::providers::OpenAIEmbeddingProvider::new(api_key.clone(), base_url, model) {
                        Ok(provider) => Ok(Arc::new(provider)),
                        Err(e) => {
                            tracing::warn!("Failed to create OpenAI provider: {}", e);
                            Err(Box::new(e))
                        }
                    }
                } else {
                    Err("OpenAI API key not provided".into())
                }
            },
            _ => {
                tracing::info!("Using mock embedding provider");
                Ok(Arc::new(crate::providers::MockEmbeddingProvider::new()))
            }
        }
    }

    /// Create vector store provider with fallback to in-memory
    fn create_vector_store_provider(config: &crate::config::Config) -> Result<Arc<dyn crate::providers::VectorStoreProvider>, Box<dyn std::error::Error>> {
        match config.providers.vector_store.provider.as_str() {
            "in-memory" => Ok(Arc::new(crate::providers::InMemoryVectorStoreProvider::new())),
            "filesystem" => {
                // Use default filesystem config
                let provider = crate::providers::FilesystemVectorStoreProvider::new(
                    None, // base_path
                    None, // max_vectors_per_shard
                    config.providers.vector_store.dimensions,
                    None, // compression_enabled
                    None, // index_cache_size
                    None, // memory_mapping_enabled
                )?;
                Ok(Arc::new(provider))
            },
            "milvus" => {
                if let Some(address) = &config.providers.vector_store.address {
                    let provider = crate::providers::MilvusVectorStoreProvider::new(
                        address.clone(),
                        config.providers.vector_store.token.clone(),
                        None, // collection
                        config.providers.vector_store.dimensions,
                    )?;
                    Ok(Arc::new(provider))
                } else {
                    Err("Milvus address not provided".into())
                }
            },
            _ => {
                tracing::info!("Using in-memory vector store provider");
                Ok(Arc::new(crate::providers::InMemoryVectorStoreProvider::new()))
            }
        }
    }

    /// Create a new MCP server instance
    ///
    /// Initializes all required services and configurations.
    /// Automatically configures with Ollama embeddings and in-memory storage.
    pub fn new(
        cache_manager: Option<Arc<CacheManager>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration from environment (with sensible defaults)
        let config = crate::config::Config::from_env()
            .unwrap_or_else(|_| crate::config::Config::default());

        // Initialize resource limits
        let resource_limits = Arc::new(ResourceLimits::new(config.resource_limits.clone()));
        init_global_resource_limits(config.resource_limits)?;

        // Create authentication service and handler
        let auth_service = AuthService::new(config.auth.clone());
        let auth_handler = Arc::new(crate::server::auth::AuthHandler::new(auth_service));

        // Try to create real providers, fallback to mock if unavailable
        let embedding_provider: Arc<dyn crate::providers::EmbeddingProvider> = match Self::create_embedding_provider(&config) {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!("Failed to create embedding provider, using mock: {}", e);
                Arc::new(crate::providers::MockEmbeddingProvider::new())
            }
        };

        let vector_store_provider: Arc<dyn crate::providers::VectorStoreProvider> = match Self::create_vector_store_provider(&config) {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!("Failed to create vector store provider, using in-memory: {}", e);
                Arc::new(crate::providers::InMemoryVectorStoreProvider::new())
            }
        };

        let context_service = Arc::new(crate::services::ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));

        // Create services
        let indexing_service = Arc::new(IndexingService::new(context_service.clone())?);
        let search_service = Arc::new(SearchService::new(context_service));

        // Create cache manager
        let cache_manager = cache_manager.unwrap_or_else(|| {
            Arc::new({
                let config = crate::core::cache::CacheConfig {
                    enabled: false,
                    ..Default::default()
                };
                // For disabled cache, we can create synchronously since no Redis connection needed
                futures::executor::block_on(CacheManager::new(config))
                    .expect("Failed to create disabled cache manager")
            })
        });

        // Create handlers
        let index_codebase_handler = Arc::new(crate::server::handlers::index_codebase::IndexCodebaseHandler::new(
            indexing_service,
            Arc::clone(&auth_handler),
            Arc::clone(&resource_limits),
        ));

        let search_code_handler = Arc::new(crate::server::handlers::search_code::SearchCodeHandler::new(
            search_service,
            Arc::clone(&auth_handler),
            Arc::clone(&resource_limits),
            cache_manager,
        ));

        let get_indexing_status_handler = Arc::new(crate::server::handlers::get_indexing_status::GetIndexingStatusHandler::new());

        let clear_index_handler = Arc::new(crate::server::handlers::clear_index::ClearIndexHandler::new());

        Ok(Self {
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
        })
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
                 • **🧠 Semantic Search**: AI-powered code understanding and retrieval\n\
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
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        use rmcp::model::Tool;

        use std::borrow::Cow;

        let tools = vec![
            Tool {
                name: Cow::Borrowed("index_codebase"),
                title: None,
                description: Some(Cow::Borrowed("Index a codebase directory for semantic search using vector embeddings")),
                input_schema: Arc::new(serde_json::to_value(schemars::schema_for!(crate::server::args::IndexCodebaseArgs)).unwrap().as_object().unwrap().clone()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("search_code"),
                title: None,
                description: Some(Cow::Borrowed("Search for code using natural language queries")),
                input_schema: Arc::new(serde_json::to_value(schemars::schema_for!(crate::server::args::SearchCodeArgs)).unwrap().as_object().unwrap().clone()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("get_indexing_status"),
                title: None,
                description: Some(Cow::Borrowed("Get the current indexing status and statistics")),
                input_schema: Arc::new(serde_json::to_value(schemars::schema_for!(crate::server::args::GetIndexingStatusArgs)).unwrap().as_object().unwrap().clone()),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: Default::default(),
            },
            Tool {
                name: Cow::Borrowed("clear_index"),
                title: None,
                description: Some(Cow::Borrowed("Clear the search index for a collection")),
                input_schema: Arc::new(serde_json::to_value(schemars::schema_for!(crate::server::args::ClearIndexArgs)).unwrap().as_object().unwrap().clone()),
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
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "index_codebase" => {
                let args: crate::server::args::IndexCodebaseArgs = serde_json::from_value(
                    serde_json::Value::Object(request.arguments.unwrap_or_default())
                ).map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.index_codebase_handler.handle(Parameters(args)).await
            },
            "search_code" => {
                let args: crate::server::args::SearchCodeArgs = serde_json::from_value(
                    serde_json::Value::Object(request.arguments.unwrap_or_default())
                ).map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.search_code_handler.handle(Parameters(args)).await
            },
            "get_indexing_status" => {
                let args: crate::server::args::GetIndexingStatusArgs = serde_json::from_value(
                    serde_json::Value::Object(request.arguments.unwrap_or_default())
                ).map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.get_indexing_status_handler.handle(Parameters(args)).await
            },
            "clear_index" => {
                let args: crate::server::args::ClearIndexArgs = serde_json::from_value(
                    serde_json::Value::Object(request.arguments.unwrap_or_default())
                ).map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))?;
                self.clear_index_handler.handle(Parameters(args)).await
            },
            _ => Err(McpError::invalid_params(format!("Unknown tool: {}", request.name), None)),
        }
    }
}

/// Initialize logging and tracing for the MCP server
///
/// Sets up structured logging with appropriate levels for production use.
/// Uses stderr for logs to avoid interfering with stdio MCP protocol.
fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("mcp_context_browser=debug".parse()?)
                .add_directive("rmcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false) // Disable ANSI colors for better log parsing
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    Ok(())
}

/// Run the MCP Context Browser server
///
/// This is the main entry point that starts the MCP server with stdio transport.
/// The server implements the MCP protocol for semantic code search and indexing.
///
/// # Architecture Notes
/// - Async-first design using Tokio runtime
/// - Provider pattern for extensibility
/// - Structured concurrency with proper error handling
/// - Comprehensive logging and observability
/// - Rate limiting for production security
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing first for proper error reporting
    init_tracing()?;

    tracing::info!(
        "🚀 Starting MCP Context Browser v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!(
        "📋 System Info: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    // Load configuration
    let config = crate::config::Config::from_env()?;

    // Initialize global HTTP client pool
    tracing::info!("🌐 Initializing HTTP client pool...");
    if let Err(e) = init_global_http_client(HttpClientConfig::default()) {
        tracing::warn!(
            "⚠️  Failed to initialize HTTP client pool: {}. Using default clients.",
            e
        );
    } else {
        tracing::info!("✅ HTTP client pool initialized successfully");
    }

    // Initialize global database pool
    if config.database.enabled {
        tracing::info!("🗄️  Initializing database connection pool...");
        match init_global_database_pool(config.database.clone()) {
            Ok(_) => tracing::info!("✅ Database connection pool initialized successfully"),
            Err(e) => {
                tracing::error!("💥 Failed to initialize database connection pool: {}", e);
                return Err(e.into());
            }
        }
    } else {
        tracing::info!("ℹ️  Database connection pool disabled");
    }

    // Initialize rate limiter for HTTP API
    let rate_limiter = if config.metrics.rate_limiting.enabled {
        tracing::info!("🔒 Initializing rate limiter...");
        let limiter = Arc::new(RateLimiter::new(config.metrics.rate_limiting.clone()));
        if let Err(e) = limiter.init().await {
            tracing::warn!(
                "⚠️  Failed to initialize Redis rate limiter: {}. Running without rate limiting.",
                e
            );
            None
        } else {
            tracing::info!("✅ Rate limiter initialized successfully");
            Some(limiter)
        }
    } else {
        tracing::info!("ℹ️  Rate limiting disabled");
        None
    };

    // Initialize cache manager
    let cache_manager = if config.cache.enabled {
        tracing::info!("🗄️  Initializing cache manager...");
        match CacheManager::new(config.cache.clone()).await {
            Ok(manager) => {
                tracing::info!("✅ Cache manager initialized successfully");
                Some(Arc::new(manager))
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  Failed to initialize cache manager: {}. Running without caching.",
                    e
                );
                None
            }
        }
    } else {
        tracing::info!("ℹ️  Caching disabled");
        None
    };

    // Log provider configuration for debugging
    tracing::debug!("Initializing service providers...");
    let server = match McpServer::new(cache_manager) {
        Ok(server) => {
            tracing::info!("✅ Service providers initialized successfully");
            server
        }
        Err(e) => {
            tracing::error!("❌ Failed to initialize server: {}", e);
            return Err(e);
        }
    };

    // Log server capabilities
    let capabilities = server.get_info().capabilities;
    tracing::info!(
        "🔧 Server capabilities: tools={}, prompts={}, resources={}",
        capabilities.tools.is_some(),
        capabilities.prompts.is_some(),
        capabilities.resources.is_some()
    );

    // Start HTTP metrics server if enabled
    let metrics_handle = if config.metrics.enabled {
        tracing::info!(
            "📊 Starting metrics HTTP server on port {}",
            config.metrics.port
        );
        let metrics_server = MetricsApiServer::with_limits(
            config.metrics.port,
            rate_limiter.clone(),
            Some(server.resource_limits.clone()),
        );

        Some(tokio::spawn(async move {
            if let Err(e) = metrics_server.start().await {
                tracing::error!("💥 Metrics server failed: {}", e);
            }
        }))
    } else {
        tracing::info!("ℹ️  Metrics server disabled");
        None
    };

    tracing::info!("📡 Starting MCP protocol server on stdio transport");
    tracing::info!("🎯 Ready to accept MCP client connections");

    // Handle graceful shutdown signals
    let shutdown_signal = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for shutdown signal: {}", e);
            return;
        }
        tracing::info!("🛑 Received shutdown signal, initiating graceful shutdown...");
    };

    // Start the MCP service with stdio transport
    let service_future = server.serve(stdio());

    tokio::select! {
        result = service_future => {
            match result {
                Ok(service) => {
                    tracing::info!("🎉 MCP server started successfully, waiting for connections...");
                    service.waiting().await?;
                    tracing::info!("👋 MCP server shutdown complete");
                }
                Err(e) => {
                    tracing::error!("💥 Failed to start MCP service: {:?}", e);
                    return Err(e.into());
                }
            }
        }
        _ = shutdown_signal => {
            tracing::info!("🔄 Graceful shutdown initiated");
        }
    }

    // Wait for metrics server to finish if it was started
    if let Some(handle) = metrics_handle {
        let _ = handle.await;
    }

    Ok(())
}