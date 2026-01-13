//! Application-wide constants and default values
//!
//! Centralizes configuration of timeout values, limits, and other magic numbers
//! to ensure consistency across the codebase and enable easy customization.

use std::time::Duration;

// ============================================================================
// HTTP Request Timeouts
// ============================================================================

/// Default HTTP request timeout for embedding providers (OpenAI, Ollama, etc.)
///
/// This timeout applies to API calls that should complete relatively quickly.
/// Embedding requests typically take 100-500ms depending on model size.
/// 30 seconds provides ample margin for network latency and model inference.
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP request timeout for indexing operations
///
/// Large codebase indexing can take significant time due to:
/// - File system I/O
/// - AST parsing
/// - Embedding generation
/// - Vector database writes
///
/// 5 minutes (300 seconds) is a reasonable upper bound for most codebases.
/// This can be overridden per-request if needed.
pub const INDEXING_OPERATION_TIMEOUT: Duration = Duration::from_secs(300);

// ============================================================================
// Cache TTL (Time-To-Live) Values
// ============================================================================

/// Cache TTL for search results
///
/// Search results are cached to avoid redundant embedding and vector searches.
/// 30 minutes balances freshness with cache hit rates. Long enough to cover
/// multiple queries in a typical analysis session, short enough that stale
/// results aren't served for long.
pub const SEARCH_RESULT_CACHE_TTL: Duration = Duration::from_secs(1800);

// ============================================================================
// Search Configuration
// ============================================================================

/// Minimum query length in characters
///
/// Prevents single-character or trivial queries from consuming resources.
pub const SEARCH_QUERY_MIN_LENGTH: usize = 3;

/// Maximum query length in characters
///
/// Prevents extremely long queries that could cause performance issues
/// in embedding generation or vector search.
pub const SEARCH_QUERY_MAX_LENGTH: usize = 1000;

/// Maximum number of search results to return
///
/// Even if user requests more, results are clamped to this value to ensure
/// reasonable response sizes and prevent excessive resource consumption.
pub const SEARCH_RESULT_LIMIT_MAX: usize = 50;

/// Minimum acceptable search result limit (must request at least 1)
pub const SEARCH_RESULT_LIMIT_MIN: usize = 1;

// ============================================================================
// FastEmbed Model Configuration
// ============================================================================

/// FastEmbed model maximum token length
///
/// FastEmbed models have a fixed maximum token length. This is model-specific.
/// Default FastEmbed models (like BAAI/bge-base-en-v1.5) support up to 512 tokens.
/// Note: This should ideally come from model introspection API, not hardcoded.
pub const FASTEMBED_MAX_TOKENS: usize = 512;

/// FastEmbed model embedding dimension
///
/// Default FastEmbed models produce embeddings of this dimension.
/// This is model-specific. BAAI/bge-base-en-v1.5 produces 768-dimensional embeddings.
/// Note: This should ideally come from model introspection API, not hardcoded.
pub const FASTEMBED_EMBEDDING_DIMENSION: usize = 384;

// ============================================================================
// Protected Collection Names
// ============================================================================

/// System-reserved collection name (cannot be cleared)
pub const PROTECTED_COLLECTION_SYSTEM: &str = "system";

/// Admin-reserved collection name (cannot be cleared)
pub const PROTECTED_COLLECTION_ADMIN: &str = "admin";

// ============================================================================
// HTTP & Network Configuration
// ============================================================================

/// HTTP client idle timeout
pub const HTTP_CLIENT_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

/// HTTP keepalive duration
pub const HTTP_KEEPALIVE_DURATION: Duration = Duration::from_secs(60);

/// Max idle HTTP connections per host
pub const HTTP_MAX_IDLE_PER_HOST: u32 = 10;

// ============================================================================
// Server Port Configuration
// ============================================================================

/// Default MCP server port
pub const SERVER_DEFAULT_PORT: u16 = 3000;

/// Default metrics port
pub const METRICS_DEFAULT_PORT: u16 = 3001;

// ============================================================================
// Database Connection Pool
// ============================================================================

/// Default max database connections
pub const DB_MAX_CONNECTIONS: u32 = 20;

/// Default minimum idle database connections
pub const DB_MIN_IDLE: u32 = 5;

