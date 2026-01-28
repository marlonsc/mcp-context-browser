# Contributing to MCP Context Browser

Thank you for your interest in contributing! This guide helps you get started with development.

## 🚀 Getting Started

### Prerequisites

-   **Rust 1.89+**: Install from [rustup.rs](https://rustup.rs/)
-   **Git**: Version control system

### Setup Development Environment

```bash

# Clone the repository
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Build the project
make build

# Run all tests (950+)
make test

# Run quality checks
make quality
```

## 🔄 Development Workflow

1.  **Choose Task**: Check [GitHub Issues](https://github.com/marlonsc/mcp-context-browser/issues) for tasks
2.  **Create Branch**: Use descriptive names

   ```bash
   git checkout -b feature/your-feature-name
   ```

1.  **Make Changes**: Implement your feature or fix
2.  **Test Changes**: Ensure tests pass

   ```bash
   make test
   ```

1.  **Submit PR**: Create pull request with clear description

## 📝 Coding Standards

### Rust Guidelines

-   Follow [The Rust Programming Language](https://doc.rust-lang.org/book/) conventions
-   Use `rustfmt` for formatting: `cargo fmt`
-   Follow `clippy` suggestions: `cargo clippy`
-   Write idiomatic Rust code

### Code Structure (v0.1.2 Clean Architecture)

```text
crates/
├── mcb/                # Unified facade crate (public API)
├── mcb-domain/         # Core types, ports, entities (innermost)
├── mcb-application/    # Business services (use cases, domain services)
├── mcb-providers/      # External integrations (embedding, vector store, language)
├── mcb-infrastructure/ # Shared systems (DI, config, null adapters)
├── mcb-server/         # MCP protocol, HTTP transport, admin
└── mcb-validate/       # Architecture validation (Phases 1-3 verified)
```

### Commit Messages

Use clear, descriptive commit messages:

```bash
feat: add new MCP tool handler
fix: resolve memory leak in vector storage
docs: update API documentation
```

## 🧪 Testing

### Running Tests

```bash

# Run all tests (950+)
make test

# Run unit tests only
make test-unit

# Run specific test with output
cargo test test_name -- --nocapture
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test implementation
        assert_eq!(result, expected);
    }
}
```

## 📋 Pull Request Guidelines

### Before Submitting

-   [ ] Tests pass: `make test`
-   [ ] Code formats correctly: `make fmt`
-   [ ] No linting errors: `make lint`
-   [ ] Quality checks pass: `make quality`
-   [ ] Documentation updated if needed

### PR Description

Include:

-   What changes were made
-   Why the changes were needed
-   How to test the changes
-   Any breaking changes

### Review Process

1.  Automated checks run (tests, linting)
2.  Code review by maintainers
3.  Changes requested or approved
4.  Merge when approved

## 🐛 Reporting Issues

### Bug Reports

**Include:**

-   Steps to reproduce
-   Expected vs actual behavior
-   Environment details (Rust version, OS)
-   Error messages/logs

### Feature Requests

**Include:**

-   Problem description
-   Proposed solution
-   Use cases
-   Alternative approaches considered

## 🚀 Examples

The project includes several examples demonstrating different usage patterns:

### Configuration Examples

**Basic Configuration** (`examples/config_demo.rs`):

```rust
// Demonstrates TOML configuration loading and validation
// v0.1.2: Use mcb facade crate for public API
use mcb::infrastructure::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from config.toml
    let config = Config::from_file("config.toml").await?;
    println!("Loaded configuration: {:?}", config);
    Ok(())
}
```

**Using DI Container** (`examples/di_demo.rs`):

```rust
// Demonstrates v0.1.2 Two-Layer DI Strategy (ADR-012)
use mcb::infrastructure::di::{DiContainerBuilder, AppContainer};
use mcb::application::ports::providers::EmbeddingProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build container with Shaku modules (null providers for testing)
    let container = DiContainerBuilder::new().build().await?;

    // Resolve providers from container
    let embedding: Arc<dyn EmbeddingProvider> = container.resolve();
    println!("Embedding provider resolved: {:?}", embedding);
    Ok(())
}
```

### Running Examples

```bash

# Run a specific example
cargo run --example config_demo

# Run with custom configuration
CONFIG_FILE=my_config.toml cargo run --example advanced_routing

# Run server directly
cargo run --bin mcp-context-browser
```

## 📞 Getting Help

-   **Issues**: Use GitHub Issues for bugs and features
-   **Discussions**: Use GitHub Discussions for questions
-   **Documentation**: Check docs/architecture/ARCHITECTURE.md for technical details

## Code of Conduct

Be respectful and constructive in all interactions. Focus on improving the project and helping fellow contributors.

---

## Cross-References

### Architecture (v0.1.2)

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) - System overview
-   **ADR-012**: [Two-Layer DI Strategy](../adr/012-di-strategy-two-layer-approach.md) - Shaku + factories
-   **ADR-013**: [Clean Architecture Crate Separation](../adr/013-clean-architecture-crate-separation.md) - Eight-crate structure
-   **Implementation Status**: [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) - Current state

### Operations

-   **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
-   **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
-   **Roadmap**: [ROADMAP.md](./ROADMAP.md)
