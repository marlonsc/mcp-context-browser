# Module Structure

This document shows the hierarchical structure of modules in the MCP Context Browser.

## Crate Structure (Clean Architecture Monorepo)

```
mcp-context-browser/
├── Cargo.toml (workspace root)
├── crates/
│   ├── mcb/                          # Facade crate (re-exports public API)
│   │   └── src/lib.rs
│   │
│   ├── mcb-domain/                   # Domain layer (core business logic)
│   │   └── src/
│   │       ├── ports/                # Port traits (interfaces)
│   │       │   ├── providers/        # Provider port traits
│   │       │   │   ├── embedding.rs
│   │       │   │   ├── vector_store.rs
│   │       │   │   ├── cache.rs
│   │       │   │   └── crypto.rs
│   │       │   ├── infrastructure/   # Infrastructure port traits
│   │       │   │   ├── sync.rs
│   │       │   │   ├── snapshot.rs
│   │       │   │   └── events.rs
│   │       │   └── admin.rs          # Admin port traits
│   │       ├── entities/             # Domain entities
│   │       ├── value_objects/        # Value objects
│   │       ├── repositories/         # Repository port traits
│   │       ├── error.rs              # Domain errors
│   │       └── types.rs              # Domain types
│   │
│   ├── mcb-application/              # Application layer (business services)
│   │   └── src/
│   │       ├── services/             # Application services
│   │       │   ├── context.rs        # ContextService
│   │       │   ├── search.rs         # SearchService
│   │       │   └── indexing.rs       # IndexingService
│   │       ├── domain_services/      # Domain service implementations
│   │       │   └── chunking.rs       # ChunkingOrchestrator
│   │       └── ports/                # Application-level ports
│   │
│   ├── mcb-infrastructure/           # Infrastructure layer (technical services)
│   │   └── src/
│   │       ├── di/                   # Shaku dependency injection
│   │       │   ├── modules/          # DI modules
│   │       │   └── bootstrap.rs      # DI bootstrap
│   │       ├── config/               # Configuration management
│   │       ├── cache/                # Cache infrastructure
│   │       ├── crypto/               # Encryption and hashing
│   │       ├── health/               # Health checks
│   │       ├── logging/              # Logging configuration
│   │       ├── adapters/             # Infrastructure adapters
│   │       │   ├── infrastructure/   # Null adapters for DI
│   │       │   ├── providers/        # Provider adapters
│   │       │   └── repository/       # Repository adapters
│   │       └── infrastructure/       # Re-exports and facades
│   │
│   ├── mcb-providers/                # Provider implementations (adapters)
│   │   └── src/
│   │       ├── embedding/            # Embedding providers (6)
│   │       │   ├── openai.rs
│   │       │   ├── ollama.rs
│   │       │   ├── voyageai.rs
│   │       │   ├── gemini.rs
│   │       │   ├── fastembed.rs
│   │       │   └── null.rs
│   │       ├── vector_store/         # Vector store providers (6)
│   │       │   ├── memory.rs
│   │       │   ├── encrypted.rs
│   │       │   └── null.rs
│   │       ├── cache/                # Cache providers
│   │       │   ├── moka.rs
│   │       │   └── redis.rs
│   │       ├── language/             # Language processors (12)
│   │       │   ├── rust.rs
│   │       │   ├── python.rs
│   │       │   ├── javascript.rs
│   │       │   ├── typescript.rs
│   │       │   ├── go.rs
│   │       │   ├── java.rs
│   │       │   ├── c.rs
│   │       │   ├── cpp.rs
│   │       │   ├── csharp.rs
│   │       │   ├── ruby.rs
│   │       │   ├── php.rs
│   │       │   ├── swift.rs
│   │       │   └── kotlin.rs
│   │       ├── routing/              # Routing logic
│   │       │   ├── circuit_breaker.rs
│   │       │   └── health.rs
│   │       └── admin/                # Admin providers
│   │           └── metrics.rs
│   │
│   ├── mcb-server/                   # MCP protocol server
│   │   └── src/
│   │       ├── handlers/             # MCP tool handlers
│   │       ├── transport/            # Stdio transport
│   │       ├── tools/                # Tool registry
│   │       ├── admin/                # Admin API
│   │       └── init.rs               # Server initialization
│   │
│   └── mcb-validate/                 # Architecture validation
│       └── src/
│           ├── validators/           # Individual validators
│           └── report.rs             # Validation reporting
```

## Architecture Layers

| Layer | Crate | Purpose | Key Components |
|-------|-------|---------|----------------|
| **Domain** | `mcb-domain` | Business entities and rules | Ports, types, entities, repositories |
| **Application** | `mcb-application` | Use case orchestration | ContextService, IndexingService, SearchService |
| **Infrastructure** | `mcb-infrastructure` | Technical services | DI, auth, cache, config, health |
| **Providers** | `mcb-providers` | External service adapters | Embedding (6), VectorStore (6), Cache, Language (12) |
| **Server** | `mcb-server` | Protocol implementation | MCP handlers, admin API |
| **Validation** | `mcb-validate` | Architecture enforcement | 12 validators, violation reporting |
| **Facade** | `mcb` | Public API | Re-exports from all crates |

## Dependency Graph

```
mcb-domain (innermost - no external deps)
    ↑
mcb-application → mcb-domain
    ↑
mcb-infrastructure → mcb-domain
    ↑
mcb-providers → mcb-domain, mcb-application
    ↑
mcb-server → mcb-domain, mcb-infrastructure, mcb-providers
    ↑
mcb (facade) → all above
```

## Feature Flags

Provider features are controlled via Cargo.toml feature flags:

| Feature | Default | Description |
|---------|---------|-------------|
| `embedding-ollama` | Yes | Ollama embedding provider |
| `embedding-openai` | No | OpenAI embedding provider |
| `embedding-voyageai` | No | VoyageAI embedding provider |
| `embedding-gemini` | No | Google Gemini embedding provider |
| `embedding-fastembed` | No | FastEmbed local embeddings |
| `vectorstore-memory` | Yes | In-memory vector store |
| `vectorstore-encrypted` | No | AES-GCM encrypted store |
| `cache-moka` | Yes | Moka cache provider |
| `cache-redis` | No | Redis cache provider |
| `lang-all` | Yes | All 12 language processors |

*Updated: 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