/// Database connection max lifetime (30 minutes)
pub const DB_CONNECTION_MAX_LIFETIME: Duration = Duration::from_secs(1800);

/// Database connection idle timeout (10 minutes)
pub const DB_CONNECTION_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

/// Database connection timeout (30 seconds)
pub const DB_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Resource Limits Configuration
// ============================================================================

/// Memory max usage percentage before unhealthy
pub const MEMORY_LIMIT_UNHEALTHY_PERCENT: f64 = 85.0;

/// Memory max per operation (512MB)
pub const MEMORY_LIMIT_PER_OPERATION: usize = 512 * 1024 * 1024;

/// Memory warning threshold percentage
pub const MEMORY_WARNING_THRESHOLD: f64 = 75.0;

/// CPU max usage percentage before unhealthy
pub const CPU_LIMIT_UNHEALTHY_PERCENT: f64 = 80.0;

/// CPU timeout per operation (5 minutes)
pub const CPU_TIMEOUT_PER_OPERATION: Duration = Duration::from_secs(300);

/// CPU warning threshold percentage
pub const CPU_WARNING_THRESHOLD: f64 = 70.0;

/// Disk max usage percentage before unhealthy
pub const DISK_LIMIT_UNHEALTHY_PERCENT: f64 = 90.0;

/// Disk minimum free space requirement (1GB)
pub const DISK_MIN_FREE_SPACE: u64 = 1024 * 1024 * 1024;

/// Disk warning threshold percentage
pub const DISK_WARNING_THRESHOLD: f64 = 80.0;

// ============================================================================
// Concurrent Operation Limits
// ============================================================================

/// Maximum concurrent indexing operations
pub const MAX_CONCURRENT_INDEXING: usize = 3;

/// Maximum concurrent search operations
pub const MAX_CONCURRENT_SEARCH: usize = 10;

/// Maximum concurrent embedding operations
pub const MAX_CONCURRENT_EMBEDDING: usize = 5;

/// Operation queue maximum size
pub const MAX_OPERATION_QUEUE_SIZE: usize = 100;

// ============================================================================
// Admin Service Defaults
// ============================================================================

/// Maximum activities in memory
pub const ADMIN_MAX_ACTIVITIES: usize = 100;

/// Activity record retention (days)
pub const ADMIN_ACTIVITY_RETENTION_DAYS: u32 = 30;

/// Activity buffer capacity
pub const ADMIN_ACTIVITY_BUFFER_SIZE: usize = 1000;

/// Maximum configuration history entries
pub const ADMIN_MAX_HISTORY_ENTRIES: usize = 1000;

/// Configuration history retention (days)
pub const ADMIN_HISTORY_RETENTION_DAYS: u32 = 90;

/// Maximum history entries per query
pub const ADMIN_CONFIG_QUERY_LIMIT: usize = 100;

/// Log buffer capacity
pub const ADMIN_LOG_BUFFER_SIZE: usize = 1000;

/// Log retention (days)
pub const ADMIN_LOG_RETENTION_DAYS: u32 = 7;

/// Maximum log entries per query
pub const ADMIN_LOG_QUERY_LIMIT: usize = 100;

/// Backup retention (days)
pub const ADMIN_BACKUP_RETENTION_DAYS: u32 = 30;

/// Gzip compression level (1-9)
pub const ADMIN_BACKUP_COMPRESSION_LEVEL: u32 = 6;

/// Maximum backup files
pub const ADMIN_MAX_BACKUPS: usize = 10;

/// Backup directory path
pub const ADMIN_BACKUPS_DIR: &str = "./backups";

/// Data directory path
pub const ADMIN_DATA_DIR: &str = "./data";

/// Exports directory path
pub const ADMIN_EXPORTS_DIR: &str = "./exports";

// ============================================================================
// Health Check Configuration
// ============================================================================

/// CPU unhealthy threshold for health checks
pub const HEALTH_CPU_UNHEALTHY_PERCENT: f64 = 90.0;

/// CPU degraded threshold for health checks
pub const HEALTH_CPU_DEGRADED_PERCENT: f64 = 75.0;

