// Clippy allows for complex patterns in server code
#![allow(clippy::io_other_error)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::while_let_loop)]

//! # MCP Context Browser Server
//!
//! MCP protocol server implementation for semantic code analysis using vector embeddings.
//!
//! For user guides and tutorials, see the [online documentation](https://marlonsc.github.io/mcp-context-browser/).
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
//! use mcb_server::run_server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Run with default config (XDG paths + environment)
//!     run_server(None).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate implements the transport and protocol layer for the MCP Context Browser.
//! It depends on domain contracts and infrastructure services while remaining independent
//! of specific provider implementations.
//!
//! ## Core Types
//!
//! The most important types for users:
//!
//! | Type | Description |
//! |------|-------------|
//! | [`McpServer`] | Main server struct |
//! | [`McpServerBuilder`] | Builder for server configuration |
//!
//! ## Feature Flags
//!
//! - `fastembed`: Local embeddings via FastEmbed (default)
//! - `filesystem-store`: Local filesystem vector storage (default)
//! - `milvus`: Milvus vector database support
//! - `edgevec`: EdgeVec in-memory vector store
//! - `redis-cache`: Redis distributed caching
//! - `full`: All features enabled

// Documentation configuration for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod admin;
pub mod args;
pub mod auth;
pub mod builder;
pub mod constants;
pub mod formatter;
pub mod handlers;
pub mod init;
pub mod mcp_server;
pub mod tools;
pub mod transport;

// Placeholder modules removed - functionality handled by infrastructure layer

// Re-export core types for public API
pub use builder::McpServerBuilder;
pub use init::run_server;
pub use mcp_server::McpServer;
