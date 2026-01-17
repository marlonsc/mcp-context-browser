# Contributing to MCP Context Browser

Thank you for your interest in contributing! This guide helps you get started with development.

## 🚀 Getting Started

### Prerequisites

\1-  **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
\1-  **Git**: Version control system

### Setup Development Environment

```bash

# Clone the repository
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Build the project
cargo build

# Run basic tests
cargo test

# Run the development server
cargo run
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
   cargo test
   ```

1.  **Submit PR**: Create pull request with clear description

## 📝 Coding Standards

### Rust Guidelines

\1-   Follow [The Rust Programming Language](https://doc.rust-lang.org/book/) conventions
\1-   Use `rustfmt` for formatting: `cargo fmt`
\1-   Follow `clippy` suggestions: `cargo clippy`
\1-   Write idiomatic Rust code

### Code Structure (v0.1.1 Modular Crates)

```text
crates/
├── mcb/                # Unified facade crate (public API)
├── mcb-domain/         # Core types, ports, entities (innermost)
├── mcb-application/    # Business services (use cases, domain services)
├── mcb-providers/      # External integrations (embedding, vector store, language)
├── mcb-infrastructure/ # Shared systems (DI, config, null adapters)
├── mcb-server/         # MCP protocol, HTTP transport, admin
└── mcb-validate/       # Architecture validation (development tool)
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

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
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

\1-   [ ] Tests pass: `cargo test`
\1-   [ ] Code formats correctly: `cargo fmt --check`
\1-   [ ] No linting errors: `cargo clippy -- -D warnings`
\1-   [ ] CI checks pass: `make ci`
\1-   [ ] Documentation updated if needed

### PR Description

Include:

\1-   What changes were made
\1-   Why the changes were needed
\1-   How to test the changes
\1-   Any breaking changes

### Review Process

1.  Automated checks run (tests, linting)
2.  Code review by maintainers
3.  Changes requested or approved
4.  Merge when approved

## 🐛 Reporting Issues

### Bug Reports

**Include:**

\1-   Steps to reproduce
\1-   Expected vs actual behavior
\1-   Environment details (Rust version, OS)
\1-   Error messages/logs

### Feature Requests

**Include:**

\1-   Problem description
\1-   Proposed solution
\1-   Use cases
\1-   Alternative approaches considered

## 🚀 Examples

The project includes several examples demonstrating different usage patterns:

### Configuration Examples

**Basic Configuration**(`examples/config_demo.rs`):

```rust
// Demonstrates TOML configuration loading and validation
use mcp_context_browser::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from config.toml
    let config = Config::from_file("config.toml").await?;
    println!("Loaded configuration: {:?}", config);
    Ok(())
}
```

**Advanced Routing**(`examples/advanced_routing.rs`):

```rust
// Demonstrates provider routing with circuit breakers and failover
use mcp_context_browser::routing::{Router, CircuitBreakerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure routing with multiple providers and circuit breakers
    let router = Router::new()
        .with_circuit_breaker(CircuitBreakerConfig::default())
        .with_failover_providers(vec!["openai", "ollama", "gemini"]);

    // Route requests intelligently based on health and performance
    let result = router.route_embedding_request(query).await?;
    println!("Routed through: {}", result.provider_used);
    Ok(())
}
```

### Running Examples

```bash

# Run a specific example
cargo run --example config_demo

# Run with custom configuration
CONFIG_FILE=my_config.toml cargo run --example advanced_routing

# List all available examples
cargo run --bin mcp-context-browser -- --help
```

## 📞 Getting Help

\1-  **Issues**: Use GitHub Issues for bugs and features
\1-  **Discussions**: Use GitHub Discussions for questions
\1-  **Documentation**: Check docs/architecture/ARCHITECTURE.md for technical details

## Code of Conduct

Be respectful and constructive in all interactions. Focus on improving the project and helping fellow contributors.

---

## Cross-References

\1-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
\1-  **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
\1-  **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
\1-  **Roadmap**: [ROADMAP.md](./ROADMAP.md)
\1-  **Module Documentation**: [docs/modules/](../modules/)
