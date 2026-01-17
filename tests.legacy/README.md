# Testing Strategy and Documentation

## Overview

This test suite provides comprehensive coverage for the MCP Context Browser, implementing multiple testing strategies to ensure code quality, performance, and reliability. Tests are organized by module following the source code structure.

## Test Organization

Tests are organized in a structure that mirrors the `src/` directory (Clean Architecture layers):

```text
tests/
├── unit/                   # Unit tests organized by Clean Architecture layer
│   ├── domain/             # Tests for src/domain/
│   │   ├── chunking/       # Code chunking tests
│   │   ├── ports/          # Port trait tests
│   │   ├── core_types_test.rs
│   │   ├── error_handling_test.rs
│   │   └── validation_test.rs
│   ├── application/        # Tests for src/application/
│   │   └── services_test.rs
│   ├── adapters/           # Tests for src/adapters/
│   │   ├── providers/      # Embedding, vector store, routing
│   │   ├── repository/     # Repository tests
│   │   └── hybrid_search/  # Hybrid search tests
│   ├── infrastructure/     # Tests for src/infrastructure/
│   │   ├── auth/           # Authentication tests
│   │   ├── config/         # Configuration tests
│   │   ├── daemon/         # Daemon tests
│   │   ├── di/             # DI/Shaku tests
│   │   ├── metrics/        # Metrics tests
│   │   ├── snapshot/       # Snapshot tests
│   │   └── sync/           # Sync manager tests
│   ├── server/             # Tests for src/server/
│   │   ├── admin/          # Admin web interface tests
│   │   ├── transport/      # HTTP transport tests
│   │   └── handlers_test.rs
│   ├── property_based_test.rs  # Property-based tests
│   ├── security_test.rs        # Security tests
│   └── unit_test.rs            # General unit tests
├── e2e/                    # End-to-end integration tests
│   └── docker/             # Docker-based tests
├── perf/                   # Performance benchmarks
├── fixtures/               # Test data and helpers
│   ├── artifacts/          # Test data files
│   └── test_helpers.rs     # Shared utilities
└── README.md               # This documentation
```

**Naming Convention**: `{source_file}_test.rs` for test files that correspond to source files.

## Test Categories

### 1. Layer-Specific Unit Tests (`tests/unit/{layer}/`)

Unit tests mirror the Clean Architecture layers:

