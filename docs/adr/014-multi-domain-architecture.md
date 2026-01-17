# ADR 014: Multi-Domain Architecture Strategy

## Status

**Superseded** (v0.1.1)
**Date**: 2026-01-14
**Version**: v0.1.1 Update

> **Note**: This ADR describes future plans for multi-domain expansion. The v0.1.1 release
> implemented the [Seven-Crate Clean Architecture](013-clean-architecture-crate-separation.md)
> which provides the foundation for this multi-domain strategy.

## Context

MCB v0.1.1 has implemented the seven-crate Clean Architecture foundation. Future versions will integrate:

-   Code analysis capabilities (complexity, debt, quality)
-   Git integration (repository analysis, commit history)
-   Advanced tools (refactoring, scaffolding, mutation testing)

**Challenge**: How to organize multiple domains using the modular crate structure?

## Decision

Adopt **modular domain architecture** within the seven-crate structure:

```
mcp-context-browser/
├── libs/                           # Shared workspace libraries
│   ├── tree-sitter-analysis/       # AST parsing (shared by all domains)
│   ├── code-metrics/               # Metrics algorithms (v0.3.0+)
│   └── analysis-core/              # Orchestration utilities (v0.3.0+)
├── crates/
│   ├── mcb-domain/src/             # Domain layer (core)
│   │   ├── entities/               # Domain entities
│   │   ├── value_objects/          # Value objects
│   │   ├── ports/                  # Port traits
│   │   │   ├── providers/          # Current: embedding, vector_store
│   │   │   ├── infrastructure/     # Current: sync, snapshot, events
│   │   │   └── analysis/           # Future: analysis domain ports (v0.3.0+)
│   │   └── repositories/           # Repository traits
│   ├── mcb-application/src/        # Application layer
│   │   ├── use_cases/              # Current: context, search, indexing
│   │   ├── domain_services/        # Current: chunking
│   │   └── analysis/               # Future: ComplexityService (v0.3.0+)
│   ├── mcb-providers/src/          # Provider implementations
│   │   ├── embedding/              # Current: 6 providers
│   │   ├── vector_store/           # Current: 3 providers
│   │   └── analyzers/              # Future: PMAT adapters (v0.3.0+)
│   ├── mcb-infrastructure/src/     # Cross-cutting concerns
│   ├── mcb-server/src/             # MCP protocol handlers
│   └── mcb-validate/src/           # Architecture validation
```

### Domain Principles

1.  **Independence**: Each domain can be developed/tested separately
2.  **Ports & Adapters**: Domain logic only depends on interfaces
3.  **Incremental Growth**: Add domains without touching existing ones
4.  **Shared Libraries**: Common code in workspace crates

### Integration Strategy

**v0.1.1** (Current release):

-   Seven-crate Clean Architecture implemented
-   Shaku DI with two-layer strategy
-   20+ port traits with shaku::Interface (in mcb-application)
-   mcb-validate enforces layer boundaries

**v0.3.0** (Analysis domain):

-   Implement analysis domain logic
-   Port PMAT's complexity/TDG/SATD algorithms
-   3 new MCP tools

**v0.5.0** (Quality + Git domains):

-   Implement quality and git domains
-   7 new MCP tools

## Consequences

**Positive**:

-   Clear separation of concerns
-   Incremental feature addition
-   Testable in isolation
-   Future domains easy to add

**Negative**:

-   More directories (complexity)
-   Potential code duplication without careful planning

**Mitigation**:

-   Use workspace libraries for shared code
-   Enforce port traits for cross-domain communication
-   Document domain boundaries clearly

## Implementation Checklist

**v0.1.1 (Completed)**:

-   [x] Seven-crate Clean Architecture implemented
-   [x] Shaku DI with two-layer strategy
-   [x] 20+ port traits with shaku::Interface (in mcb-application)
-   [x] mcb-validate enforces layer boundaries

**v0.3.0 (Planned)**:

-   [ ] Create `crates/mcb-application/src/ports/analysis/` (analysis domain ports)
-   [ ] Create `crates/mcb-providers/src/analyzers/` (PMAT adapters)
-   [ ] Define `AnalysisInterface` trait
-   [ ] Port PMAT complexity/TDG/SATD algorithms

**v0.5.0 (Planned)**:

-   [ ] Define `QualityInterface` trait
-   [ ] Define `GitInterface` trait
-   [ ] Implement quality and git domain services

## Related ADRs

-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI patterns
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Seven-crate structure

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
