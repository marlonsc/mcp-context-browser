# MCP Context Browser

A Model Context Protocol (MCP) server for semantic code analysis and search using vector embeddings. This project provides a clean, modular architecture for code indexing and semantic search capabilities.

## Features

- **Semantic Code Search**: Find code using natural language queries
- **Vector Embeddings**: Use vector embeddings for accurate code similarity matching
- **Modular Architecture**: Clean separation of concerns with providers, services, and registry patterns
- **MCP Protocol**: Full compliance with Model Context Protocol for AI assistants
- **Extensible**: Easy to add new embedding providers and vector stores

## Architecture

The project follows a clean, modular architecture:

```
src/
├── core/           # Core types and error handling
├── providers/      # Provider implementations (embeddings, vector stores)
├── registry/       # Provider registration and dependency injection
├── factory/        # Factory patterns for creating providers
├── services/       # Business logic services
├── server/         # MCP server implementation
├── lib.rs          # Library exports
└── main.rs         # Executable entry point
```

### Key Components

- **Core**: Fundamental types, errors, and traits
- **Providers**: Pluggable implementations for embeddings and vector storage
- **Registry**: Thread-safe provider registration and lookup
- **Factory**: Creation patterns for different provider types
- **Services**: Business logic orchestration
- **Server**: MCP protocol handling

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd mcp-context-browser

# Build the project
cargo build --release

# Run the server
cargo run
```

## Usage

The MCP Context Browser provides two main tools:

### 1. Index Codebase

Index a directory for semantic search:

```json
{
  "name": "index_codebase",
  "arguments": {
    "path": "/path/to/your/codebase"
  }
}
```

### 2. Search Code

Search for code using natural language:

```json
{
  "name": "search_code",
  "arguments": {
    "query": "find functions that handle user authentication",
    "limit": 10
  }
}
```

## Configuration

The server uses default providers for MVP:

- **Embedding Provider**: Mock provider (returns fixed vectors)
- **Vector Store**: In-memory store

For production use, you would configure real providers:

```rust
// Example configuration (future feature)
let config = EmbeddingConfig {
    provider: "openai".to_string(),
    model: "text-embedding-ada-002".to_string(),
    api_key: Some("your-api-key".to_string()),
    // ... other settings
};
```

## Development

### Adding New Providers

1. Implement the provider trait:
```rust
#[async_trait]
impl EmbeddingProvider for MyProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> { /* ... */ }
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> { /* ... */ }
    fn dimensions(&self) -> usize { /* ... */ }
    fn provider_name(&self) -> &str { /* ... */ }
}
```

2. Register in the factory:
```rust
// In factory/mod.rs
async fn create_embedding_provider(&self, config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>> {
    match config.provider.as_str() {
        "my-provider" => Ok(Arc::new(MyProvider::new())),
        // ... existing providers
    }
}
```

### Testing

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Dependencies

Key dependencies include:

- `tokio`: Async runtime
- `serde`: Serialization
- `reqwest`: HTTP client (for future API integrations)
- `milvus-sdk-rust`: Vector database client
- `tracing`: Logging framework
- `async-trait`: Async trait support

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Roadmap

- [ ] Real embedding providers (OpenAI, VoyageAI, Ollama)
- [ ] Persistent vector stores (Milvus, Pinecone)
- [ ] Code parsing and chunking improvements
- [ ] Configuration file support
- [ ] Performance optimizations
- [ ] Additional MCP tools