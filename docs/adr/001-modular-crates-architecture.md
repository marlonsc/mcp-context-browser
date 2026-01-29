# ADR 001: Modular Crates Architecture

## Status

**Implemented** (v0.1.1)

> Fully implemented with Clean Architecture + Shaku DI across 8 crates.

## Context

Initially, the MCP Context Browser had a monolithic architecture. As the project grew, the need for better code organization, separation of concerns, and component reusability emerged. We evaluated adopting a modular architecture by dividing the system into multiple Rust crates, each responsible for a specific domain or functionality (e.g., core server crate, context providers crate, inter-module communication crate, etc.). We also considered how to manage the orderly initialization and shutdown of modules in a resilient manner.

## Decision

We opted for a modular architecture based on crates, where the project is divided into independent sub-modules compiled separately. Each crate encapsulates specific services and logics (e.g., core server crate, providers crate, EventBus crate, etc.), but all operate in an integrated manner. To coordinate the lifecycle of modules, we introduced a central component called ServiceManager responsible for registering, initializing, and maintaining references to all services (from each crate) running. Similarly, we implemented a graceful shutdown mechanism via ShutdownCoordinator, which orchestrates the termination of each service in the correct order when the application is shut down, ensuring resource release (threads, connections) in a safe manner.

## Implementation

### Port Trait Definition (mcb-application)

Ports are defined as traits extending `shaku::Interface` for DI compatibility:

```rust
// crates/mcb-application/src/ports/providers/embedding.rs
use shaku::Interface;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Interface + Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

Important: All port traits must:

-   Extend `shaku::Interface` (which implies `'static + Send + Sync` with thread_safe feature)
-   Be object-safe (no generic methods, no `Self` returns)
-   Use `async_trait` for async methods

### Provider Implementation (mcb-providers)

Providers implement ports and are registered as Shaku components:

```rust
// crates/mcb-providers/src/embedding/openai.rs
use shaku::Component;
use mcb_domain::ports::providers::EmbeddingProvider;

#[derive(Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct OpenAIEmbeddingProvider {
    #[shaku(default)]
    api_key: String,
    #[shaku(default)]
    model: String,
    #[shaku(default)]
    client: reqwest::Client,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // OpenAI API call implementation
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Batch embedding implementation
    }

    fn dimensions(&self) -> usize { 1536 }
    fn provider_name(&self) -> &str { "openai" }
}
```

### Null Provider for Testing

```rust
// crates/mcb-providers/src/embedding/null.rs
use shaku::Component;
use mcb_domain::ports::providers::EmbeddingProvider;

#[derive(Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct NullEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for NullEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Embedding> {
        Ok(Embedding::zeros(128))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        Ok(texts.iter().map(|_| Embedding::zeros(128)).collect())
    }

    fn dimensions(&self) -> usize { 128 }
    fn provider_name(&self) -> &str { "null" }
}
```

### DI Module Registration (mcb-infrastructure)

Shaku modules register components for DI:

```rust
// crates/mcb-infrastructure/src/di/modules/embedding_module.rs
use shaku::module;
use mcb_providers::embedding::NullEmbeddingProvider;

module! {
    pub EmbeddingModuleImpl {
        components = [NullEmbeddingProvider],
        providers = []
    }
}
```

### Service Layer with Injected Dependencies (mcb-application)

Use cases receive dependencies via constructor injection:

```rust
// crates/mcb-application/src/services/context.rs
use std::sync::Arc;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};

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

    pub async fn embed_and_store(&self, collection: &str, texts: &[String]) -> Result<()> {
        let embeddings = self.embedding_provider.embed_batch(texts).await?;
        self.vector_store_provider.store(collection, &embeddings).await?;
        Ok(())
    }
}
```

### Two-Layer DI Strategy

The system uses a two-layer approach for DI (see [ADR-012](012-di-strategy-two-layer-approach.md)):

**Layer 1: Shaku Modules** - Provide null implementations as defaults for testing:

```rust
// Testing with Shaku modules (null providers) — HISTORICAL; DI is now dill (ADR-029)
let container = DiContainerBuilder::new().build().await?;
// Uses NullEmbeddingProvider, NullVectorStoreProvider, etc.
```

**Layer 2: Runtime Factories** - Create production providers from configuration:

```rust
// Production with factories
let embedding = EmbeddingProviderFactory::create(&config.embedding, None)?;
let vector_store = VectorStoreProviderFactory::create(&config.vector_store, crypto)?;
let services = DomainServicesFactory::create_services(
    cache, crypto, config, embedding, vector_store, chunker
).await?;
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mcb_providers::embedding::NullEmbeddingProvider;
    use mcb_providers::vector_store::NullVectorStoreProvider;

    #[tokio::test]
    async fn test_context_service_with_null_providers() {
        let embedding_provider = Arc::new(NullEmbeddingProvider);
        let vector_store_provider = Arc::new(NullVectorStoreProvider);

        let service = ContextService::new(embedding_provider, vector_store_provider);

        let result = service.embed_and_store("test", &["hello".to_string()]).await;
        assert!(result.is_ok());
    }
}
```

## Consequences

This change to multiple crates improved code maintainability and scalability. Developers can evolve modules in isolation and even publish reusable crates. The modular architecture also facilitates unit testing and integration testing focused per module. On the other hand, it added complexity in managing versions between internal crates and required an orchestration layer (ServiceManager/ShutdownCoordinator) to coordinate dependencies and initialization order. These additional structures increase robustness at the cost of a small coordination overhead. Overall, the decision aligned with the goal of a pluggable and extensible design, allowing inclusion or removal of functionalities (crates) without significantly impacting the rest of the system.

## Crate Structure

```
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services orchestration
├── mcb-providers/       # Layer 3: Provider implementations (embedding, vector stores)
├── mcb-infrastructure/  # Layer 4: DI, config, cache, crypto, health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation rules
```

**Dependency Direction** (inward only):

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓
              mcb-providers
```

## Related ADRs

-   [ADR-002: Dependency Injection with Shaku](002-dependency-injection-shaku.md)
-   [ADR-003: Unified Provider Architecture](003-unified-provider-architecture.md)
-   [ADR-004: Event Bus (Local and Distributed)](004-event-bus-local-distributed.md)
-   [ADR-005: Context Cache Support (Moka and Redis)](005-context-cache-support.md)
