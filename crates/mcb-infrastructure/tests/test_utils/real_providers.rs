//! Real Provider Test Utilities
//!
//! Provides factory functions for creating real (not mocked) provider instances
//! for integration testing. Uses NullEmbeddingProvider and InMemoryVectorStoreProvider
//! which are actual implementations, not mocks.
//!
//! ## Key Principle
//!
//! Tests should use real providers, not mocks, to validate actual behavior.
//! - `NullEmbeddingProvider`: Deterministic hash-based embeddings (no external deps)
//! - `InMemoryVectorStoreProvider`: Real in-memory storage with actual search
//! - `NullCacheProvider`: Real no-op cache (not a mock)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::test_utils::real_providers::*;
//!
//! #[tokio::test]
//! async fn test_with_real_providers() {
//!     let ctx = create_test_app_context().await.unwrap();
//!     let embedding = ctx.embedding_handle().get();
//!     let results = embedding.embed_batch(&["test".into()]).await.unwrap();
//!     assert_eq!(results[0].dimensions, 384);
//! }
//! ```

// Force linkme registration of all providers from mcb-providers
extern crate mcb_providers;

use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_domain::value_objects::{Embedding, SearchResult};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Create a test AppContext with real providers
///
/// Uses NullEmbeddingProvider and InMemoryVectorStoreProvider (or NullVectorStore
/// depending on config). These are real implementations, not mocks.
///
/// # Important
///
/// The `extern crate mcb_providers` at the top of this module forces linkme
/// to register all providers. Without it, the registry would be empty.
pub async fn create_test_app_context() -> Result<AppContext> {
    let config = AppConfig::default();
    init_app(config).await
}

/// Create AppContext with custom configuration
pub async fn create_test_app_context_with_config(config: AppConfig) -> Result<AppContext> {
    init_app(config).await
}

/// Context for full-stack integration tests
///
/// Provides convenient access to providers resolved through the DI container.
pub struct FullStackTestContext {
    app_context: AppContext,
}

impl FullStackTestContext {
    /// Create new full-stack test context
    pub async fn new() -> Result<Self> {
        let app_context = create_test_app_context().await?;
        Ok(Self { app_context })
    }

    /// Get the underlying AppContext
    pub fn app_context(&self) -> &AppContext {
        &self.app_context
    }

    /// Get embedding provider (via DI handle)
    pub fn embedding(&self) -> Arc<dyn EmbeddingProvider> {
        self.app_context.embedding_handle().get()
    }

    /// Get vector store provider (via DI handle)
    pub fn vector_store(&self) -> Arc<dyn VectorStoreProvider> {
        self.app_context.vector_store_handle().get()
    }

    /// Embed texts using the real embedding provider
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding().embed_batch(texts).await
    }

    /// Create collection in vector store
    pub async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        self.vector_store()
            .create_collection(name, dimensions)
            .await
    }

    /// Index chunks with embeddings
    ///
    /// This exercises the real embedding → vector store flow.
    pub async fn index_chunks(
        &self,
        collection: &str,
        chunks: &[CodeChunk],
    ) -> Result<Vec<String>> {
        // Extract texts for embedding
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();

        // Generate real embeddings (NullEmbeddingProvider uses deterministic hash)
        let embeddings = self.embed_texts(&texts).await?;

        // Build metadata from chunks
        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut meta = HashMap::new();
                meta.insert("id".to_string(), serde_json::json!(chunk.id));
                meta.insert("file_path".to_string(), serde_json::json!(chunk.file_path));
                meta.insert("content".to_string(), serde_json::json!(chunk.content));
                meta.insert(
                    "start_line".to_string(),
                    serde_json::json!(chunk.start_line),
                );
                meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
                meta.insert("language".to_string(), serde_json::json!(chunk.language));
                meta
            })
            .collect();

        // Insert into vector store - returns Vec<String> of IDs
        self.vector_store()
            .insert_vectors(collection, &embeddings, metadata)
            .await
    }

    /// Search with real embedding and vector search
    pub async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<SearchResult> {
        // Embed query
        let query_embeddings = self.embed_texts(&[query.to_string()]).await?;
        let query_embedding = &query_embeddings[0];

        // Search vector store using search_similar
        self.vector_store()
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await
            .map(|results| {
                results.into_iter().next().unwrap_or_else(|| SearchResult {
                    id: String::new(),
                    file_path: String::new(),
                    content: String::new(),
                    start_line: 0,
                    score: 0.0,
                    language: String::new(),
                })
            })
    }

    /// Search and return all results
    pub async fn search_all(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Embed query
        let query_embeddings = self.embed_texts(&[query.to_string()]).await?;
        let query_embedding = &query_embeddings[0];

        // Search vector store using search_similar
        self.vector_store()
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await
    }
}