/// Memory unhealthy threshold for health checks
pub const HEALTH_MEMORY_UNHEALTHY_PERCENT: f64 = 90.0;

/// Memory degraded threshold for health checks
pub const HEALTH_MEMORY_DEGRADED_PERCENT: f64 = 80.0;

/// Disk unhealthy threshold for health checks
pub const HEALTH_DISK_UNHEALTHY_PERCENT: f64 = 90.0;

/// Disk degraded threshold for health checks
pub const HEALTH_DISK_DEGRADED_PERCENT: f64 = 80.0;

/// Database pool unhealthy threshold for health checks
pub const HEALTH_DB_POOL_UNHEALTHY_PERCENT: f64 = 95.0;

/// Database pool degraded threshold for health checks
pub const HEALTH_DB_POOL_DEGRADED_PERCENT: f64 = 80.0;

/// Cache hit rate degraded threshold (50%)
pub const HEALTH_CACHE_HIT_RATE_DEGRADED: f64 = 0.5;

// ============================================================================
// Performance Test Configuration
// ============================================================================

/// Default performance test duration (seconds)
pub const PERF_TEST_DURATION_SECS: u32 = 30;

/// Default performance test concurrency
pub const PERF_TEST_CONCURRENCY: u32 = 4;

/// Performance test timeout (5 seconds)
pub const PERF_TEST_TIMEOUT_MS: u64 = 5000;

/// Performance test P95 multiplier
pub const PERF_P95_MULTIPLIER: f64 = 1.2;

/// Performance test P99 multiplier
pub const PERF_P99_MULTIPLIER: f64 = 1.5;

// ============================================================================
// Rate Limiting Configuration
// ============================================================================

/// Health endpoint rate limit (requests/minute)
pub const RATE_LIMIT_HEALTH: u32 = 100;

/// Admin endpoint rate limit (requests/minute)
pub const RATE_LIMIT_ADMIN: u32 = 100;

/// Indexing endpoint rate limit (requests/minute)
pub const RATE_LIMIT_INDEXING: u32 = 10;

/// Search endpoint rate limit (requests/minute)
pub const RATE_LIMIT_SEARCH: u32 = 10;

/// Shutdown cooldown (seconds)
pub const RATE_LIMIT_SHUTDOWN_COOLDOWN: u32 = 60;

/// Reload cooldown (seconds)
pub const RATE_LIMIT_RELOAD_COOLDOWN: u32 = 30;

/// Backup cooldown (seconds)
pub const RATE_LIMIT_BACKUP_COOLDOWN: u32 = 60;

/// Restore rate limit (requests/minute)
pub const RATE_LIMIT_RESTORE: u32 = 10;

/// In-memory rate limiting cache max entries
pub const RATE_LIMIT_CACHE_MAX_ENTRIES: usize = 10000;

/// Default rate limit requests per window
pub const RATE_LIMIT_DEFAULT_MAX_REQUESTS: u32 = 100;

/// Internal rate limiting window in seconds
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

/// Internal rate limiting burst allowance
pub const RATE_LIMIT_BURST_ALLOWANCE: u32 = 20;

/// Auth rate limiting max requests per window
pub const RATE_LIMIT_AUTH_MAX_REQUESTS: u32 = 10;

/// Auth rate limiting window in seconds
pub const RATE_LIMIT_AUTH_WINDOW_SECONDS: u64 = 60;

/// Auth rate limiting lockout duration in seconds (5 minutes)
pub const RATE_LIMIT_AUTH_LOCKOUT_DURATION: u64 = 300;

/// Auth rate limiting max failed attempts
pub const RATE_LIMIT_AUTH_MAX_FAILED_ATTEMPTS: u32 = 5;

// ============================================================================
// Cleanup Configuration
// ============================================================================

/// Cleanup batch size
pub const CLEANUP_BATCH_SIZE: usize = 100;

/// Cleanup retention (days)
pub const CLEANUP_RETENTION_DAYS: u32 = 30;

/// Index rebuild timeout (seconds, 1 hour)
pub const INDEX_REBUILD_TIMEOUT_SECS: u64 = 3600;

/// Cache clear timeout (seconds, 5 minutes)
pub const CACHE_CLEAR_TIMEOUT_SECS: u64 = 300;

