//! Provider configuration management module
//!
//! This module provides comprehensive management for AI and vector store providers
//! including health checking, configuration validation, and provider selection logic.

pub mod embedding;
pub mod vector_store;
mod manager;

pub use embedding::EmbeddingProviderConfig;
pub use vector_store::VectorStoreProviderConfig;
pub use manager::ProviderConfigManager;

/// Health status of a provider
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderHealth {
    /// Provider is healthy and operational
    Healthy,
    /// Provider is experiencing issues but may still work
    Degraded,
    /// Provider is completely unavailable
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Requirements for provider compatibility
#[derive(Debug, Clone)]
pub struct ProviderRequirements {
    /// Minimum dimensions required
    pub min_dimensions: Option<usize>,
    /// Maximum tokens supported
    pub max_tokens: Option<usize>,
    /// Required features
    pub required_features: Vec<String>,
}

impl Default for ProviderRequirements {
    fn default() -> Self {
        Self {
            min_dimensions: Some(128), // Minimum reasonable embedding dimension
            max_tokens: Some(512),     // Minimum token support
            required_features: vec!["embeddings".to_string()],
        }
    }
}
