//! Infrastructure layer constants
//!
//! Contains constants that are part of the infrastructure implementation.
//! Domain-specific constants are defined in `mcb_domain::constants`.

// ============================================================================
// CONFIGURATION CONSTANTS
// ============================================================================

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = "mcb.toml";

/// Default configuration directory name
pub const DEFAULT_CONFIG_DIR: &str = "mcb";

/// Environment variable prefix for configuration
pub const CONFIG_ENV_PREFIX: &str = "MCB";

// ============================================================================
// AUTHENTICATION CONSTANTS
// ============================================================================

/// JWT default expiration time in seconds (24 hours)
pub const JWT_DEFAULT_EXPIRATION_SECS: u64 = 86400;

/// JWT refresh token expiration time in seconds (7 days)
pub const JWT_REFRESH_EXPIRATION_SECS: u64 = 604800;

/// Default bcrypt cost for password hashing
pub const BCRYPT_DEFAULT_COST: u32 = 12;

/// API key header name
pub const API_KEY_HEADER: &str = "x-api-key";

/// Authorization header name
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// Bearer token prefix
pub const BEARER_PREFIX: &str = "Bearer ";

// ============================================================================
// CACHE CONSTANTS
// ============================================================================

/// Default cache TTL in seconds (1 hour)
pub const CACHE_DEFAULT_TTL_SECS: u64 = 3600;

/// Default cache size limit in bytes (100MB)
pub const CACHE_DEFAULT_SIZE_LIMIT: usize = 100 * 1024 * 1024;

/// Redis connection pool size
pub const REDIS_POOL_SIZE: usize = 10;

/// Cache namespace separator
pub const CACHE_NAMESPACE_SEPARATOR: &str = ":";

// ============================================================================
// HTTP SERVER CONSTANTS
// ============================================================================

