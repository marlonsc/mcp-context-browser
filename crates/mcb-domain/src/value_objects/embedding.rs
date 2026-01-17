//! Semantic Embedding Value Objects
//!
//! Value objects representing semantic embeddings and related
//! concepts for similarity search and text understanding.

use serde::{Deserialize, Serialize};

/// Value Object: Semantic Text Embedding
///
/// Represents a vector embedding of text content that captures semantic meaning.
/// Embeddings enable similarity search and are the foundation of the semantic
/// search capabilities.
///
/// ## Business Rules
///
/// - Vector must contain at least one element
/// - Dimensions must be positive
/// - Model name identifies the embedding generation method
///
/// ## Example
///
/// ```rust
/// use mcb_domain::value_objects::Embedding;
///
/// let embedding = Embedding {
///     vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
///     model: "text-embedding-ada-002".to_string(),
///     dimensions: 1536,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    /// The embedding vector values
    pub vector: Vec<f32>,
    /// Name of the model that generated this embedding
    pub model: String,
    /// Dimensionality of the embedding vector
    pub dimensions: usize,
}
