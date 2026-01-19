# ADR-013: Clean Architecture Crate Separation

## Status

Implemented (v0.1.1) - Seven crates
Updated (v0.1.2) - Added mcb-validate as 8th crate

## Context

As MCP Context Browser evolved from a monolithic architecture to a production-ready system, the codebase grew to include multiple providers, complex DI patterns, validation systems, and protocol handlers. A monolithic structure created several challenges:

1.  **Coupling**: Changes to infrastructure affected domain logic
2.  **Testability**: Testing required loading entire application context
3.  **Compilation**: Small changes triggered full rebuilds
4.  **Clarity**: No clear boundaries for where code belongs
5.  **Dependency Direction**: Violations of dependency inversion were easy to introduce

The Clean Architecture pattern, as described by Robert C. Martin, addresses these concerns through strict layer separation with dependencies pointing inward toward the domain.

## Decision

We organize the codebase into **eight Cargo workspace crates** following Clean Architecture principles:

### Layer 1: Domain (`mcb-domain`)

**Purpose**: Core business entities, port traits (interfaces), and domain validation rules.

**Characteristics**:

-   Zero external dependencies (except `async_trait`, `thiserror`)
-   Defines port traits that extend `shaku::Interface` for DI compatibility
-   Contains domain entities: `CodeChunk`, `Repository`, `Embedding`, `SearchResult`
-   Contains value objects: `Language`, `ChunkType`, `SearchQuery`
-   Defines domain errors with `thiserror`
-   No implementations of external services

**Key Directories**:

```
mcb-domain/src/
├── entities/           # Domain entities (CodeChunk, Codebase)
├── events/             # Domain events (DomainEvent, EventPublisher)
├── repositories/       # Repository port traits (ChunkRepository, SearchRepository)
├── value_objects/      # Value objects (Embedding, Config, Search, Types)
├── constants.rs        # Domain constants
└── error.rs            # Domain error types
```

**Dependencies**: None (except trait utilities)

### Layer 2: Application (`mcb-application`)

**Purpose**: Business logic orchestration and use case implementations.

**Characteristics**:

-   Depends only on `mcb-domain`
-   Contains use cases: `ContextService`, `SearchService`, `IndexingService`
-   Orchestrates domain operations without knowing implementations
-   Defines application-level ports (service interfaces)
-   Contains the `ChunkingOrchestrator` for batch processing

**Key Directories**:

```
mcb-application/src/
├── services/
│   ├── context.rs      # ContextService - embedding + storage coordination
│   ├── search.rs       # SearchService - semantic search
│   └── indexing.rs     # IndexingService - codebase indexing
├── domain_services/
│   └── chunking.rs     # ChunkingOrchestrator
├── ports/              # Application-level port interfaces
└── use_cases/          # Use case modules
```

**Dependencies**: `mcb-domain`

### Layer 3: Providers (`mcb-providers`)

**Purpose**: Implementations of domain port traits for external services.

**Characteristics**:

-   Depends on `mcb-domain` (implements port traits)
-   Feature-flagged providers for optional dependencies
-   Contains real implementations: OpenAI, Ollama, etc. (6 embedding, 3 vector store, 12 language)
-   Contains null implementations for testing
-   Organized by provider category

**Key Directories**:

```
mcb-providers/src/
├── embedding/          # 6 embedding providers
│   ├── openai.rs
│   ├── ollama.rs
│   ├── voyage.rs
│   ├── gemini.rs
│   ├── fastembed.rs
│   └── null.rs
├── vector_store/       # 3 vector store providers
│   ├── in_memory.rs
│   ├── encrypted.rs
│   └── null.rs
├── cache/              # Cache providers
│   ├── moka.rs
│   └── null.rs
├── language/           # 12 AST-based language processors
│   ├── rust.rs
│   ├── python.rs
│   └── ...
├── routing/            # Circuit breaker, failover, health
└── hybrid_search/      # BM25 + semantic search
```

**Dependencies**: `mcb-domain`, external SDKs (feature-gated)

### Layer 4: Infrastructure (`mcb-infrastructure`)

**Purpose**: Shared technical services and cross-cutting concerns.

**Characteristics**:

-   Depends on `mcb-domain`, `mcb-application`, `mcb-providers`
-   Contains the Shaku-based DI system
-   Contains configuration management
-   Contains cross-cutting services (auth, metrics, events)
-   Provides factories for production provider creation

**Key Directories**:

```
mcb-infrastructure/src/
├── di/
│   ├── bootstrap.rs    # Container composition root
│   ├── modules/        # Shaku modules (7 modules)
│   │   ├── cache_module.rs
│   │   ├── embedding_module.rs
│   │   ├── data_module.rs
│   │   ├── language_module.rs
│   │   ├── infrastructure.rs
│   │   ├── server.rs
│   │   └── admin.rs
│   └── factory/        # Provider factories
│       ├── embedding.rs
│       └── vector_store.rs
├── config/             # Configuration types (13 modules)
├── adapters/           # Null adapters for DI
├── crypto/             # Encryption services
├── health/             # Health check infrastructure
└── logging/            # Logging configuration
```

**Dependencies**: `mcb-domain`, `mcb-application`, `mcb-providers`

### Layer 5: Server (`mcb-server`)

**Purpose**: MCP protocol implementation and HTTP API.

**Characteristics**:

-   Depends on all other crates
-   Entry point for the application
-   MCP protocol handler with stdio transport
-   Tool handlers (index, search, clear, status)
-   Admin API endpoints

