# ADR 020: Testing Strategy for Integrated Code

## Status

**Accepted** (v0.2.0 - Structure, v0.3.0 - Migration)
**Date**: 2026-01-14

## Context

PMAT has 4600+ tests. MCB has 308+ tests. Integration must preserve both.

## Decision

**Three-tier testing strategy**:

### Tier 1: Unit Tests

**Location**: `crates/<crate>/tests/` or `libs/<lib>/tests/`

**Pattern**:

```rust
// crates/mcb-application/tests/search_service_test.rs — HISTORICAL; DI is now dill (ADR-029)

#[tokio::test]
async fn test_search_returns_relevant_results() {
    // Use null providers from mcb-infrastructure
    let container = DiContainerBuilder::new().build().await.unwrap();
    let service = SearchService::new(
        container.resolve::<Arc<dyn EmbeddingProvider>>(),
        container.resolve::<Arc<dyn VectorStoreProvider>>(),
    );

    // Test with mock data
    let results = service.search("query").await.unwrap();
    assert!(!results.is_empty());
}
```

### Tier 2: Integration Tests

**Location**: `crates/mcb-server/tests/integration/`

**Pattern**:

```rust
// crates/mcb-server/tests/integration/full_flow_test.rs

#[tokio::test]
async fn test_index_and_search_flow() {
    let app = TestApp::new().await;

    // Index sample code
    app.index("sample/").await.unwrap();

    // Search
    let results = app.search("function definition").await.unwrap();
    assert!(results.len() > 0);
}
```

### Tier 3: Property-Based Tests

**Location**: `libs/code-metrics/tests/` (ported from PMAT)

**Pattern**:

```rust
// libs/code-metrics/tests/complexity_properties.rs

proptest! {
    #[test]
    fn complexity_is_non_negative(code in arbitrary_code()) {
        let analyzer = ComplexityAnalyzer::new();
        let result = analyzer.analyze(&code);
        prop_assert!(result.cyclomatic >= 0);
    }
}
```

## v0.1.1 Test Organization

Current test structure in eight-crate workspace:

```
crates/
├── mcb-domain/tests/           # Domain logic tests
├── mcb-application/tests/      # Service tests
├── mcb-providers/tests/        # Provider tests
├── mcb-infrastructure/tests/   # DI and config tests
├── mcb-server/tests/           # Server and integration tests
│   ├── admin/                  # Admin API tests
│   ├── handlers/               # MCP handler tests
│   └── test_utils/             # Test fixtures
└── mcb-validate/tests/         # Validator tests
```

**Current test count**: 308+ tests (100% pass rate)

## Test Migration Plan

### v0.2.0 (Structure)

-   Define test directory structure for libs/
-   Create test utilities for PMAT code
-   Port infrastructure tests

### v0.3.0 (Migration)

-   Port 1000+ PMAT tests for complexity, TDG, SATD
-   Update tests to use MCB types
-   Target: 1500+ total tests

### v0.4.0 (Extended)

-   Port 1500+ additional PMAT tests
-   Target: 3000+ total tests

### v1.0.0 (Complete)

-   5390+ total tests
-   Full coverage of all features

## Consequences

**Positive**:

-   Preserved test coverage
-   Clear test organization
-   Property-based testing for edge cases

**Negative**:

-   CI time increase (~5 min → ~15 min)

**Mitigation**:

-   Parallel test execution
-   Test categorization (quick/full)
-   CI caching

## Related ADRs

-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Test container setup
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Test location per crate
-   [ADR-017: Phased Feature Integration](017-phased-feature-integration.md) - Test migration timeline

---

*Updated 2026-01-17 - Reflects v0.1.2 test structure*
