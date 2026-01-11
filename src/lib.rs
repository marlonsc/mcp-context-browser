//! # MCP Context Browser
//!
//! A Model Context Protocol server for semantic code analysis using vector embeddings.
//!
//! ## Features
//!
//! - **Semantic Search**: AI-powered code understanding and retrieval using vector embeddings
//! - **Multi-Provider**: Support for OpenAI, Ollama, FastEmbed, VoyageAI, Gemini embedding providers
//! - **Vector Storage**: Milvus, EdgeVec, or filesystem-based vector storage
//! - **AST Parsing**: 14 programming languages with tree-sitter based code chunking
//! - **Hybrid Search**: Combines BM25 lexical search with semantic similarity
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcp_context_browser::run_server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run_server().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate follows Clean Architecture with four layers:
//!
//! - **`domain`**: Business entities, value objects, and port traits
//! - **`application`**: Use cases and service orchestration
//! - **`adapters`**: External service implementations (providers, databases)
//! - **`infrastructure`**: Cross-cutting concerns (cache, auth, config, metrics)
//!
//! ## Modules
//!
//! - [`server`]: MCP protocol implementation and handlers
//! - [`chunking`]: AST-based code chunking for 14 languages
//! - [`adapters`]: Embedding and vector store provider implementations
//! - [`application`]: Business logic services (indexing, search, context)
//!
//! ## Feature Flags
//!
//! - `fastembed`: Local embeddings via FastEmbed (default)
//! - `filesystem-store`: Local filesystem vector storage (default)
//! - `milvus`: Milvus vector database support
//! - `edgevec`: EdgeVec in-memory vector store
//! - `redis-cache`: Redis distributed caching
//! - `full`: All features enabled

pub mod adapters;
pub mod admin;
pub mod application;
pub mod chunking;
pub mod config_example;
pub mod daemon;
pub mod domain;
pub mod infrastructure;
pub mod server;
pub mod snapshot;
pub mod sync;

// Re-export core types for public API
pub use domain::error::{Error, Result};
pub use domain::types::*;

// Re-export main entry points
pub use server::builder::McpServerBuilder;
pub use server::init::run_server;
pub use server::mcp_server::McpServer;
