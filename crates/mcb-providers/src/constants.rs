//! Provider Constants
//!
//! Constants specific to provider implementations. These are separated from
//! domain constants (which live in mcb-domain) and infrastructure constants.

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
// EMBEDDING API CONSTANTS
// ============================================================================

/// VoyageAI max input tokens
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// VoyageAI max output tokens
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// OpenAI max tokens per request
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Ollama server default port
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

// ============================================================================
// CACHE PROVIDER CONSTANTS
// ============================================================================

/// Default cache TTL in seconds (1 hour)
pub const CACHE_DEFAULT_TTL_SECS: u64 = 3600;

/// Default cache size limit in bytes (100MB)
pub const CACHE_DEFAULT_SIZE_LIMIT: usize = 100 * 1024 * 1024;

/// Redis connection pool size
pub const REDIS_POOL_SIZE: usize = 10;

/// Redis default port
pub const REDIS_DEFAULT_PORT: u16 = 6379;

/// Cache namespace separator
pub const CACHE_NAMESPACE_SEPARATOR: &str = ":";

// ============================================================================
// EVENTS PROVIDER CONSTANTS
// ============================================================================

/// NATS default connection timeout in seconds
pub const NATS_CONNECT_TIMEOUT_SECS: u64 = 10;

/// NATS default request timeout in seconds
pub const NATS_REQUEST_TIMEOUT_SECS: u64 = 5;

/// Event bus buffer size
pub const EVENT_BUS_BUFFER_SIZE: usize = 1000;

// ============================================================================
// VECTOR STORE PROVIDER CONSTANTS
// ============================================================================

/// AES-GCM key size in bytes
pub const AES_GCM_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// Encrypted data padding alignment
pub const ENCRYPTED_DATA_PADDING: usize = 256;

// ============================================================================
// LANGUAGE PROVIDER CONSTANTS
// ============================================================================

/// Default max chunk size (lines)
pub const LANGUAGE_DEFAULT_MAX_CHUNK_SIZE: usize = 50;

/// Maximum chunks per file
pub const LANGUAGE_MAX_CHUNKS_PER_FILE: usize = 75;

/// Priority threshold for chunk filtering
pub const LANGUAGE_PRIORITY_THRESHOLD: usize = 50;

// ============================================================================
// HTTP CONSTANTS
// ============================================================================

/// JSON content type
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// HTTP request timeout in seconds (for embedding API calls)
pub const HTTP_REQUEST_TIMEOUT_SECS: u64 = 30;

/// HTTP client maximum idle connections per host
pub const HTTP_MAX_IDLE_PER_HOST: usize = 10;

/// HTTP client idle connection timeout in seconds
pub const HTTP_CLIENT_IDLE_TIMEOUT_SECS: u64 = 90;

/// HTTP TCP keep-alive interval in seconds
pub const HTTP_KEEPALIVE_SECS: u64 = 60;

/// HTTP request timeout error message template
pub const ERROR_MSG_REQUEST_TIMEOUT: &str = "Request timed out after {:?}";

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

/// Tree-sitter node type: interface declaration (JS, Java, C#, Swift, Kotlin)
pub const AST_NODE_INTERFACE_DECLARATION: &str = "interface_declaration";

/// Tree-sitter node type: struct specifier (C/C++)
pub const AST_NODE_STRUCT_SPECIFIER: &str = "struct_specifier";

// ============================================================================
// LANGUAGE IDENTIFIERS
// ============================================================================

/// JavaScript language identifier
pub const LANG_JAVASCRIPT: &str = "javascript";

/// TypeScript language identifier
pub const LANG_TYPESCRIPT: &str = "typescript";

/// Python language identifier
pub const LANG_PYTHON: &str = "python";

/// Rust language identifier
pub const LANG_RUST: &str = "rust";

/// Go language identifier
pub const LANG_GO: &str = "go";

/// Java language identifier
pub const LANG_JAVA: &str = "java";

/// C language identifier
pub const LANG_C: &str = "c";

/// C++ language identifier
pub const LANG_CPP: &str = "cpp";

/// C# language identifier
pub const LANG_CSHARP: &str = "csharp";

/// Ruby language identifier
pub const LANG_RUBY: &str = "ruby";

/// PHP language identifier
pub const LANG_PHP: &str = "php";

/// Swift language identifier
pub const LANG_SWIFT: &str = "swift";

/// Kotlin language identifier
pub const LANG_KOTLIN: &str = "kotlin";

// ============================================================================
// EDGEVEC VECTOR STORE CONSTANTS
// ============================================================================

/// EdgeVec HNSW M parameter (max connections per node in layers > 0)
pub const EDGEVEC_HNSW_M: u32 = 16;

/// EdgeVec HNSW M0 parameter (max connections per node in layer 0)
pub const EDGEVEC_HNSW_M0: u32 = 32;

/// EdgeVec HNSW ef_construction parameter
pub const EDGEVEC_HNSW_EF_CONSTRUCTION: u32 = 200;

/// EdgeVec HNSW ef_search parameter
pub const EDGEVEC_HNSW_EF_SEARCH: u32 = 64;

/// EdgeVec default dimensions (for OpenAI embeddings)
pub const EDGEVEC_DEFAULT_DIMENSIONS: usize = 1536;

// ============================================================================
// FILESYSTEM VECTOR STORE CONSTANTS
// ============================================================================

/// Filesystem vector store bytes per dimension (f32 = 4 bytes)
pub const FILESYSTEM_BYTES_PER_DIMENSION: usize = 4;

/// Filesystem vector store max vectors per shard
pub const FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD: usize = 10000;

/// Filesystem vector store index cache size
pub const FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE: usize = 1000;

// ============================================================================
// MILVUS VECTOR STORE CONSTANTS
// ============================================================================

/// Milvus field varchar max length
pub const MILVUS_FIELD_VARCHAR_MAX_LENGTH: i32 = 512;

/// Milvus metadata varchar max length
pub const MILVUS_METADATA_VARCHAR_MAX_LENGTH: i32 = 65535;

/// Milvus IvfFlat nlist parameter
pub const MILVUS_IVFFLAT_NLIST: u32 = 128;

/// Milvus default port
pub const MILVUS_DEFAULT_PORT: u16 = 19530;

/// Milvus default query limit for aggregation queries
pub const MILVUS_DEFAULT_QUERY_LIMIT: i64 = 10_000;

// ============================================================================
// HYBRID SEARCH CONSTANTS
// ============================================================================

/// BM25 weight in hybrid search (0.0-1.0), default 40% BM25
pub const HYBRID_SEARCH_BM25_WEIGHT: f32 = 0.4;

/// Semantic weight in hybrid search (0.0-1.0), default 60% semantic
pub const HYBRID_SEARCH_SEMANTIC_WEIGHT: f32 = 0.6;

/// BM25 k1 parameter (term frequency saturation, standard tuning value)
pub const HYBRID_SEARCH_BM25_K1: f32 = 1.2;

/// BM25 b parameter (document length normalization, standard tuning value)
pub const HYBRID_SEARCH_BM25_B: f32 = 0.75;

/// BM25 token minimum length filter (filter very short tokens)
pub const BM25_TOKEN_MIN_LENGTH: usize = 2;
