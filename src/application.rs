//! Core Business Services - Enterprise Code Intelligence
//!
//! This module contains the business services that power the semantic code search
//! platform. Each service encapsulates specific business capabilities that transform
//! raw code into actionable intelligence for development teams.

pub mod context;
pub mod indexing;
pub mod search;

// Re-export services from their respective modules
pub use context::{ContextService, GenericContextService, RepositoryContextService};
pub use indexing::IndexingService;
pub use search::SearchService;
