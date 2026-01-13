# ADR 006: Code Audit and Architecture Improvements

## Status

**Implemented** (v0.1.0+)

> Major improvements completed:
>
> -   Provider pattern with trait-based DI across all providers
> -   Async-first architecture with 480+ async functions
> -   thiserror-based error handling throughout
> -   File sizes reduced (most < 500 lines)
> -   3 files in src/infrastructure/auth/ retain RwLock unwraps (acceptable for lock poisoning)
> -   Test coverage improved to 790+ tests (100% pass rate)
>
> **January 2026 Updates:**
>
> -   ✅ **14 domain port traits** fully wired in `src/domain/ports/` (up from 2)
> -   ✅ **CostTracker bug fix** - Fixed infinite recursion in trait method implementations
> -   ✅ **SnapshotProvider consolidation** - Merged duplicate trait definitions
> -   ✅ **Dead code removal** - Removed unused `hybrid_search_provider` parameter from ContextService
> -   ✅ **New trait implementations wired:**
>     - `IntelligentChunker` → `CodeChunker`
>     - `EventBus` → `EventPublisher`
>     - `SyncManager` → `SyncCoordinator`
>     - `SnapshotManager` → `SnapshotProvider` (expanded to 4 methods)
> -   ✅ **All traits extend `shaku::Interface`** for DI compatibility

## Context

The MCP Context Browser codebase has grown organically and accumulated several anti-patterns and technical debt that impact maintainability, reliability, and development velocity. A comprehensive code audit identified critical issues that need addressing before stable release.

Key problems identified:

\1-  **Giant structures**: Files with 1000+ lines violating Single Responsibility Principle
\1-  **Excessive unwrap/expect usage**: 157 occurrences across 28 files causing potential runtime crashes
\1-  **Tight coupling**: Direct concrete type dependencies instead of trait-based abstractions
\1-  **Missing input validation**: Lack of robust validation leading to runtime errors
\1-  **Inadequate error handling**: Generic error types without proper context
\1-  **Missing design patterns**: No Builder, Strategy, or Repository patterns implemented
\1-  **Poor testability**: High coupling making unit testing difficult

Current state analysis:

\1-   Total files: 189
\1-   Total lines: 42,314
\1-   Files with >1000 lines: 2 (config.rs, server/mod.rs)
\1-   unwrap/expect count: 157 across 28 files
\1-   Test coverage: ~60% (estimated)

## Decision

Implement comprehensive architectural improvements following SOLID principles, modern Rust best practices, and established design patterns to eliminate anti-patterns and establish a maintainable codebase foundation.

Key architectural decisions:

1.  **Break down giant structures**into focused modules following SRP
2.  **Eliminate all unwrap/expect**with proper error handling using thiserror
3.  **Implement Strategy Pattern**for provider abstractions
4.  **Add Builder Pattern**for complex configuration objects
5.  **Introduce Repository Pattern**for data access layers
6.  **Establish proper Dependency Injection**using trait bounds instead of `Arc<ConcreteType>`
7.  **Add comprehensive input validation**using the validator crate
8.  **Implement TDD approach**with mockall for comprehensive testing

## Consequences

These architectural improvements will significantly enhance code quality but require substantial refactoring effort.

### Positive Consequences

\1-  **Maintainability**: Smaller, focused modules easier to understand and modify
\1-  **Reliability**: Proper error handling eliminates unexpected crashes
\1-  **Testability**: Dependency injection enables comprehensive unit testing
\1-  **Extensibility**: Design patterns allow easy addition of new providers/features
\1-  **Performance**: Better resource management and optimization opportunities
\1-  **Security**: Input validation prevents malicious or malformed data
\1-  **Developer Experience**: Clearer APIs and better error messages
\1-  **Code Quality**: Adherence to Rust best practices and community standards

### Negative Consequences

\1-  **Development Time**: Significant refactoring effort required (6-8 weeks)
\1-  **Learning Curve**: Team needs to adapt to new patterns and abstractions
\1-  **Temporary Instability**: Refactoring may introduce bugs during transition
\1-  **Increased Complexity**: Additional abstraction layers add cognitive overhead
\1-  **Build Time**: More comprehensive testing increases CI/CD duration
\1-  **Documentation Updates**: All docs need updating for new architecture

## Alternatives Considered

### Alternative 1: Incremental Refactoring

\1-  **Description**: Address anti-patterns gradually over multiple releases
\1-  **Pros**: Less disruptive, allows feature development in parallel
\1-  **Cons**: Accumulates more technical debt, inconsistent codebase
\1-  **Rejection Reason**: Current issues are critical and blocking quality improvements

### Alternative 2: Complete Rewrite

\1-  **Description**: Rewrite entire codebase with clean architecture from scratch
\1-  **Pros**: Clean slate, no legacy constraints, modern patterns throughout
\1-  **Cons**: Extremely high risk, long development time, potential feature loss
\1-  **Rejection Reason**: Too risky for production system, better to evolve existing code

### Alternative 3: Minimal Fixes Only

