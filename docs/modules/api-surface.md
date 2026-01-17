# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Crate Public APIs

### mcb (Facade Crate)

Re-exports from all internal crates for unified access:

```rust
// Domain types
pub use mcb_domain::{CodeChunk, Embedding, SearchResult, Language, Error, Result};

// Service interfaces
pub use mcb_domain::ports::{EmbeddingProvider, VectorStoreProvider, CacheProvider};

// Service implementations
pub use mcb_application::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};

// Server
pub use mcb_server::{McpServer, McpServerBuilder, run_server};

// Configuration
pub use mcb_infrastructure::{AppConfig, ServerConfig};
```

### mcb-domain

Core types and port traits:

-   **Types**: `CodeChunk`, `Embedding`, `SearchResult`, `Language`
-   **Errors**: `Error`, `Result<T>`
-   **Ports**: 14+ trait interfaces

### mcb-application

Business logic services:

-   `ContextServiceImpl`
-   `IndexingServiceImpl`
-   `SearchServiceImpl`
-   `ChunkingOrchestrator`

### mcb-server

HTTP server and MCP protocol:

-   `McpServer` - MCP protocol handler
-   `McpServerBuilder` - Server builder pattern
-   `run_server` - Entry point function

### mcb-providers

External integrations:

-   6 embedding providers
-   3 vector store providers
-   2 cache providers
-   12 language processors

### mcb-infrastructure

Configuration and DI:

-   `AppConfig`, `ServerConfig`, `AuthConfig`
-   `McpModule` - DI container
-   Null adapters for testing

### mcb-validate

Architecture validation (internal tooling):

-   12 validators
-   Architecture report generation

## API Stability

### Current Status

-   **Version**: 0.1.1 (First Stable Release)
-   **Stability**: Stable for documented APIs
-   **Compatibility**: Semantic versioning from v0.1.0+

### Public API Commitments

-   MCP protocol interface stability
-   Core semantic search functionality
-   Provider abstraction interfaces
-   Configuration structure

### Breaking Change Policy

-   Minor versions (0.x.0): May include breaking changes
-   Patch versions (0.x.y): Bug fixes only
-   Major version (1.0.0+): Stable API with deprecation cycles

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
