//! Domain Type Definitions
//!
//! Type aliases and basic type definitions for dynamic domain concepts.
//! These allow the domain to be extended without changing core types.


/// Programming language identifier
///
/// A string-based identifier for programming languages that allows dynamic
/// extension without modifying the domain layer. Language support is determined
/// by the application and infrastructure layers.
pub type Language = String;

/// System operation type identifier
///
/// A string-based identifier for operation types used in metrics and rate limiting.
/// Allows dynamic extension of operation types without domain changes.
pub type OperationType = String;

/// Embedding provider identifier
///
/// A string-based identifier for embedding providers that allows dynamic
/// extension without modifying the domain layer. Provider capabilities
/// are determined by the application and infrastructure layers.
pub type EmbeddingProviderKind = String;

/// Vector store provider identifier
///
/// A string-based identifier for vector store providers that allows dynamic
/// extension without modifying the domain layer. Provider capabilities
/// are determined by the application and infrastructure layers.
pub type VectorStoreProviderKind = String;