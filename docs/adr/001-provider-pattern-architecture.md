# ADR 001: Provider Pattern Architecture

## Status

Accepted

## Context

The MCP Context Browser needs to support multiple AI providers (OpenAI, Anthropic, Ollama) and vector databases (Milvus, Pinecone, Qdrant) without creating tight coupling between the core business logic and external service implementations. The system must be extensible to add new providers without modifying existing code, and support testing with mock implementations.

Current requirements:

-   Support for multiple embedding providers with different APIs
-   Multiple vector storage backends with varying capabilities
-   Ability to switch providers at runtime
-   Testability with mock implementations
-   Clean separation between business logic and external dependencies

## Decision

Implement a provider pattern using Rust traits for abstraction, with a registry system for provider management and dependency injection for service instantiation.

Key architectural elements:

-   `EmbeddingProvider` trait for text-to-vector conversion
-   `VectorStoreProvider` trait for vector storage and retrieval
-   `ProviderRegistry` for runtime provider registration and lookup
-   `ServiceProvider` factory for dependency injection
-   Trait-based dependency injection in service constructors

## Consequences

The provider pattern provides excellent separation of concerns and extensibility but introduces some complexity in provider management.

### Positive Consequences

-   **High Extensibility**: New providers can be added without modifying existing code
-   **Clean Architecture**: Clear separation between business logic and external services
-   **Testability**: Easy mocking and testing through dependency injection
-   **Runtime Flexibility**: Providers can be switched without recompilation
-   **Type Safety**: Rust traits ensure compile-time interface compliance

### Negative Consequences

-   **Increased Complexity**: Additional abstraction layers and indirection
-   **Provider Management**: Need for registry and factory patterns
-   **Trait Bounds**: Generic constraints can complicate service implementations
-   **Testing Overhead**: More setup required for unit testing with mocks

## Alternatives Considered

### Alternative 1: Direct Provider Usage

-   **Description**: Services directly instantiate and use specific provider implementations
-   **Pros**: Simpler code, fewer abstractions
-   **Cons**: Tight coupling, difficult to test, hard to add new providers
-   **Rejection Reason**: Violates SOLID principles and makes the system inflexible

### Alternative 2: Configuration-Based Factory

-   **Description**: Simple factory pattern with configuration strings to select providers
-   **Pros**: Less complex than full registry system
-   **Cons**: Limited runtime flexibility, still requires recompilation for new providers
-   **Rejection Reason**: Doesn't provide the same level of testability and runtime flexibility

### Alternative 3: Plugin Architecture

-   **Description**: Dynamic loading of provider implementations as plugins
-   **Pros**: True runtime extensibility without recompilation
-   **Cons**: Significant complexity, stability concerns, platform limitations
-   **Rejection Reason**: Overkill for current requirements, adds operational complexity

## Implementation Notes

### Core Traits

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}

#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()>;
    async fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<(f32, Embedding)>>;
    async fn clear(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}
```

### Registry Implementation

```rust
pub struct ProviderRegistry {
    embedding_providers: HashMap<String, Arc<dyn EmbeddingProvider>>,
    vector_store_providers: HashMap<String, Arc<dyn VectorStoreProvider>>,
}

impl ProviderRegistry {
    pub fn register_embedding_provider(&mut self, name: &str, provider: Arc<dyn EmbeddingProvider>) {
        self.embedding_providers.insert(name.to_string(), provider);
    }

    pub fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>> {
        self.embedding_providers.get(name).cloned()
            .ok_or_else(|| Error::not_found(format!("Embedding provider: {}", name)))
    }
}
```

### Service Constructor Injection

```rust
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self { embedding_provider, vector_store_provider }
    }
}
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_context_service_with_mocks() {
        let embedding_provider = Arc::new(MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(MockVectorStoreProvider::new());

        let service = ContextService::new(embedding_provider, vector_store_provider);

        // Test with injected mocks
        let result = service.embed_text("test").await;
        assert!(result.is_ok());
    }
}
```

## References

-   [Provider Pattern](https://en.wikipedia.org/wiki/Provider_model)
-   [Dependency Injection in Rust](https://doc.rust-lang.org/book/ch10-02-traits.html)
-   [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
