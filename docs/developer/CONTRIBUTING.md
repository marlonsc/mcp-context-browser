# Contributing to MCP Context Browser

Thank you for your interest in contributing! This guide helps you get started with development.

## 🚀 Getting Started

### Prerequisites

-   **Rust 1.89+**: Install from [rustup.rs](https://rustup.rs/)
-   **Git**: Version control system

### Setup Development Environment

```bash

# Clone the repository
git clone https://github.com/marlonsc/mcb.git
cd mcb

# Build the project
make build

# Run all tests (950+)
make test

# Run quality checks
make quality
```

## 🔄 Development Workflow

1.  **Choose Task**: Check [GitHub Issues](https://github.com/marlonsc/mcb/issues) for tasks
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
-   [ ] No Rust lint errors: `make lint`; no Markdown lint errors: `make docs-lint`
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

## 🔧 Troubleshooting

### `make quality` or `make build` fails with linker errors

Errors like `cannot open ... .rlib: No such file or directory` or `can't find crate` often mean a corrupted or partial `target/` cache. Try:

```bash
cargo clean
make build
make quality
```

Use a normal system linker if you hit `rust-lld` issues (e.g. set `RUSTFLAGS` or use default `rustup` toolchain).

### Docs-only validation (no Rust build)

To check only documentation:

```bash
make docs-lint
make docs-validate QUICK=1
```

These do not require `cargo build` or a full toolchain.

## 🚀 Code References

Configuration uses **Figment** (ADR-025). DI uses **dill** and **init_app** (ADR-029):

-   **Config**: `mcb_infrastructure::config::ConfigLoader`, `AppConfig`. See [CONFIGURATION.md](../CONFIGURATION.md) and [ADR-025](../adr/025-figment-configuration.md).
-   **DI / bootstrap**: `mcb_infrastructure::di::bootstrap::init_app(config)` returns `AppContext`. See [ADR-029](../adr/029-hexagonal-architecture-dill.md).
-   **Run server**: `cargo run --bin mcb` or `make build` then run the binary.

## 📞 Getting Help

-   **Issues**: Use GitHub Issues for bugs and features
-   **Discussions**: Use GitHub Discussions for questions
-   **Documentation**: Check docs/architecture/ARCHITECTURE.md for technical details

## Code of Conduct

Be respectful and constructive in all interactions. Focus on improving the project and helping fellow contributors.

---

## Cross-References

### Architecture (v0.1.4)

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) - System overview
-   **ADR-029**: [Hexagonal Architecture with dill](../adr/029-hexagonal-architecture-dill.md) - DI, handles, linkme
-   **ADR-013**: [Clean Architecture Crate Separation](../adr/013-clean-architecture-crate-separation.md) - Eight-crate structure
-   **Implementation Status**: [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) - Current state

### Operations

-   **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
-   **CI/CD & Release**: [CI_RELEASE.md](../operations/CI_RELEASE.md) - Pre-commit hooks, GitHub Actions, release process
-   **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
-   **Roadmap**: [ROADMAP.md](./ROADMAP.md)
