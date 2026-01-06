# Contributing to MCP Context Browser

Thank you for your interest in contributing! This guide helps you get started with development.

## 🚀 Getting Started

### Prerequisites

-   **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
-   **Git**: Version control system

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

3.  **Make Changes**: Implement your feature or fix
4.  **Test Changes**: Ensure tests pass

   ```bash
   cargo test
   ```

5.  **Submit PR**: Create pull request with clear description

## 📝 Coding Standards

### Rust Guidelines

-   Follow [The Rust Programming Language](https://doc.rust-lang.org/book/) conventions
-   Use `rustfmt` for formatting: `cargo fmt`
-   Follow `clippy` suggestions: `cargo clippy`
-   Write idiomatic Rust code

### Code Structure

```
src/
├── core/           # Core types and error handling
├── providers/      # External service integrations
├── services/       # Business logic
├── server/         # MCP protocol implementation
└── main.rs         # Application entry point
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

-   [ ] Tests pass: `cargo test`
-   [ ] Code formats correctly: `cargo fmt --check`
-   [ ] No linting errors: `cargo clippy -- -D warnings`
-   [ ] CI checks pass: `make ci`
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

## 📞 Getting Help

-   **Issues**: Use GitHub Issues for bugs and features
-   **Discussions**: Use GitHub Discussions for questions
-   **Documentation**: Check docs/architecture/ARCHITECTURE.md for technical details

## 🙏 Code of Conduct

Be respectful and constructive in all interactions. Focus on improving the project and helping fellow contributors.