**Key Directories**:

```
mcb-server/src/
├── handlers/           # MCP tool handlers
├── transport/          # Stdio transport
├── tools/              # Tool registry
├── admin/              # Admin API
├── init.rs             # Server initialization
└── main.rs             # Entry point
```

**Dependencies**: All crates

### Layer 6: Validate (`mcb-validate`)

**Purpose**: Architecture enforcement and code quality validation.

**Characteristics**:

-   Standalone validation tool
-   30+ validators for architecture rules
-   Violation trait system for unified reporting
-   TOML-based configuration
-   Used in CI/CD pipelines

**Key Components**:

-   `CleanArchitectureValidator`: Layer dependency rules
-   `ShakuValidator`: DI pattern compliance
-   `QualityValidator`: Code quality metrics
-   `OrganizationValidator`: File organization rules

**Dependencies**: Development tool, not production dependency

### Layer 7: Facade (`mcb`)

**Purpose**: Public API re-exports for external consumers.

**Characteristics**:

-   Re-exports public types from all crates
-   Provides unified interface for library users
-   Minimal code, mostly re-exports

**Dependencies**: All crates

## Dependency Graph

```
                    ┌─────────────┐
                    │   mcb-server│
                    │   (Layer 5) │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│mcb-infrastructure│ │ mcb-validate│ │       mcb       │
│    (Layer 4)     │ │  (Layer 6)  │ │   (Layer 7)     │
└────────┬────────┘ └─────────────┘ └─────────────────┘
         │
    ┌────┴────┬────────────┐
    │         │            │
    ▼         ▼            ▼
┌────────┐ ┌────────────┐ ┌─────────────┐
│mcb-app │ │mcb-providers│ │             │
│(Layer 2)│ │  (Layer 3) │ │             │
└────┬───┘ └──────┬─────┘ │             │
     │            │       │             │
     └────────────┼───────┘             │
                  │                     │
                  ▼                     │
           ┌─────────────┐              │
           │  mcb-domain │◄─────────────┘
           │   (Layer 1) │
           └─────────────┘

Arrow direction: depends on
```

## Clean Architecture Rules Enforced

1.  **Dependency Rule**: Dependencies only point inward (toward domain)
2.  **Abstraction Rule**: Inner layers define interfaces (ports), outer layers implement (adapters)
3.  **Entity Rule**: Domain entities have no external dependencies
4.  **Use Case Rule**: Application layer orchestrates, doesn't implement infrastructure

## Validation

The `mcb-validate` crate enforces these architectural rules:

```bash

# Run architecture validation
cargo run -p mcb-validate

# Key validators:

# - CleanArchitectureValidator: Checks layer dependency violations

# - ShakuValidator: Verifies DI patterns (SHAKU001-016 rules)

# - DependencyValidator: Ensures crate dependencies follow rules
```

## Consequences

### Positive

-   **Clear Boundaries**: Each crate has explicit responsibilities
-   **Testability**: Test domain/application without infrastructure
-   **Compilation**: Parallel builds, incremental compilation
-   **Maintainability**: Changes isolated to appropriate layers
-   **Onboarding**: Developers know where code belongs

### Negative

-   **Complexity**: Eight crates vs one requires coordination
-   **Boilerplate**: Port traits need implementations in multiple places
-   **Learning Curve**: Clean Architecture concepts required

### Neutral

-   **Cargo Workspace**: Standard Rust pattern, well-supported tooling
-   **Feature Flags**: Providers can be optionally included

## Implementation Notes

### Adding a New Provider

1.  Create implementation in `mcb-providers/src/<category>/`
2.  Implement the port trait from `mcb-application/src/ports/`
3.  Add feature flag in `mcb-providers/Cargo.toml`
4.  Register in factory in `mcb-infrastructure/src/di/factory/`

### Adding a New Use Case

1.  Define service interface in `mcb-application/src/ports/`
2.  Implement service in `mcb-application/src/services/`
3.  Inject port dependencies via constructor
4.  Wire in `mcb-infrastructure/src/di/` if needed

### Testing Patterns

```rust
// Unit test (mcb-application)
#[tokio::test]
async fn test_search_service() {
    let embedding = Arc::new(MockEmbeddingProvider::new());
    let vector_store = Arc::new(MockVectorStoreProvider::new());
    let service = SearchService::new(embedding, vector_store);
    // Test without infrastructure
}

// Integration test (mcb-server)
#[tokio::test]
async fn test_full_indexing_flow() {
    let container = DiContainerBuilder::new().build().await?;
    // Uses null providers from Shaku modules
}
```

## Related ADRs

-   [ADR-001: Provider Pattern Architecture](001-provider-pattern-architecture.md) - Provider trait patterns
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async patterns per layer
-   [ADR-003: C4 Model Documentation](003-c4-model-documentation.md) - Architecture visualization
-   [ADR-004: Multi-Provider Strategy](004-multi-provider-strategy.md) - mcb-providers organization
-   [ADR-005: Documentation Excellence](005-documentation-excellence.md) - Documentation per crate
-   [ADR-006: Code Audit and Improvements](006-code-audit-and-improvements.md) - Quality standards per layer
-   [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - mcb-server admin module
-   [ADR-011: HTTP Transport](011-http-transport-request-response-pattern.md) - mcb-server transport layer
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku DI in mcb-infrastructure

## References

-   [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
-   [Shaku Documentation](https://docs.rs/shaku)
-   Workspace-next refactoring plan (January 2026)
