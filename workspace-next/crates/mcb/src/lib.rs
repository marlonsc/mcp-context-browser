//! # MCP Context Browser
//!
//! A Model Context Protocol server for semantic code analysis using vector embeddings.
//!
//! This crate provides the main public API for the MCP Context Browser.
//! It re-exports the domain types and will eventually provide access to
//! the full application functionality.
//!
//! ## Features
//!
//! - **Semantic Code Search**: Find code by meaning using vector embeddings
//! - **Multi-Language Support**: AST-based parsing for 12+ programming languages
//! - **Multiple Vector Stores**: Support for various vector database backends
//! - **Clean Architecture**: Domain-driven design with dependency injection
//!
//! ## Example
//!
//! ```rust
//! use mcb::domain::{CodeChunk, Language, Embedding};
//!
//! // Domain types are available through the mcb facade
//! let chunk = CodeChunk {
//!     id: "chunk-1".to_string(),
//!     content: "fn main() {}".to_string(),
//!     file_path: "example.rs".to_string(),
//!     start_line: 1,
//!     end_line: 1,
//!     language: Language::Rust,
//!     metadata: serde_json::json!({}),
//! };
//! ```
//!
//! ## Architecture
//!
//! The codebase follows Clean Architecture principles:
//!
//! - `domain` - Core business logic and types (ports, entities, domain errors)
//! - `services` - Application services and use cases
//! - `infrastructure` - External concerns (config, logging, HTTP)
//! - `adapters` - Repository implementations and external integrations
//! - `server` - MCP protocol server and API endpoints

/// Domain layer - core business logic and types
pub mod domain {
    //! Re-exports from the domain crate for convenience
    pub use mcb_domain::*;
}

// Re-export commonly used domain types at the crate root
pub use domain::*;