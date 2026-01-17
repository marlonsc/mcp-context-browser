# domain Module

**Source**: `crates/mcb-domain/src/`
**Crate**: `mcb-domain`
**Files**: 15+ (port files + core files)
**Lines of Code**: ~3,000
**Traits**: 14+
**Structs**: 25+
**Enums**: 8

## Overview

The domain module defines the core business entities and port interfaces following Clean Architecture principles. All domain logic is technology-agnostic, with external concerns abstracted behind port traits.

## Key Exports

### Port Traits (14+ total)

All traits extend `shaku::Interface` for DI compatibility:

**Provider Ports (`ports/providers/`):**
-   `EmbeddingProvider` - Text-to-vector conversion
-   `VectorStoreProvider` - Vector storage and retrieval
-   `CacheProvider` - Caching operations
-   `CryptoProvider` - Encryption and hashing

**Infrastructure Ports (`ports/infrastructure/`):**
-   `SyncProvider` - Low-level sync operations
-   `SnapshotProvider` - Codebase snapshot management
-   `StateStoreProvider` - State persistence
-   `LockProvider` - Distributed locking
-   `EventPublisher` - Domain event publishing

**Admin Ports (`ports/admin.rs`):**
-   `PerformanceMetricsInterface` - Performance metrics tracking
-   `IndexingOperationsInterface` - Indexing operations

**Repository Ports (`repositories/`):**
-   `ChunkRepository` - Code chunk persistence
-   `SearchRepository` - Search operations

**Service Ports (in `mcb-application`):**
-   `ContextServiceInterface` - High-level code intelligence
-   `SearchServiceInterface` - Semantic search
-   `IndexingServiceInterface` - Codebase indexing
-   `ChunkingOrchestratorInterface` - Batch chunking coordination

### Core Types
-   `CodeChunk` - Semantic code unit
-   `Embedding` - Vector representation
-   `SearchResult` - Search result with score
-   `Language` - Programming language enum

### Events
-   `DomainEvent` - Domain-level events (IndexRebuild, SyncCompleted, etc.)

## File Structure

```text
crates/mcb-domain/src/
├── ports/
│   ├── providers/           # Provider port traits
│   │   ├── embedding.rs     # EmbeddingProvider trait
│   │   ├── vector_store.rs  # VectorStoreProvider trait
│   │   ├── cache.rs         # CacheProvider trait
│   │   └── crypto.rs        # CryptoProvider trait
│   ├── infrastructure/      # Infrastructure port traits
│   │   ├── sync.rs          # SyncProvider, LockProvider traits
│   │   ├── snapshot.rs      # SnapshotProvider, StateStoreProvider traits
│   │   └── events.rs        # EventPublisher trait
│   ├── admin.rs             # Admin service interfaces
│   └── mod.rs               # Re-exports
├── entities/                # Domain entities
├── value_objects/           # Value objects
├── repositories/            # Repository port traits
│   ├── chunk_repository.rs  # ChunkRepository trait
│   └── search_repository.rs # SearchRepository trait
├── error.rs                 # Domain error types
├── types.rs                 # Core domain types
└── lib.rs                   # Module exports
```

## Port/Adapter Mappings

| Port | Implementation | Location |
|------|---------------|----------|
| `EmbeddingProvider` | OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null | `crates/mcb-providers/src/embedding/` |
| `VectorStoreProvider` | Memory, Encrypted, Null | `crates/mcb-providers/src/vector_store/` |
| `CacheProvider` | Moka, Redis | `crates/mcb-providers/src/cache/` |
| `EventPublisher` | NullEventBus | `crates/mcb-infrastructure/src/adapters/infrastructure/` |
| `SyncProvider` | NullSyncProvider | `crates/mcb-infrastructure/src/adapters/infrastructure/` |
| `SnapshotProvider` | NullSnapshotProvider | `crates/mcb-infrastructure/src/adapters/infrastructure/` |
| `ChunkRepository` | NullChunkRepository | `crates/mcb-infrastructure/src/adapters/repository/` |
| `SearchRepository` | NullSearchRepository | `crates/mcb-infrastructure/src/adapters/repository/` |
| `ContextServiceInterface` | ContextService | `crates/mcb-application/src/services/` |
| `SearchServiceInterface` | SearchService | `crates/mcb-application/src/services/` |
| `IndexingServiceInterface` | IndexingService | `crates/mcb-application/src/services/` |
| `ChunkingOrchestratorInterface` | ChunkingOrchestrator | `crates/mcb-application/src/domain_services/` |

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
