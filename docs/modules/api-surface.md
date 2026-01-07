# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Public Modules

### Core Library Modules
- chunking (text processing utilities)
- config (configuration management)
- core (core types and utilities)
- providers (AI provider abstractions)
- services (business logic layer)
- server (MCP protocol server)
- metrics (monitoring and observability)
- sync (cross-process coordination)
- daemon (background services)
- snapshot (change tracking)

### Public Re-exports
- Rate limiting system (core::rate_limit)
- Resource limits system (core::limits)
- Advanced caching system (core::cache)
- Hybrid search system (core::hybrid_search)
- Multi-provider routing (providers::routing)

## Public Functions

### Core Types
- Error handling and conversion functions
- Configuration validation functions
- Cache management operations

### Provider Interfaces
- EmbeddingProvider trait methods
- VectorStoreProvider trait methods
- Provider factory functions

### Service Interfaces
- ContextService::embed_text()
- IndexingService::index_codebase()
- SearchService::search()

## Public Types

### Data Structures
- Embedding (vector representation)
- CodeChunk (processed code segment)
- SearchResult (search response)
- ContextConfig (service configuration)
- ProviderConfig (provider settings)

### Enums
- Error (comprehensive error types)
- ProviderType (available providers)
- IndexStatus (indexing progress)

## API Stability

### Current Status
- **Version**: 0.0.4 (Documentation Excellence)
- **Stability**: Experimental - APIs may change
- **Compatibility**: Breaking changes expected until 1.0.0

### Public API Commitments
- MCP protocol interface stability
- Core semantic search functionality
- Provider abstraction interfaces

*Generated automatically on: 2026-01-07 21:25:37 UTC*
