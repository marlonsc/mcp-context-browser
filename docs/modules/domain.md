# domain Module

**Source**: `crates/mcb-domain/src/`
**Crate**: `mcb-domain`
**Files**: 15+
**Lines of Code**: ~1,500
**Traits**: 3 (repository interfaces)
**Structs**: 10+
**Enums**: 5

## Overview

The domain module defines the core business entities, value objects, and repository interfaces following Clean Architecture principles. All domain logic is technology-agnostic, with external concerns abstracted behind port traits.

> **Note**: Port traits (EmbeddingProvider, VectorStoreProvider, etc.) are defined in `mcb-application/src/ports/`, not in mcb-domain. The domain layer contains only entities, value objects, and repository interfaces.

## Key Exports

### Repository Interfaces (`repositories/`)

-   `ChunkRepository` - Code chunk persistence (`Send + Sync`; DI via dill, ADR-029)
-   `SearchRepository` - Search operations (`Send + Sync`; DI via dill, ADR-029)

### Domain Events (`events/`)

-   `DomainEvent` - Base event trait
-   `EventPublisher` - Event publishing interface
-   `ServiceState` - Service lifecycle states

### Entities (`entities/`)

-   `CodeChunk` - Parsed code segment with metadata
-   `Codebase` - Repository metadata

### Value Objects (`value_objects/`)

-   `Embedding` - Vector representation with metadata
-   `SearchResult` - Ranked search results
-   Config types - Configuration value objects

## File Structure (Actual)

```text
crates/mcb-domain/src/
├── entities/               # Domain entities
│   ├── code_chunk.rs
│   ├── codebase.rs
│   └── mod.rs
├── events/                 # Domain events
│   ├── domain_events.rs
│   └── mod.rs
├── repositories/           # Repository port traits
│   ├── chunk_repository.rs
│   ├── search_repository.rs
│   └── mod.rs
├── value_objects/          # Value objects
│   ├── config.rs
│   ├── embedding.rs
│   ├── search.rs
│   ├── types.rs
│   └── mod.rs
├── constants.rs            # Domain constants
├── error.rs                # Domain error types
└── mod.rs                  # Module exports
```

## Port/Adapter Mappings

| Port (in mcb-application) | Implementation | Location |
|---------------------------|---------------|----------|
| `EmbeddingProvider` | OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null | `crates/mcb-providers/src/embedding/` |
| `VectorStoreProvider` | InMemory, Encrypted, Null | `crates/mcb-providers/src/vector_store/` |
| `CacheProvider` | Moka, Redis, Null | `crates/mcb-providers/src/cache/` |
| `EventBusProvider` | Tokio, Null | `crates/mcb-providers/src/events/` |

| Port (in mcb-domain) | Implementation | Location |
|----------------------|---------------|----------|
| `ChunkRepository` | In-memory (via providers) | `crates/mcb-providers/` |
| `SearchRepository` | In-memory (via providers) | `crates/mcb-providers/` |

---

*Updated 2026-01-18 - Reflects v0.1.2 crate architecture*
