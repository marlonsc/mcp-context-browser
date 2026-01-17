//! Use Cases - Application Layer Services
//!
//! This module contains the use case implementations that orchestrate
//! business logic and coordinate between domain entities and external ports.
//!
//! ## Use Cases Implemented
//!
//! - `context_service`: Code intelligence and semantic operations
//! - `search_service`: Semantic search operations
//! - `indexing_service`: Code indexing and ingestion operations
//!
//! ## Dependency Injection
//!
//! All use cases are designed to work with dependency injection via Shaku.
//! They receive their dependencies (ports) through constructor injection.

pub mod context_service;
pub mod search_service;
pub mod indexing_service;

pub use context_service::*;
pub use search_service::*;
pub use indexing_service::*;