\1-  **Description**: Only fix critical unwrap/expect issues, leave architecture as-is
\1-  **Pros**: Quick implementation, minimal disruption
\1-  **Cons**: Doesn't address root causes, technical debt continues growing
\1-  **Rejection Reason**: Doesn't solve systemic architectural problems

## Implementation Notes

### Phase 1: Foundation (Weeks 1-2)

**Code Changes Required:**

```rust
// Break down config.rs into specialized modules
pub mod embedding_config;
pub mod vector_store_config;
pub mod auth_config;
pub mod server_config;

// Implement proper error handling
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration validation failed: {field} - {reason}")]
    Validation { field: String, reason: String },
    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },
}

// Strategy Pattern for providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
}
```

**Migration Path:**

1.  Create new module structure alongside existing code
2.  Implement new types with backward compatibility
3.  Gradually migrate usage from old to new APIs
4.  Remove old code after full migration

### Phase 2: Design Patterns (Weeks 3-4)

**Builder Pattern Implementation:**

```rust
#[derive(Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Config {
    #[builder(setter(into))]
    pub embedding_provider: Box<dyn EmbeddingProvider>,
    #[builder(setter(into))]
    pub vector_store_provider: Box<dyn VectorStoreProvider>,
    #[builder(default)]
    pub auth: AuthConfig,
}

impl ConfigBuilder {
    fn validate(&self) -> Result<(), String> {
        // Validation logic here
        Ok(())
    }
}
```

**Repository Pattern:**

```rust
#[async_trait]
pub trait ChunkRepository {
    async fn save(&self, chunk: &CodeChunk) -> Result<String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<CodeChunk>>;
    async fn search_similar(&self, vector: &[f32], limit: usize) -> Result<Vec<CodeChunk>>;
}
```

### Phase 3: Quality Assurance (Weeks 5-6)

**Testing Strategy:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub EmbeddingProviderImpl {}
        impl EmbeddingProvider for EmbeddingProviderImpl {
            async fn embed(&self, text: &str) -> Result<Embedding>;
            async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
            fn dimensions(&self) -> usize;
        }
    }

    #[tokio::test]
    async fn test_service_with_mock_provider() {
        let mut mock_provider = MockEmbeddingProviderImpl::new();
        mock_provider
            .expect_embed()
            .returning(|_| Ok(Embedding::default()));

        let service = ContextService::new(mock_provider);
        let result = service.embed_text("test").await;
        assert!(result.is_ok());
    }
}
```

**Performance Benchmarks:**

\1-   Establish baseline metrics before changes
\1-   Monitor compilation time, binary size, runtime performance
\1-   Set up continuous benchmarking in CI/CD

### Phase 4: Validation and Release (Weeks 7-8)

**Rollback Plan:**

\1-   Feature flags for gradual rollout
\1-   Database migration rollback scripts
\1-   Configuration rollback procedures
\1-   Monitoring alerts for performance regressions

**Security Considerations:**

\1-   Input validation prevents injection attacks
\1-   Proper error handling avoids information leakage
\1-   Dependency updates for security patches
\1-   Code review security checklist

### Dependencies to Add

```toml
[dependencies]

# Validation
validator = { version = "0.16", features = ["derive"] }

# Better error handling
anyhow = "1.0"
thiserror = "1.0"

# Builder pattern
derive_builder = "0.12"

# Testing
mockall = "0.11"
test-case = "3.0"

# Async utilities
futures = "0.3"

# Configuration management
config = "0.13"
```

## Success Metrics

| Metric | Before | Target v0.1.0 | Measurement |
|--------|--------|----------------|-------------|
| Lines per file | >1000 | <500 | Static analysis |
| unwrap/expect count | 157 | 0 | Code search |
| Test coverage | ~60% | >85% | Cargo-tarpaulin |
| Compilation time | ~45s | <30s | Cargo build --timings |
| Cyclomatic complexity | >15 | <10 | Cargo +nightly rustc -- -Zunpretty=hir |
| Memory usage | Baseline | <10% increase | Valgrind massif |
| Error handling coverage | Partial | Complete | Manual review |

## References

\1-   [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
\1-   [SOLID Principles in Rust](https://www.fpcomplete.com/blog/solid-principles-rust/)
\1-   [Error Handling in Rust](https://blog.yoshuawuyts.com/error-handling-survey/)
\1-   [Repository Pattern](https://martinfowler.com/eaaCatalog/repository.html)
\1-   [Builder Pattern](https://refactoring.guru/design-patterns/builder)
\1-   [Strategy Pattern](https://refactoring.guru/design-patterns/strategy)

Related ADRs:

\1-   [ADR 001: Provider Pattern Architecture](../001-provider-pattern-architecture.md)
\1-   [ADR 002: Async-First Architecture](../002-async-first-architecture.md)
\1-   [ADR 003: C4 Model Documentation](../003-c4-model-documentation.md)
\1-   [ADR 004: Multi-Provider Strategy](../004-multi-provider-strategy.md)
