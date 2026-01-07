# ADR 001: Strategy Pattern Implementation

Date: 2026-01-07

## Status

Accepted

## Context

The MCP Context Browser codebase has grown to include multiple provider implementations (OpenAI, Ollama, Gemini, etc.) and service layers. The current architecture uses dynamic dispatch (`Arc<dyn Trait>`) throughout, which provides runtime flexibility but makes compile-time verification and optimization difficult.

The God Object pattern in `McpServer` with 15+ dependencies creates tight coupling and makes testing and maintenance challenging.

## Decision

Implement the Strategy pattern using Rust's trait bounds and generics instead of dynamic dispatch where appropriate. This provides:

1. **Compile-time verification** of provider compatibility
2. **Better performance** through monomorphization
3. **Improved testability** with concrete types
4. **Cleaner dependency injection** with trait bounds

Key changes:
- Generic service implementations: `GenericContextService<E, V>`
- Repository pattern with trait bounds
- Provider strategy composition at compile time
- Maintain backward compatibility with existing dynamic dispatch interfaces

## Consequences

### Positive
- **Performance**: Monomorphization eliminates dynamic dispatch overhead
- **Safety**: Compile-time verification of provider compatibility
- **Testability**: Concrete types enable easier unit testing
- **Maintainability**: Clear separation of concerns with trait bounds

### Negative
- **Binary size**: Increased due to monomorphization
- **Compilation time**: Longer compile times with generics
- **Complexity**: More complex type signatures

### Risks
- **Breaking changes**: Generic APIs may require different usage patterns
- **Migration complexity**: Converting existing code to use generics

## Implementation

### Generic Service Layer
```rust
pub struct GenericContextService<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    embedding_provider: Arc<E>,
    vector_store_provider: Arc<V>,
}
```

### Repository Pattern
```rust
pub struct VectorStoreChunkRepository<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    // Implementation
}
```

### Usage Example
```rust
let embedding_provider = Arc::new(MockEmbeddingProvider::new());
let vector_store_provider = Arc::new(InMemoryVectorStoreProvider::new());

let service = GenericContextService::new(
    embedding_provider,
    vector_store_provider,
);
```

## Alternatives Considered

### Option 1: Continue with Dynamic Dispatch
- **Pros**: Simple, flexible, backward compatible
- **Cons**: Runtime overhead, harder testing, less type safety

### Option 2: Hybrid Approach
- **Pros**: Best of both worlds
- **Cons**: Increased complexity, inconsistent API

### Option 3: Full Generic Rewrite
- **Pros**: Maximum performance and safety
- **Cons**: Breaking changes, migration effort

## References

- [Strategy Pattern](https://en.wikipedia.org/wiki/Strategy_pattern)
- [Rust Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [Monomorphization](https://doc.rust-lang.org/book/ch10-02-traits.html)