// ============================================================================
// Hybrid Search Configuration
// ============================================================================

/// BM25 weight in hybrid search (0.0-1.0)
pub const HYBRID_SEARCH_BM25_WEIGHT: f64 = 0.4;

/// Semantic weight in hybrid search (0.0-1.0)
pub const HYBRID_SEARCH_SEMANTIC_WEIGHT: f64 = 0.6;

/// BM25 k1 parameter (standard tuning value)
pub const HYBRID_SEARCH_BM25_K1: f64 = 1.2;

/// BM25 b parameter (standard tuning value)
pub const HYBRID_SEARCH_BM25_B: f64 = 0.75;

// ============================================================================
// Provider Routing Configuration
// ============================================================================

/// Health factor weight in provider scoring
pub const PROVIDER_ROUTING_HEALTH_WEIGHT: f64 = 0.3;

/// Cost factor weight in provider scoring
pub const PROVIDER_ROUTING_COST_WEIGHT: f64 = 0.25;

/// Quality factor weight in provider scoring
pub const PROVIDER_ROUTING_QUALITY_WEIGHT: f64 = 0.2;

/// Latency factor weight in provider scoring
pub const PROVIDER_ROUTING_LATENCY_WEIGHT: f64 = 0.15;

/// Load factor weight in provider scoring
pub const PROVIDER_ROUTING_LOAD_WEIGHT: f64 = 0.1;

/// OpenAI quality score
pub const PROVIDER_QUALITY_OPENAI: f64 = 0.9;

/// Anthropic quality score
pub const PROVIDER_QUALITY_ANTHROPIC: f64 = 0.95;

/// Gemini quality score
pub const PROVIDER_QUALITY_GEMINI: f64 = 0.85;

/// Ollama quality score (local)
pub const PROVIDER_QUALITY_OLLAMA: f64 = 0.75;

/// Default provider quality score
pub const PROVIDER_QUALITY_DEFAULT: f64 = 0.6;

/// Ollama latency score (local, fast)
pub const PROVIDER_LATENCY_OLLAMA: f64 = 0.9;

/// OpenAI latency score (cloud, variable)
pub const PROVIDER_LATENCY_OPENAI: f64 = 0.7;

/// Gemini latency score (cloud, good)
pub const PROVIDER_LATENCY_GEMINI: f64 = 0.8;

/// Default provider latency score
pub const PROVIDER_LATENCY_DEFAULT: f64 = 0.75;

/// Provider load score for low load
pub const PROVIDER_LOAD_LOW: f64 = 1.0;

/// Provider load score for medium load
pub const PROVIDER_LOAD_MEDIUM: f64 = 0.9;

/// Provider load score for high load (local)
pub const PROVIDER_LOAD_HIGH_LOCAL: f64 = 0.9;

/// Provider load score for high load (default)
pub const PROVIDER_LOAD_HIGH_DEFAULT: f64 = 0.6;

/// Provider load score for critical load (local)
pub const PROVIDER_LOAD_CRITICAL_LOCAL: f64 = 0.8;

/// Provider load score for critical load (default)
pub const PROVIDER_LOAD_CRITICAL_DEFAULT: f64 = 0.4;

/// Provider context default cost sensitivity (0.0-1.0)
pub const PROVIDER_CONTEXT_COST_SENSITIVITY_DEFAULT: f64 = 0.5;

/// Provider context default quality requirement (0.0-1.0)
pub const PROVIDER_CONTEXT_QUALITY_DEFAULT: f64 = 0.5;

/// Provider context default latency sensitivity (0.0-1.0)
pub const PROVIDER_CONTEXT_LATENCY_DEFAULT: f64 = 0.5;

/// Bonus score for preferred providers
pub const PROVIDER_PREFERENCE_BONUS: f64 = 0.1;

/// BM25 token minimum length filter
pub const BM25_TOKEN_MIN_LENGTH: usize = 2;

/// Max failover attempts before giving up
pub const PROVIDER_MAX_FAILOVER_ATTEMPTS: usize = 3;

/// OpenAI provider priority for failover
pub const PROVIDER_PRIORITY_OPENAI: u32 = 2;

/// Gemini provider priority for failover
pub const PROVIDER_PRIORITY_GEMINI: u32 = 4;

