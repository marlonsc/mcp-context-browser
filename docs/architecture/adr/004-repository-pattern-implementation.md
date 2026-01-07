# ADR 004: Repository Pattern Implementation

Date: 2026-01-07

## Status

Accepted

## Context

The MCP Context Browser needs a clean separation between business logic and data access. The current implementation has data access logic scattered across services, making testing difficult and creating tight coupling between business logic and storage mechanisms.

Key requirements:
1. **Testability**: Business logic should be testable without database/storage dependencies
2. **Flexibility**: Easy switching between different storage implementations
3. **Maintainability**: Clear separation of concerns
4. **Consistency**: Uniform data access patterns across the application

## Decision

Implement the Repository pattern with trait-based interfaces for data access. Create repository implementations for different storage backends while maintaining a consistent interface for business logic.

Key components:
- **Repository Traits**: Define data access interfaces
- **Concrete Implementations**: Vector store, database, and in-memory implementations
- **Dependency Injection**: Services receive repositories through constructor injection
- **Async Support**: All repository operations are async for scalability

## Consequences

### Positive
- **Testability**: Easy mocking of data access for unit tests
- **Flexibility**: Swap storage implementations without changing business logic
- **Maintainability**: Clear separation of concerns
- **Consistency**: Uniform data access patterns
- **Performance**: Async operations support high concurrency

### Negative
- **Complexity**: Additional abstraction layer
- **Boilerplate**: Repository trait implementations
- **Learning Curve**: Understanding the pattern

### Risks
- **Over-abstraction**: Repository pattern can be overkill for simple operations
- **Performance Overhead**: Additional indirection
- **Maintenance Burden**: Keeping repository interfaces in sync

## Implementation

### Repository Traits

```rust
#[async_trait]
pub trait ChunkRepository: Send + Sync {
    async fn save(&self, chunk: &CodeChunk) -> Result<String>;
    async fn save_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<String>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<CodeChunk>>;
    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn delete_collection(&self, collection: &str) -> Result<()>;
    async fn stats(&self) -> Result<RepositoryStats>;
}

#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn semantic_search(&self, collection: &str, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn hybrid_search(&self, collection: &str, query: &str, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()>;
    async fn clear_index(&self, collection: &str) -> Result<()>;
    async fn search_stats(&self) -> Result<SearchStats>;
}
```

### Concrete Implementation

```rust
pub struct VectorStoreChunkRepository<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    embedding_provider: Arc<E>,
    vector_store_provider: Arc<V>,
    collection_prefix: String,
}
```

### Service Integration

```rust
pub struct RepositoryContextService<C, S>
where
    C: ChunkRepository + Send + Sync,
    S: SearchRepository + Send + Sync,
{
    chunk_repository: Arc<C>,
    search_repository: Arc<S>,
}
```

## Alternatives Considered

### Option 1: Active Record Pattern
- **Pros**: Simple, direct database operations
- **Cons**: Tight coupling, hard to test, business logic in data layer

### Option 2: Data Access Objects (DAO)
- **Pros**: Simple abstraction, easy to understand
- **Cons**: Less flexible than Repository pattern

### Option 3: Query Objects
- **Pros**: Flexible querying
- **Cons**: Complex for simple CRUD operations

## Repository Responsibilities

### Chunk Repository
- **Storage**: Persist code chunks with embeddings
- **Retrieval**: Find chunks by ID or collection
- **Batch Operations**: Efficient bulk operations
- **Statistics**: Provide storage and performance metrics

### Search Repository
- **Semantic Search**: Vector similarity search
- **Hybrid Search**: Combine keyword and semantic search
- **Indexing**: Prepare data for search operations
- **Performance Monitoring**: Track search metrics

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_chunk_repository_save() {
    let repo = create_test_repository();
    let chunk = create_test_chunk();

    let result = tokio::runtime::Runtime::new().unwrap()
        .block_on(repo.save(&chunk));

    assert!(result.is_ok());
}
```

### Integration Tests
```rust
#[test]
fn test_repository_service_integration() {
    let repo = create_test_repository();
    let service = RepositoryContextService::new(Arc::new(repo));

    // Test full workflow
    let result = tokio::runtime::Runtime::new().unwrap()
        .block_on(service.store_and_search());

    assert!(result.is_ok());
}
```

## References

- [Repository Pattern](https://martinfowler.com/eaaCatalog/repository.html)
- [Domain-Driven Design](https://domainlanguage.com/ddd/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)