//! Embedding provider implementations

pub mod fastembed;
pub mod gemini;
pub mod null;
pub mod ollama;
pub mod openai;
pub mod voyageai;

// Re-export for convenience
pub use fastembed::FastEmbedProvider;
pub use gemini::GeminiEmbeddingProvider;
pub use null::NullEmbeddingProvider;
pub use ollama::OllamaEmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider;
pub use voyageai::VoyageAIEmbeddingProvider;

// Re-export MockEmbeddingProvider from the old location for backward compatibility
pub use null::NullEmbeddingProvider as MockEmbeddingProvider;