/// Default provider priority for failover
pub const PROVIDER_PRIORITY_DEFAULT: u32 = 100;

// ============================================================================
// Circuit Breaker Configuration
// ============================================================================

/// Failures before circuit opens
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: usize = 5;

/// Recovery timeout (60 seconds)
pub const CIRCUIT_BREAKER_RECOVERY_TIMEOUT: Duration = Duration::from_secs(60);

/// Successes to close circuit
pub const CIRCUIT_BREAKER_SUCCESS_THRESHOLD: usize = 3;

/// Max requests in half-open state
pub const CIRCUIT_BREAKER_HALF_OPEN_MAX_REQUESTS: u32 = 10;

/// State persistence save interval (30 seconds)
pub const CIRCUIT_BREAKER_PERSIST_INTERVAL: Duration = Duration::from_secs(30);

// ============================================================================
// Code Chunking Configuration
// ============================================================================

/// Default lines per code chunk
pub const DEFAULT_CHUNK_SIZE: usize = 20;

/// Rust language chunk size
pub const CHUNK_SIZE_RUST: usize = 20;

/// Python language chunk size
pub const CHUNK_SIZE_PYTHON: usize = 15;

/// JavaScript/TypeScript language chunk size
pub const CHUNK_SIZE_JAVASCRIPT: usize = 15;

/// Go language chunk size
pub const CHUNK_SIZE_GO: usize = 15;

/// Java language chunk size
pub const CHUNK_SIZE_JAVA: usize = 15;

/// C language chunk size
pub const CHUNK_SIZE_C: usize = 15;

/// C++ language chunk size
pub const CHUNK_SIZE_CPP: usize = 15;

/// C# language chunk size
pub const CHUNK_SIZE_CSHARP: usize = 15;

/// Ruby language chunk size
pub const CHUNK_SIZE_RUBY: usize = 15;

/// PHP language chunk size
pub const CHUNK_SIZE_PHP: usize = 15;

/// Swift language chunk size
pub const CHUNK_SIZE_SWIFT: usize = 15;

/// Kotlin language chunk size
pub const CHUNK_SIZE_KOTLIN: usize = 15;

/// Node extraction rule default minimum content length
pub const NODE_EXTRACTION_MIN_LENGTH: usize = 20;

/// Node extraction rule default minimum lines
pub const NODE_EXTRACTION_MIN_LINES: usize = 1;

/// Node extraction rule default maximum depth
pub const NODE_EXTRACTION_MAX_DEPTH: usize = 3;

/// Node extraction rule default priority
pub const NODE_EXTRACTION_DEFAULT_PRIORITY: i32 = 5;

// ============================================================================
// Vector Store Configuration
// ============================================================================

/// Milvus field varchar max length (file paths)
pub const MILVUS_FIELD_VARCHAR_MAX_LENGTH: i32 = 512;

/// Milvus metadata field varchar max length
pub const MILVUS_METADATA_VARCHAR_MAX_LENGTH: i32 = 65535;

/// Milvus IVFFLAT index parameter (nlist)
pub const MILVUS_IVFFLAT_NLIST: usize = 1024;

/// Filesystem vector store max vectors per shard
pub const FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD: usize = 100000;

/// Filesystem vector store index cache size
pub const FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE: usize = 10000;

/// Filesystem vector store bytes per dimension (float32)
pub const FILESYSTEM_BYTES_PER_DIMENSION: usize = 4;

/// EdgeVec quantization level 1
pub const EDGEVEC_QUANTIZATION_LEVEL_1: u32 = 16;

/// EdgeVec quantization level 2
pub const EDGEVEC_QUANTIZATION_LEVEL_2: u32 = 32;

/// EdgeVec quantization level 3
pub const EDGEVEC_QUANTIZATION_LEVEL_3: u32 = 64;

/// Milvus retry delay base (milliseconds)
pub const MILVUS_RETRY_DELAY_BASE_MS: u64 = 1000;

// ============================================================================
// Embedding Provider Token Limits
// ============================================================================

/// OpenAI embedding max tokens
pub const EMBEDDING_MAX_TOKENS_OPENAI: usize = 8192;

