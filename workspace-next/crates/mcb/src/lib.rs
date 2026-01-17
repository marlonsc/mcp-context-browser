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
//! ```ignore
//! use mcb::domain::CodeChunk;
//!
//! // Domain types are available through the mcb facade
//! let chunk = CodeChunk {
//!     id: "chunk-1".to_string(),
//!     content: "fn main() {}".to_string(),
//!     file_path: "example.rs".to_string(),
//!     start_line: 1,
//!     end_line: 1,
//!     language: "rust".to_string(),  // Language is a String type alias
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
///
/// Re-exports from the domain crate for convenience
pub mod domain {
    pub use mcb_domain::*;
}

/// Server layer - MCP protocol server and handlers
///
/// Re-exports from the server crate for convenience
pub mod server {
    pub use mcb_server::*;
}

/// Infrastructure layer - DI, config, and infrastructure services
///
/// Re-exports from the infrastructure crate for convenience
pub mod infrastructure {
    pub use mcb_infrastructure::*;
}

// Re-export commonly used domain types at the crate root
pub use domain::*;

// Re-export main entry point at the crate root
pub use server::run_server;

// Re-export server types for convenience
pub use server::{McpServer, McpServerBuilder};
