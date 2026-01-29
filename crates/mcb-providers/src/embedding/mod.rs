//! Embedding Provider Implementations
//!
//! Converts text into dense vector embeddings for semantic search.
//! Each provider offers different tradeoffs between quality, cost, and privacy.
//!
//! ## Available Providers
//!
//! | Provider | Type | Status |
//! |----------|------|--------|
//! | NullEmbeddingProvider | Testing | Complete |
//! | OllamaEmbeddingProvider | Local | Complete |
//! | OpenAIEmbeddingProvider | Cloud | Complete |
//! | VoyageAIEmbeddingProvider | Cloud | Complete |
//! | GeminiEmbeddingProvider | Cloud | Complete |
//! | FastEmbedProvider | Local ML | Complete (optional) |
//!
//! ## Provider Selection Guide
//!
//! ### Development/Testing
//! - **Default**: Use `NullEmbeddingProvider` for unit tests
//!
//! ### Local/Privacy-First
//! - **Ollama**: Local LLM server with embedding models
//! - **FastEmbed**: Pure local ONNX inference (requires `embedding-fastembed` feature)
//!
//! ### Cloud/Production
//! - **OpenAI**: High quality, widely adopted
//! - **VoyageAI**: Optimized for code embeddings
//! - **Gemini**: Google ecosystem integration

#[cfg(feature = "embedding-fastembed")]
pub mod fastembed;
pub mod gemini;
pub mod helpers;
pub mod null;
pub mod ollama;
pub mod openai;
pub mod voyageai;

// Re-export for convenience
#[cfg(feature = "embedding-fastembed")]
pub use fastembed::FastEmbedProvider;
pub use gemini::GeminiEmbeddingProvider;
pub use helpers::constructor;
pub use null::NullEmbeddingProvider;
pub use ollama::OllamaEmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider;
pub use voyageai::VoyageAIEmbeddingProvider;
