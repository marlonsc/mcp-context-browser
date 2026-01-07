//! Enterprise AI Assistant Integration Server
//!
//! MCP Context Browser provides the business logic interface between AI assistants
//! (like Claude Desktop) and semantic code search capabilities. This server transforms
//! natural language queries into precise code discoveries, enabling development teams
//! to accelerate from hours of manual code search to seconds of AI-powered insights.
//!
//! ## Business Value Delivered
//!
//! - **Accelerated Development**: Reduces code search time from 30-60 minutes to <30 seconds
//! - **Enhanced Team Productivity**: Enables developers to focus on building rather than searching
//! - **Knowledge Democratization**: Makes complex business logic accessible through natural language
//! - **Enterprise Integration**: Standardized MCP protocol ensures compatibility with leading AI assistants
//!
//! ## Core Business Capabilities
//!
//! - **Intelligent Codebase Ingestion**: AST-based analysis transforms raw code into searchable intelligence
//! - **Semantic Search Engine**: Natural language queries return contextually relevant code results
//! - **Real-time Synchronization**: Automatic background updates keep search results current
//! - **Enterprise Security**: JWT authentication and access controls for business compliance
//!
//! ## Architecture Connections
//!
//! This server orchestrates the business workflow between:
//! - **AI Providers**: OpenAI, Ollama, Gemini for semantic understanding
//! - **Vector Stores**: Milvus, Filesystem for enterprise-scale persistence
//! - **Business Services**: Context, indexing, and search services for code intelligence
//! - **AI Assistants**: MCP protocol integration for seamless collaboration

// Module declarations
pub mod args;
pub mod auth;
pub mod formatter;
pub mod handlers;
pub mod init;
pub mod rate_limit_middleware;
pub mod security;
pub mod server;

// Re-exports for public API
pub use args::*;
pub use auth::AuthHandler;
pub use formatter::ResponseFormatter;
pub use handlers::*;
pub use init::run_server;
pub use server::McpServer;