# ADR 002: Async-First Architecture

## Status

Accepted

## Context

The MCP Context Browser handles AI operations (embedding generation, vector searches) and large codebase processing that require high performance and concurrency. The system needs to handle multiple concurrent users, process large codebases efficiently, and integrate with external APIs that may have high latency.

Key performance requirements:

-   Handle 1000+ concurrent users
-   Process codebases with 1000+ files efficiently
-   Maintain sub-500ms response times for queries
-   Support streaming and background processing
-   Integrate with external APIs (OpenAI, vector databases)

Traditional synchronous programming would create bottlenecks and poor resource utilization for these I/O-bound operations.

## Decision

Adopt an async-first architecture using Tokio as the async runtime throughout the entire system. All provider interfaces use async traits, and the application is designed for high concurrency from the ground up.

Key architectural decisions:

-   Tokio as the primary async runtime
-   Async traits for all provider interfaces
-   Structured concurrency with Tokio::spawn
-   Async channels for inter-task communication
-   Hyper for HTTP client operations
-   Futures and streams for data processing pipelines

## Consequences

Async-first architecture provides excellent performance and concurrency but requires careful error handling and increases code complexity.

### Positive Consequences

-   **High Performance**: Efficient handling of concurrent operations and I/O
-   **Scalability**: Support for thousands of concurrent users
-   **Resource Efficiency**: Better CPU and memory utilization
-   **Future-Proof**: Aligns with modern async programming patterns
-   **Integration**: Natural fit with async HTTP clients and databases

### Negative Consequences

-   **Complexity**: Async code is harder to reason about and debug
-   **Error Handling**: Async error propagation is more complex
-   **Testing**: Async tests require special handling
-   **Learning Curve**: Steeper learning curve for team members
-   **Debugging**: Stack traces are less informative in async contexts

## Alternatives Considered

### Alternative 1: Synchronous Architecture

-   **Description**: Traditional blocking I/O with thread pools for concurrency
-   **Pros**: Simpler code, easier debugging, familiar patterns
-   **Cons**: Poor performance for I/O operations, limited concurrency
-   **Rejection Reason**: Cannot meet performance requirements for AI operations and concurrent users

### Alternative 2: Mixed Sync/Async

-   **Description**: Sync core with async wrappers for external operations
-   **Pros**: Gradual adoption, less complexity
-   **Cons**: Inconsistent patterns, performance bottlenecks at boundaries
-   **Rejection Reason**: Creates architectural inconsistency and performance issues

### Alternative 3: Actor Model (Actix)

-   **Description**: Use Actix for actor-based concurrency instead of Tokio
-   **Pros**: High-level abstractions, built-in supervision
-   **Cons**: Additional complexity, less ecosystem support
-   **Rejection Reason**: Tokio has better ecosystem support and performance for our use case

## Implementation Notes

### Async Runtime Configuration

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configure Tokio runtime for optimal performance
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("mcp-worker")
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack
        .enable_io()
        .enable_time()
        .build()?;

    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    // Application logic here
    Ok(())
}
```

### Async Trait Pattern

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Default implementation using streams for concurrency
        let futures = texts.iter().map(|text| self.embed(text));
        let results = futures_util::future::join_all(futures).await;

        results.into_iter().collect()
    }

    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

### Structured Concurrency

```rust
pub async fn process_codebase(&self, path: &Path) -> Result<IndexingStats> {
    // Create a task scope for structured concurrency
    let (stats_tx, stats_rx) = tokio::sync::mpsc::channel(100);

    // Spawn background tasks
    let file_processing = tokio::spawn(async move {
        self.process_files_concurrently(path, stats_tx).await
    });

    let metadata_update = tokio::spawn(async move {
        self.update_metadata_concurrently(path).await
    });

    // Wait for all tasks to complete
    let (file_result, metadata_result) = tokio::try_join!(file_processing, metadata_update)?;

    file_result?;
    metadata_result?;

    // Collect final statistics
    let mut total_stats = IndexingStats::default();
    while let Some(stats) = stats_rx.recv().await {
        total_stats.merge(stats);
    }

    Ok(total_stats)
}
```

### Error Handling in Async Code

```rust
pub async fn handle_request(&self, request: Request) -> Result<Response> {
    // Use timeout for external operations
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        self.process_request(request)
    ).await
    .map_err(|_| Error::timeout("Request processing timed out"))??;

    Ok(result)
}

async fn process_request(&self, request: Request) -> Result<Response> {
    // Handle cancellation gracefully
    let mut operation = self.start_operation(request);

    tokio::select! {
        result = operation.wait() => {
            result
        }
        _ = tokio::signal::ctrl_c() => {
            operation.cancel().await?;
            Err(Error::cancelled("Operation was cancelled"))
        }
    }
}
```

### Testing Async Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_embedding_provider() {
        let provider = MockEmbeddingProvider::new();

        // Test async operation
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.dimensions, 128);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let provider = Arc::new(MockEmbeddingProvider::new());

        // Test concurrent embedding
        let texts = vec!["text1".to_string(), "text2".to_string(), "text3".to_string()];
        let embeddings = provider.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.dimensions == 128));
    }
}
```

## References

-   [Tokio Documentation](https://tokio.rs/)
-   [Async Programming in Rust](https://rust-lang.github.io/async-book/)
-   [Structured Concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/)
