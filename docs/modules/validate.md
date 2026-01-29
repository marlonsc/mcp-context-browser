# Validation Module

**Source**: `crates/mcb-validate/src/`
**Crate**: `mcb-validate`
**Files**: 50+
**Lines of Code**: ~8,000

## Overview

The validation module provides comprehensive architecture enforcement and code quality validation for the MCP Context Browser project. It implements a multi-phase validation pipeline that ensures Clean Architecture compliance, code quality standards, and architectural decision record (ADR) adherence.

## Architecture

The validation system follows a layered approach with seven verified phases:

```
Validation Pipeline (Pure Rust):
┌─────────────────────────────────────────────┐
│ YAML Rules → Rule Loader → Rule Engine     │
│                                             │
│ Layer 1: Linters (Clippy/Ruff) ✅ Verified │
│ Layer 2: AST (Tree-sitter) ✅ Verified     │
│ Layer 3: Rule Engines ✅ Verified          │
│ Layer 4: Metrics (RCA) ✅ Verified         │
│ Layer 5: Duplication ✅ Verified           │
│ Layer 6: Architecture ✅ Verified          │
│ Layer 7: Integration ✅ Verified           │
│                                             │
│ Output: Unified Violation Interface        │
└─────────────────────────────────────────────┘
```

## Key Components

### Linters (`linters/`)

Code quality linting via external tools:

-   **Clippy**: Rust linter for common mistakes and style issues
-   **Ruff**: Python linter (for Python code analysis)
-   **Status**: ✅ 17/17 tests pass

### AST Queries (`ast/`)

Tree-sitter based AST parsing and querying:

-   `query.rs` - AST query execution
-   `decoder.rs` - AST node decoding
-   `languages.rs` - Language support (Rust, Python, JS, TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
-   `mod.rs` - Module exports
-   **Status**: ✅ 26/26 tests pass

### Rule Engines (`engines/`)

Multiple rule engine implementations:

-   `expression_engine.rs` - evalexpr-based expression evaluation
-   `rete_engine.rs` - RETE algorithm for pattern matching
-   `router.rs` - Rule routing and selection
-   `hybrid_engine.rs` - Combined engine approach
-   `rust_rule_engine.rs` - Rust-specific rule engine
-   `rusty_rules_engine.rs` - Rusty-rules integration
-   `validator_engine.rs` - Validator trait implementation
-   **Status**: ✅ 30/30 tests pass

### Metrics (`metrics/`)

Code metrics analysis using Rust-code-analysis:

-   `analyzer.rs` - Metrics computation
-   `rca_analyzer.rs` - Rust-code-analysis integration (feature-gated)
-   `thresholds.rs` - Metric threshold definitions
-   **Supported Metrics**:
    -   Cyclomatic Complexity
    -   Cognitive Complexity
    -   Halstead Volume/Difficulty/Effort
    -   Maintainability Index
    -   SLOC/PLOC/LLOC/CLOC
-   **Status**: ✅ 9/9 tests pass

### Duplication Detection (`duplication/`)

Code clone detection using Rabin-Karp algorithm:

-   `detector.rs` - Clone detection logic
-   `fingerprint.rs` - Token fingerprinting
-   `thresholds.rs` - Duplication type definitions
-   **Clone Types**:
    -   Type 1: Exact clones (100% identical)
    -   Type 2: Renamed clones (identifiers changed, 95%+ similarity)
    -   Type 3: Gapped clones (small modifications, 80%+ similarity)
    -   Type 4: Semantic clones (future, 70%+ similarity)
-   **Status**: ✅ 11/11 tests pass

### Clean Architecture (`clean_architecture.rs`)

Architecture rule enforcement:

-   **CA001**: Domain layer independence
-   **CA002**: Application layer boundaries
-   **CA003**: Domain traits only
-   **CA004**: Handler dependency injection
-   **CA005**: Entity identity requirements
-   **CA006**: Value object immutability
-   **CA007**: Infrastructure cannot import concrete types from Application
-   **CA008**: Application must import ports from mcb-domain
-   **CA009**: Infrastructure must NOT depend on Application layer
-   **Status**: ✅ 11/11 tests pass

### Rules (`rules/`)

YAML-based rule definitions:

-   `yaml_loader.rs` - Rule loading from YAML files
-   `yaml_validator.rs` - Rule schema validation
-   `registry.rs` - Rule registry and lookup
-   `templates/` - Rule templates for common patterns
-   **Rule Categories**:
    -   `clean-architecture/` - CA001-CA009
    -   `migration/` - 12 migration detection rules (inventory→linkme, shaku→dill, etc.)
    -   `quality/` - Code quality rules
    -   `metrics/` - Metric thresholds
    -   `duplication/` - Clone detection rules
    -   `testing/` - Test organization rules
    -   `solid/` - SOLID principle enforcement

### Integration Tests (`tests/integration/`)

Comprehensive integration test suite:

-   `integration_linters.rs` - Linter integration tests
-   `integration_ast.rs` - AST query integration tests
-   `integration_engines.rs` - Rule engine integration tests
-   `integration_rca_metrics.rs` - Metrics integration tests
-   `integration_duplication.rs` - Duplication detection tests
-   `integration_architecture.rs` - Architecture validation tests
-   `integration_full.rs` - End-to-end validation pipeline tests
-   **Status**: ✅ 14/14 integration tests pass

### Benchmarks (`benches/`)

Performance benchmarks:

-   `validation_benchmark.rs` - 7 benchmark groups:
    -   unwrap_detection
    -   tokenization
    -   duplication_analysis
    -   architecture_validation
    -   report_generation
    -   config
    -   scalability

## Usage

### Command Line

```bash
# Run all validation rules
make validate

# Quick validation (skip tests)
make validate QUICK=1

# Strict validation
make validate STRICT=1
```

### Programmatic API

```rust
use mcb_validate::{ValidatorRegistry, ValidationConfig};

let config = ValidationConfig::default();
let registry = ValidatorRegistry::new();
let violations = registry.validate_all(&config)?;
```

## Validation Status

**Phases 1-7**: ✅ **All VERIFIED** (v0.1.4)

-   **Total Tests**: 750+ in mcb-validate crate
-   **Project-Wide Tests**: 1634+ (includes all crates)
-   **Verification Date**: 2026-01-28
-   **Architecture Violations**: 0

## File Structure

```text
crates/mcb-validate/src/
├── ast/                    # AST parsing and queries
├── engines/                # Rule engines
├── linters/                # External linter integration
├── metrics/                # Code metrics analysis
├── duplication/            # Clone detection
├── rules/                  # YAML rule system
├── clean_architecture.rs   # Architecture validation
├── async_patterns.rs       # Async pattern detection
├── solid.rs                # SOLID principle checks
├── scan.rs                 # File scanning
└── lib.rs                  # Public API

crates/mcb-validate/rules/
├── clean-architecture/     # CA001-CA009 rules
├── migration/              # 12 migration rules
├── quality/                # Quality gates
├── metrics/                # Metric thresholds
├── duplication/            # Clone detection rules
└── templates/              # Rule templates
```

## Related Documentation

-   [Architecture Overview](../architecture/ARCHITECTURE.md#validation-layer) - Validation layer details
-   [Implementation Status](../developer/IMPLEMENTATION_STATUS.md) - Detailed traceability
-   [ADR-013](../adr/013-clean-architecture-crate-separation.md) - Clean Architecture separation
-   [ADR-029](../adr/029-hexagonal-architecture-dill.md) - DI architecture (CA007-CA009)

---

**Last Updated**: 2026-01-28