/// Create test code chunks for integration testing
///
/// Returns realistic Rust code chunks that can be indexed and searched.
pub fn create_test_code_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "chunk_1".to_string(),
            file_path: "src/config.rs".to_string(),
            content: r#"#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            database_url: "postgres://localhost/db".to_string(),
        }
    }
}"#
            .to_string(),
            start_line: 1,
            end_line: 15,
            language: "rust".to_string(),
            metadata: json!({"type": "struct", "name": "Config"}),
        },
        CodeChunk {
            id: "chunk_2".to_string(),
            file_path: "src/auth.rs".to_string(),
            content: r#"pub async fn authenticate(token: &str) -> Result<User, AuthError> {
    let claims = verify_jwt(token)?;
    let user = User::from_claims(claims);
    Ok(user)
}

pub fn verify_jwt(token: &str) -> Result<Claims, AuthError> {
    // JWT verification logic
    todo!()
}"#
            .to_string(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "authenticate"}),
        },
        CodeChunk {
            id: "chunk_3".to_string(),
            file_path: "src/handlers.rs".to_string(),
            content: r#"pub async fn handle_request(req: Request) -> Response {
    let config = Config::new();
    let result = process_data(&req, &config).await?;
    Response::ok(result)
}

async fn process_data(req: &Request, config: &Config) -> Result<Data, Error> {
    // Data processing logic
    Ok(Data::default())
}"#
            .to_string(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "handle_request"}),
        },
        CodeChunk {
            id: "chunk_4".to_string(),
            file_path: "src/main.rs".to_string(),
            content: r#"#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Starting server on {}:{}", config.host, config.port);

    // Initialize server
    let server = Server::bind(&config.host, config.port);
    server.run().await?;

    Ok(())
}"#
            .to_string(),
            start_line: 1,
            end_line: 11,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "main"}),
        },
    ]
}

/// Create a single test chunk with custom content
pub fn create_test_chunk(
    id: &str,
    file_path: &str,
    content: &str,
    start_line: u32,
    end_line: u32,
    language: &str,
) -> CodeChunk {
    CodeChunk {
        id: id.to_string(),
        file_path: file_path.to_string(),
        content: content.to_string(),
        start_line,
        end_line,
        language: language.to_string(),
        metadata: json!({"type": "custom"}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_app_context() {
        let result = create_test_app_context().await;
        assert!(result.is_ok(), "Should create AppContext successfully");

        let ctx = result.expect("AppContext should be valid");

        // Verify providers are resolved
        let embedding = ctx.embedding_handle().get();
        assert_eq!(
            embedding.dimensions(),
            384,
            "Null provider has 384 dimensions"
        );
        assert_eq!(
            embedding.provider_name(),
            "null",
            "Should use null provider"
        );
    }

    #[tokio::test]
    async fn test_full_stack_context_embeds_text() {
        let ctx = FullStackTestContext::new()
            .await
            .expect("Context should create");

        let texts = vec!["test query".to_string()];
        let embeddings = ctx.embed_texts(&texts).await.expect("Should embed");

        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].dimensions, 384);
        assert_eq!(embeddings[0].vector.len(), 384);
    }

    #[tokio::test]
    async fn test_full_stack_context_indexes_and_searches() {
        let ctx = FullStackTestContext::new()
            .await
            .expect("Context should create");

        // Create collection
        ctx.create_collection("test_collection", 384)
            .await
            .expect("Should create collection");

        // Index test chunks
        let chunks = create_test_code_chunks();
        let ids = ctx
            .index_chunks("test_collection", &chunks)
            .await
            .expect("Should index chunks");

        // Should have indexed all chunks
        assert_eq!(ids.len(), chunks.len(), "Should index all chunks");

        // Search for config-related code
        let results = ctx
            .search_all("test_collection", "configuration settings", 5)
            .await
            .expect("Should search");

        // Should find results (even with deterministic embeddings)
        assert!(!results.is_empty(), "Should find at least one result");
    }

    #[test]
    fn test_create_test_code_chunks() {
        let chunks = create_test_code_chunks();
        assert_eq!(chunks.len(), 4);

        // Verify each chunk has required fields
        for chunk in &chunks {
            assert!(!chunk.id.is_empty());
            assert!(!chunk.file_path.is_empty());
            assert!(!chunk.content.is_empty());
            assert!(!chunk.language.is_empty());
            assert!(chunk.end_line >= chunk.start_line);
        }
    }
}
