//! Core Business Services - Enterprise Code Intelligence
//!
//! This module contains the business services that power the semantic code search
//! platform. Each service encapsulates specific business capabilities that transform
//! raw code into actionable intelligence for development teams.
//!
//! ## Business Value Chain
//!
//! 1. **Context Service**: Transforms code into semantic understanding through AI embeddings
//! 2. **Indexing Service**: Ingests and organizes codebases for efficient search and retrieval
//! 3. **Search Service**: Delivers natural language queries to precise code results
//!
//! ## Enterprise Architecture
//!
//! Services follow SOLID principles with clean interfaces, dependency injection,
//! and comprehensive error handling. Each service is designed for enterprise-scale
//! operations with monitoring, caching, and failover capabilities.

pub mod context;
pub mod indexing;
pub mod search;

// Re-export services from their respective modules
pub use context::{ContextService, GenericContextService, RepositoryContextService};
pub use indexing::IndexingService;
pub use search::SearchService;

// Re-export for backward compatibility
pub use crate::core::types::{CodeChunk, SearchResult};
pub use crate::di::factory::ServiceProvider;
pub use crate::providers::{EmbeddingProvider, VectorStoreProvider};