/// VoyageAI embedding max tokens
pub const EMBEDDING_MAX_TOKENS_VOYAGEAI: usize = 1024;

/// Ollama mxbai-embed-large max tokens
pub const EMBEDDING_MAX_TOKENS_OLLAMA_MXBAI: usize = 1024;

/// Ollama nomic-embed-text max tokens
pub const EMBEDDING_MAX_TOKENS_OLLAMA_NOMIC: usize = 8192;

/// Ollama all-minilm max tokens
pub const EMBEDDING_MAX_TOKENS_OLLAMA_MINILM: usize = 512;

/// Ollama snowflake-arctic-embed max tokens
pub const EMBEDDING_MAX_TOKENS_OLLAMA_ARCTIC: usize = 512;

/// Ollama default max tokens
pub const EMBEDDING_MAX_TOKENS_OLLAMA_DEFAULT: usize = 8192;

/// FastEmbed max tokens
pub const EMBEDDING_MAX_TOKENS_FASTEMBED: usize = 512;

/// Gemini embedding max tokens
pub const EMBEDDING_MAX_TOKENS_GEMINI: usize = 8192;

// ============================================================================
// Embedding Provider Dimensions
// ============================================================================

/// OpenAI text-embedding-3-small dimension
pub const EMBEDDING_DIMENSION_OPENAI_SMALL: usize = 1536;

/// OpenAI text-embedding-3-large dimension
pub const EMBEDDING_DIMENSION_OPENAI_LARGE: usize = 3072;

/// OpenAI text-embedding-ada-002 dimension
pub const EMBEDDING_DIMENSION_OPENAI_ADA: usize = 1536;

/// VoyageAI default embedding dimension
pub const EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT: usize = 1024;

/// VoyageAI voyage-code-3 dimension
pub const EMBEDDING_DIMENSION_VOYAGEAI_CODE: usize = 1024;

/// Ollama nomic-embed-text dimension
pub const EMBEDDING_DIMENSION_OLLAMA_NOMIC: usize = 768;

/// Ollama all-minilm dimension
pub const EMBEDDING_DIMENSION_OLLAMA_MINILM: usize = 384;

/// Ollama mxbai-embed-large dimension
pub const EMBEDDING_DIMENSION_OLLAMA_MXBAI: usize = 1024;

/// Ollama snowflake-arctic-embed dimension
pub const EMBEDDING_DIMENSION_OLLAMA_ARCTIC: usize = 768;

/// Ollama default embedding dimension (fallback)
pub const EMBEDDING_DIMENSION_OLLAMA_DEFAULT: usize = 768;

/// FastEmbed default embedding dimension (AllMiniLML6V2)
pub const EMBEDDING_DIMENSION_FASTEMBED_DEFAULT: usize = 384;

/// Null provider default embedding dimension (dummy)
pub const EMBEDDING_DIMENSION_NULL: usize = 384;

// ============================================================================
// Indexing Configuration
// ============================================================================

/// Batch size for concurrent file processing
pub const INDEXING_BATCH_SIZE: usize = 10;

/// Minimum chunk content length (characters) for indexing
pub const INDEXING_CHUNK_MIN_LENGTH: usize = 25;

/// Minimum number of lines in a chunk for indexing
pub const INDEXING_CHUNK_MIN_LINES: usize = 2;

/// Maximum chunks per file to avoid explosion
pub const INDEXING_CHUNKS_MAX_PER_FILE: usize = 50;

/// Admin indexing default chunk size
pub const ADMIN_INDEXING_CHUNK_SIZE: usize = 1000;

/// Admin indexing max file size (10MB)
pub const ADMIN_INDEXING_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Admin max pool size validation
pub const ADMIN_POOL_SIZE_MAX: usize = 100;

// ============================================================================
// Event Bus Configuration
// ============================================================================

/// Tokio channel capacity for event bus
pub const EVENT_BUS_TOKIO_CAPACITY: usize = 100;

/// NATS stream max age (1 hour)
pub const NATS_STREAM_MAX_AGE: Duration = Duration::from_secs(3600);

/// NATS stream max messages
pub const NATS_STREAM_MAX_MSGS: i64 = 10000;