/// Default HTTP server port
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default HTTPS server port
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Default server host
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// Request timeout in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Connection timeout in seconds
pub const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Maximum request body size in bytes (10MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Health check endpoint path
pub const HEALTH_CHECK_PATH: &str = "/health";

/// Metrics endpoint path
pub const METRICS_PATH: &str = "/metrics";

// ============================================================================
// HTTP CLIENT POOL CONSTANTS
// ============================================================================

/// HTTP client request timeout in seconds (for embedding API calls)
pub const HTTP_REQUEST_TIMEOUT_SECS: u64 = 30;

/// HTTP client idle timeout in seconds
pub const HTTP_CLIENT_IDLE_TIMEOUT_SECS: u64 = 90;

/// HTTP client TCP keep-alive duration in seconds
pub const HTTP_KEEPALIVE_SECS: u64 = 60;

/// Maximum idle connections per host in HTTP client pool
pub const HTTP_MAX_IDLE_PER_HOST: usize = 10;

// ============================================================================
// RESILIENCE CONSTANTS
// ============================================================================

/// Circuit breaker failure threshold
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: u32 = 5;

/// Circuit breaker timeout in seconds
pub const CIRCUIT_BREAKER_TIMEOUT_SECS: u64 = 60;

/// Circuit breaker success threshold
pub const CIRCUIT_BREAKER_SUCCESS_THRESHOLD: u32 = 3;

/// Rate limiter default requests per second
pub const RATE_LIMITER_DEFAULT_RPS: u32 = 100;

/// Rate limiter burst size
pub const RATE_LIMITER_DEFAULT_BURST: u32 = 200;

// ============================================================================
// METRICS CONSTANTS
// ============================================================================

/// Metrics collection interval in seconds
pub const METRICS_COLLECTION_INTERVAL_SECS: u64 = 60;

/// Prometheus metrics prefix
pub const METRICS_PREFIX: &str = "mcb";

// ============================================================================
// FILE SYSTEM CONSTANTS
// ============================================================================

/// Default file permissions (0o644)
pub const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;

/// Default directory permissions (0o755)
pub const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;

/// Maximum file size for snapshot operations in bytes (100MB)
pub const MAX_SNAPSHOT_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Backup file extension
pub const BACKUP_FILE_EXTENSION: &str = ".backup";

/// Temporary file prefix
pub const TEMP_FILE_PREFIX: &str = "mcb_temp_";

// ============================================================================
// FILESYSTEM VECTOR STORE CONSTANTS
// ============================================================================

/// Maximum vectors per shard file in filesystem vector store
pub const FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD: usize = 100_000;

/// Index cache size (number of index entries to keep in memory)
pub const FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE: usize = 10_000;

/// Bytes per dimension for vector storage (f32 = 4 bytes)
pub const FILESYSTEM_BYTES_PER_DIMENSION: usize = 4;

// ============================================================================
// DATABASE CONSTANTS
// ============================================================================

/// Default database connection pool size
pub const DB_POOL_SIZE: u32 = 10;

/// Database connection timeout in seconds
pub const DB_CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Database query timeout in seconds
pub const DB_QUERY_TIMEOUT_SECS: u64 = 60;

// ============================================================================
// EVENT BUS CONSTANTS
// ============================================================================

/// NATS default connection timeout in seconds
pub const NATS_CONNECT_TIMEOUT_SECS: u64 = 10;

/// NATS default request timeout in seconds
pub const NATS_REQUEST_TIMEOUT_SECS: u64 = 5;

/// Event bus buffer size
pub const EVENT_BUS_BUFFER_SIZE: usize = 1000;

// ============================================================================
// LOGGING CONSTANTS
// ============================================================================

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Log file rotation size in bytes (10MB)
pub const LOG_ROTATION_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of log files to keep
pub const LOG_MAX_FILES: usize = 5;

// ============================================================================
// DAEMON CONSTANTS
// ============================================================================

/// Daemon process check interval in seconds
pub const DAEMON_CHECK_INTERVAL_SECS: u64 = 30;

/// Daemon restart delay in seconds
pub const DAEMON_RESTART_DELAY_SECS: u64 = 5;

/// Maximum restart attempts
pub const DAEMON_MAX_RESTART_ATTEMPTS: u32 = 3;

// ============================================================================
// SHUTDOWN CONSTANTS
// ============================================================================

/// Graceful shutdown timeout in seconds
pub const GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Force shutdown timeout in seconds
pub const FORCE_SHUTDOWN_TIMEOUT_SECS: u64 = 10;

// ============================================================================
// SIGNAL CONSTANTS
// ============================================================================

/// Signal handling poll interval in milliseconds
pub const SIGNAL_POLL_INTERVAL_MS: u64 = 100;

// ============================================================================
// CRYPTO CONSTANTS
// ============================================================================

/// AES-GCM key size in bytes
pub const AES_GCM_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// PBKDF2 iterations for key derivation
pub const PBKDF2_ITERATIONS: u32 = 100_000;

// ============================================================================
// SYNC CONSTANTS
// ============================================================================

/// Sync batch size
pub const SYNC_BATCH_SIZE: usize = 100;

/// Sync debounce delay in milliseconds
pub const SYNC_DEBOUNCE_DELAY_MS: u64 = 500;

/// Sync timeout in seconds
pub const SYNC_TIMEOUT_SECS: u64 = 300;

// ============================================================================
// LIMITS CONSTANTS
// ============================================================================

/// Default memory limit in bytes (1GB)
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;

/// Default CPU limit (number of cores)
pub const DEFAULT_CPU_LIMIT: usize = 4;

/// Default disk I/O limit in bytes per second (100MB/s)
pub const DEFAULT_DISK_IO_LIMIT: u64 = 100 * 1024 * 1024;

// ============================================================================
// OPERATIONS CONSTANTS
// ============================================================================

/// Operations tracking cleanup interval in seconds
pub const OPERATIONS_CLEANUP_INTERVAL_SECS: u64 = 3600;

/// Operations tracking retention period in seconds (7 days)
pub const OPERATIONS_RETENTION_SECS: u64 = 604800;

/// Maximum number of operations to keep in memory
pub const OPERATIONS_MAX_IN_MEMORY: usize = 10000;

// ============================================================================
// HEALTH CHECK CONSTANTS
// ============================================================================

/// Health check timeout in seconds
pub const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

/// Health check interval in seconds
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// Health check failure threshold
pub const HEALTH_CHECK_FAILURE_THRESHOLD: u32 = 3;

// ============================================================================
// EMBEDDING PROVIDER CONSTANTS
// ============================================================================

/// Null embedding provider dimension (for testing)
pub const EMBEDDING_DIMENSION_NULL: usize = 384;

/// FastEmbed default dimension (BAAI/bge models)
pub const EMBEDDING_DIMENSION_FASTEMBED_DEFAULT: usize = 384;

/// OpenAI text-embedding-3-small dimension
pub const EMBEDDING_DIMENSION_OPENAI_SMALL: usize = 1536;

/// OpenAI text-embedding-3-large dimension
pub const EMBEDDING_DIMENSION_OPENAI_LARGE: usize = 3072;

/// OpenAI text-embedding-ada-002 dimension
pub const EMBEDDING_DIMENSION_OPENAI_ADA: usize = 1536;

/// VoyageAI default dimension
pub const EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT: usize = 1024;

/// VoyageAI code model dimension
pub const EMBEDDING_DIMENSION_VOYAGEAI_CODE: usize = 1024;

/// Ollama nomic-embed-text dimension
pub const EMBEDDING_DIMENSION_OLLAMA_NOMIC: usize = 768;

/// Ollama all-minilm dimension
pub const EMBEDDING_DIMENSION_OLLAMA_MINILM: usize = 384;

/// Ollama mxbai-embed-large dimension
pub const EMBEDDING_DIMENSION_OLLAMA_MXBAI: usize = 1024;

/// Ollama snowflake-arctic-embed dimension
pub const EMBEDDING_DIMENSION_OLLAMA_ARCTIC: usize = 768;

/// Ollama default dimension
pub const EMBEDDING_DIMENSION_OLLAMA_DEFAULT: usize = 768;

/// Gemini embedding dimension
pub const EMBEDDING_DIMENSION_GEMINI: usize = 768;

/// Default embedding dimension (for providers that don't specify)
pub const EMBEDDING_DIMENSION_DEFAULT: usize = 512;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

/// Redis default port
pub const REDIS_DEFAULT_PORT: u16 = 6379;

/// Admin service default port
pub const ADMIN_SERVICE_DEFAULT_PORT: u16 = 3000;

/// Ollama server default port
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

// ============================================================================
// EMBEDDING PROVIDER API CONSTANTS
// ============================================================================

/// VoyageAI max input tokens
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// VoyageAI max output tokens
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// OpenAI token cache TTL in seconds (2 hours)
pub const OPENAI_TOKEN_CACHE_TTL_SECS: u64 = 7200;

/// OpenAI max tokens per request
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Encrypted data padding alignment
pub const ENCRYPTED_DATA_PADDING: usize = 256;

// ============================================================================
// HTTP HEADER CONSTANTS
// ============================================================================

/// Content-Type header name
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";

/// Accept header name
pub const HTTP_HEADER_ACCEPT: &str = "Accept";

/// User-Agent header name
pub const HTTP_HEADER_USER_AGENT: &str = "User-Agent";

/// JSON content type
pub const CONTENT_TYPE_JSON: &str = "application/json";

// ============================================================================
// AST NODE TYPE CONSTANTS
// ============================================================================

/// Function definition AST node type
pub const AST_NODE_FUNCTION_DEFINITION: &str = "function_definition";

/// Function declaration AST node type
pub const AST_NODE_FUNCTION_DECLARATION: &str = "function_declaration";

/// Method declaration AST node type
pub const AST_NODE_METHOD_DECLARATION: &str = "method_declaration";

/// Class declaration AST node type
pub const AST_NODE_CLASS_DECLARATION: &str = "class_declaration";

/// Interface declaration AST node type
pub const AST_NODE_INTERFACE_DECLARATION: &str = "interface_declaration";

/// Struct specifier AST node type (C/C++)
pub const AST_NODE_STRUCT_SPECIFIER: &str = "struct_specifier";

// ============================================================================
// METADATA KEY CONSTANTS
// ============================================================================

/// Start line metadata key
pub const METADATA_KEY_START_LINE: &str = "start_line";

/// End line metadata key
pub const METADATA_KEY_END_LINE: &str = "end_line";

/// Chunk type metadata key
pub const METADATA_KEY_CHUNK_TYPE: &str = "chunk_type";

/// File path metadata key
pub const METADATA_KEY_FILE_PATH: &str = "file_path";

/// Vectors count metadata key
pub const METADATA_KEY_VECTORS_COUNT: &str = "vectors_count";

// ============================================================================
// LANGUAGE CONSTANTS
// ============================================================================

/// JavaScript language identifier
pub const LANG_JAVASCRIPT: &str = "javascript";

/// TypeScript language identifier
pub const LANG_TYPESCRIPT: &str = "typescript";

/// Python language identifier
pub const LANG_PYTHON: &str = "python";

/// Rust language identifier
pub const LANG_RUST: &str = "rust";

// ============================================================================
// ENVIRONMENT VARIABLE CONSTANTS
// ============================================================================

/// Cargo package version environment variable
pub const ENV_CARGO_PKG_VERSION: &str = "CARGO_PKG_VERSION";

// ============================================================================
// ERROR MESSAGE CONSTANTS
// ============================================================================

/// Request timeout error message format
pub const ERROR_MSG_REQUEST_TIMEOUT: &str = "Request timed out after {:?}";

// ============================================================================
// BM25 / HYBRID SEARCH CONSTANTS
// ============================================================================

/// BM25 k1 parameter - controls term frequency saturation (default: 1.2)
pub const HYBRID_SEARCH_BM25_K1: f64 = 1.2;

/// BM25 b parameter - controls document length normalization (default: 0.75)
pub const HYBRID_SEARCH_BM25_B: f64 = 0.75;

/// Minimum token length for BM25 tokenization
pub const BM25_TOKEN_MIN_LENGTH: usize = 2;

/// Default BM25 weight in hybrid search (0.4 = 40% keyword-based)
pub const HYBRID_SEARCH_BM25_WEIGHT: f64 = 0.4;

/// Default semantic weight in hybrid search (0.6 = 60% embedding-based)
pub const HYBRID_SEARCH_SEMANTIC_WEIGHT: f64 = 0.6;

/// Maximum number of results to retrieve from each search method before fusion
pub const HYBRID_SEARCH_MAX_CANDIDATES: usize = 100;

// ============================================================================
// TREE-SITTER NODE TYPE CONSTANTS
// ============================================================================

/// Tree-sitter node type: function declaration (JS, Go, Swift, Kotlin)
pub const TS_NODE_FUNCTION_DECLARATION: &str = "function_declaration";

/// Tree-sitter node type: function definition (Python, C, C++, PHP)
pub const TS_NODE_FUNCTION_DEFINITION: &str = "function_definition";

/// Tree-sitter node type: method declaration (Go, Java, C#, PHP)
pub const TS_NODE_METHOD_DECLARATION: &str = "method_declaration";

/// Tree-sitter node type: class declaration (JS, Java, C#, PHP, Swift, Kotlin)
pub const TS_NODE_CLASS_DECLARATION: &str = "class_declaration";

// Re-export domain constants for convenience
pub use mcb_domain::constants::*;