-   **unit/domain/**: Domain types, ports, validation, and error handling tests
-   **unit/application/**: Business service layer tests (indexing, search, context)
-   **unit/adapters/**: Provider and repository implementation tests
-   **unit/infrastructure/**: Auth, cache, config, events, daemon, and sync tests
-   **unit/server/**: MCP handlers, admin, transport, and protocol tests

### 2. End-to-End Tests (`tests/e2e/`)

Component interaction and end-to-end testing:

-   MCP protocol implementation tests
-   Docker container integration tests
-   Cross-component interaction validation
-   End-to-end request processing
-   Concurrent access patterns
-   System boundary testing

### 3. Benchmark Tests (`tests/perf/`)

Performance measurement with Criterion:

-   Core type operations benchmarking
-   Provider performance characteristics
-   Repository operation benchmarks
-   Memory usage analysis
-   Concurrent operation performance
-   System throughput measurements

### 4. General Unit Tests (`tests/unit/*.rs`)

Cross-cutting unit tests at the root of `tests/unit/`:

-   `property_based_test.rs`: Property-based tests with proptest
-   `security_test.rs`: Security and safety tests
-   `unit_test.rs`: General utility function tests

## Testing Strategy

### TDD (Test-Driven Development)

All new features follow TDD principles:

1.  Write failing test first
2.  Implement minimal code to pass
3.  Refactor while maintaining test coverage

### Coverage Goals

-   **Unit Tests**: 80%+ coverage of individual functions
-   **Integration Tests**: All component interactions tested
-   **Property Tests**: Edge cases and invariants verified
-   **Performance Tests**: Benchmarks for critical paths

### Quality Gates

-   All tests must pass before commits
-   Coverage reports generated and reviewed
-   Performance benchmarks tracked over time
-   Property tests catch edge cases missed by example tests

## Running Tests

### Basic Test Execution

```bash
# Run all tests (organized by module)
cargo test

# Run tests for specific module
cargo test chunking
cargo test config
cargo test core

# Run integration tests
cargo test integration

# Run benchmark tests
cargo test benchmark

# Run unit tests
cargo test unit

# Run with coverage
cargo tarpaulin --out Html

# Run performance benchmarks
cargo bench
```

### Module-Specific Testing

```bash
# Test individual modules
cargo test providers::embedding_providers
cargo test core::core_types
cargo test validation

# Test specific functionality
cargo test chunking::chunking::tests::test_rust_chunking_with_tree_sitter
```

### Integration Testing

```bash
# Run all integration tests
cargo test integration

# Run specific integration tests
cargo test integration::mcp_protocol
cargo test integration::docker
```

### Property-Based Testing

```bash
# Run property tests
cargo test unit::property_based

# Run with more test cases
PROPTEST_CASES=1000 cargo test unit::property_based
```

## Test Organization

### Directory Structure

Tests follow Clean Architecture layers under `tests/unit/`:

```text
tests/
├── unit/                          # Unit tests by Clean Architecture layer
│   ├── domain/                    # Domain layer tests
│   │   ├── chunking/              # Chunking tests
│   │   ├── ports/                 # Port trait tests
│   │   ├── validation/            # Validation tests
│   │   ├── core_types_test.rs
│   │   └── error_handling_test.rs
│   ├── application/               # Application service tests
│   │   └── services_test.rs
│   ├── adapters/                  # Adapter tests
│   │   ├── providers/             # Embedding, vector store, routing
│   │   ├── repository/            # Repository tests
│   │   └── hybrid_search/         # Hybrid search tests
│   ├── infrastructure/            # Infrastructure tests
│   │   ├── auth/                  # Authentication tests
│   │   ├── config/                # Configuration tests
│   │   ├── daemon/                # Daemon tests
│   │   ├── di/                    # DI/Shaku tests
│   │   ├── metrics/               # Metrics tests
│   │   ├── snapshot/              # Snapshot tests
│   │   └── sync/                  # Sync manager tests
│   ├── server/                    # Server tests
│   │   ├── admin/                 # Admin tests
│   │   ├── transport/             # Transport tests
│   │   └── handlers_test.rs
│   ├── property_based_test.rs     # Property-based tests
│   ├── security_test.rs           # Security tests
│   └── unit_test.rs               # General unit tests
├── e2e/                           # End-to-end tests
│   └── docker/                    # Docker integration tests
├── perf/                          # Performance benchmarks
├── fixtures/                      # Test data and helpers
└── README.md                      # This documentation
```

### Naming Conventions

-   `mod.rs`: Module declaration file in each directory
-   `*_test.rs`: Test file corresponding to a source file (preferred)
-   `*_tests.rs`: Test file containing multiple test modules

## Coverage Analysis

### Current Coverage Status

-   **Unit Tests**: Comprehensive coverage of core functionality
-   **Integration**: Component interaction validation
-   **Property Tests**: Edge case and invariant verification
-   **Performance**: Benchmark tracking for optimization

### Coverage Goals by Module

-   Core Types: 95%+ coverage
-   Validation: 90%+ coverage
-   Repository: 85%+ coverage
-   Services: 80%+ coverage
-   Configuration: 85%+ coverage

## Continuous Integration

### Automated Testing

-   All tests run on every commit
-   Coverage reports generated automatically
-   Performance regression detection
-   Property test failure alerts

### Quality Gates

-   Test pass rate: 100%
-   Minimum coverage thresholds
-   Performance benchmark baselines
-   No memory leaks or crashes

## Contributing

### Adding New Tests

1.  Identify the appropriate test category
2.  Follow naming conventions
3.  Include comprehensive documentation
4.  Ensure tests are deterministic
5.  Add performance benchmarks for critical paths

### Test Best Practices

-   Tests should be fast and reliable
-   Use descriptive names that explain the behavior being tested
-   Include edge cases and error conditions
-   Mock external dependencies appropriately
-   Clean up test resources properly

## Troubleshooting

### Common Issues

-   **Flaky Tests**: Ensure tests don't depend on external state
-   **Slow Tests**: Profile and optimize or move to benchmarks
-   **Coverage Gaps**: Add missing test cases
-   **Integration Failures**: Check dependency setup and mocking

### Debug Tools

-   `cargo test -- --nocapture`: See test output
-   `cargo tarpaulin`: Generate coverage reports
-   `cargo bench`: Run performance benchmarks
-   `PROPTEST_CASES=10000 cargo test`: Increase property test iterations

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../docs/architecture/ARCHITECTURE.md)
-   **Contributing**: [CONTRIBUTING.md](../docs/developer/CONTRIBUTING.md)
-   **Examples**: [examples/](../examples/)
-   **Module Documentation**: [docs/modules/](../docs/modules/)