/// NATS consumer ack wait time (30 seconds)
pub const NATS_CONSUMER_ACK_WAIT: Duration = Duration::from_secs(30);

/// NATS consumer max delivery attempts
pub const NATS_CONSUMER_MAX_DELIVER: u32 = 10;

// ============================================================================
// Authentication Configuration
// ============================================================================

/// Minimum JWT secret length
pub const JWT_SECRET_MIN_LENGTH: usize = 32;

/// Stricter JWT secret minimum length
pub const JWT_SECRET_MIN_LENGTH_STRICT: usize = 64;

/// Minimum password length
pub const PASSWORD_MIN_LENGTH: usize = 8;

// ============================================================================
// Connection Management
// ============================================================================

/// Maximum tracked connections
pub const MAX_TRACKED_CONNECTIONS: u32 = 10000;

/// Connection polling interval (100ms)
pub const CONNECTION_POLLING_INTERVAL: Duration = Duration::from_millis(100);

// ============================================================================
// Cache Configuration
// ============================================================================

/// Default cache TTL (1 minute)
pub const CACHE_DEFAULT_TTL: Duration = Duration::from_secs(60);

/// Queue cache TTL (24 hours)
pub const CACHE_QUEUE_TTL: Duration = Duration::from_secs(86400);

/// Redis pool timeout (5 seconds)
pub const REDIS_POOL_TIMEOUT: Duration = Duration::from_secs(5);

/// Cache entry cleanup interval (10 seconds)
pub const CACHE_CLEANUP_INTERVAL: Duration = Duration::from_secs(10);

// ============================================================================
// Daemon Configuration
// ============================================================================

/// Max lock age (seconds, 5 minutes)
pub const DAEMON_MAX_LOCK_AGE_SECS: u64 = 300;

/// Sync manager cleanup retention (24 hours)
pub const SYNC_CLEANUP_RETENTION: Duration = Duration::from_secs(86400);

// ============================================================================
// Security Configuration
// ============================================================================

/// HSTS max age (seconds, 1 year/365 days)
pub const SECURITY_HSTS_MAX_AGE: u64 = 31_536_000;

/// Max request size (10MB)
pub const SECURITY_MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024;

// ============================================================================
// Crypto Configuration
// ============================================================================

/// Key rotation interval (days)
pub const CRYPTO_KEY_ROTATION_DAYS: u32 = 90;

/// AES-256 key size (bytes)
pub const CRYPTO_AES256_KEY_SIZE: usize = 32;

// ============================================================================
// Binary Watcher Configuration
// ============================================================================

/// Binary watcher debounce duration (3 seconds)
pub const BINARY_WATCHER_DEBOUNCE: Duration = Duration::from_secs(3);

/// Binary watcher sleep interval (500ms)
pub const BINARY_WATCHER_SLEEP_INTERVAL: Duration = Duration::from_millis(500);

// ============================================================================
// Server Configuration
// ============================================================================

/// Session resumption buffer size
pub const SERVER_SESSION_RESUMPTION_BUFFER_SIZE: usize = 100;

/// Max sessions
pub const SERVER_MAX_SESSIONS: usize = 1000;

/// Session cleanup interval (1 minute)
pub const SERVER_SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

// ============================================================================
// Recovery Configuration
// ============================================================================

/// Base retry delay (milliseconds)
pub const RECOVERY_BASE_DELAY_MS: u64 = 1000;

/// Max retry delay (milliseconds)
pub const RECOVERY_MAX_DELAY_MS: u64 = 30000;

/// Exponential backoff multiplier
pub const RECOVERY_BACKOFF_MULTIPLIER: f64 = 2.0;

// ============================================================================
// Backup Configuration
// ============================================================================

/// Backup channel capacity
pub const BACKUP_CHANNEL_CAPACITY: usize = 100;

// ============================================================================
// String Validation
// ============================================================================

/// String min length for validation
pub const STRING_MIN_LENGTH: usize = 1;

/// String max length for validation
pub const STRING_MAX_LENGTH: usize = 100;

// ============================================================================
// Provider Default URLs
// ============================================================================

/// Default Ollama API base URL
///
/// Ollama runs locally on port 11434 by default. This default is used
/// when no explicit URL is provided in configuration.
pub const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
