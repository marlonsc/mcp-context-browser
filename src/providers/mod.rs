//! Enterprise AI & Storage Provider Ecosystem
//!
//! This module defines the business interfaces and implementations for AI and storage
//! providers that power the semantic code search platform. The provider architecture
//! enables enterprise flexibility, allowing organizations to choose optimal AI services
//! and storage backends based on their specific business requirements.
//!
//! ## Business Value Delivered
//!
//! - **AI Provider Flexibility**: Support for leading AI models (OpenAI, Ollama, Gemini, VoyageAI)
//! - **Storage Backend Options**: Multiple storage solutions for different deployment scenarios
//! - **Enterprise Integration**: Seamless integration with corporate AI infrastructure
//! - **Cost Optimization**: Intelligent provider selection based on performance and cost
//! - **Business Continuity**: Automatic failover and health monitoring across providers
//!
//! ## Architecture Benefits
//!
//! - **Scalability**: Provider abstraction enables horizontal scaling across different services
//! - **Cost Efficiency**: Route requests to optimal providers based on business requirements
//! - **Reliability**: Circuit breakers and health checks prevent cascading failures
//! - **Future-Proof**: Clean interfaces enable easy integration of new AI and storage providers

use crate::core::{error::Result, types::Embedding};
use async_trait::async_trait;

/// AI Semantic Understanding Interface
///
/// Defines the business contract for AI providers that transform text into
/// semantic embeddings. This abstraction enables the platform to work with
/// any AI service that can understand code semantics, from enterprise OpenAI
/// deployments to self-hosted Ollama instances.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation provided)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple embed operation
        self.embed("health check").await?;
        Ok(())
    }
}

/// Enterprise Vector Storage Interface
///
/// Defines the business contract for vector storage systems that persist and
/// retrieve semantic embeddings at enterprise scale. This abstraction supports
/// multiple storage backends from in-memory development stores to production
/// Milvus clusters, ensuring optimal performance for different business needs.
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>>;
    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<crate::core::types::SearchResult>>;
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;
    async fn get_stats(
        &self,
        collection: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>>;
    async fn flush(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple operation
        self.collection_exists("__health_check__").await?;
        Ok(())
    }
}

// Submodules
pub mod embedding;
pub mod routing;
pub mod vector_store;

// Re-export implementations
pub use embedding::NullEmbeddingProvider as MockEmbeddingProvider; // Backward compatibility
pub use embedding::OllamaEmbeddingProvider;
pub use embedding::OpenAIEmbeddingProvider;
pub use vector_store::InMemoryVectorStoreProvider;

// Re-export routing system
pub use routing::{
    ProviderContext, ProviderRouter, ProviderSelectionStrategy, circuit_breaker::CircuitBreaker,
    metrics::ProviderMetricsCollector,
};
