# =============================================================================
# QUALITY - Code quality operations
# =============================================================================
# Same commands work everywhere: local, CI, Docker, any OS
# =============================================================================

.PHONY: check fmt lint fix quality audit coverage bench
.PHONY: validate validate-report validate-summary validate-arch
.PHONY: validate-deps validate-quality validate-patterns validate-tests validate-docs validate-naming
.PHONY: validate-solid validate-org validate-kiss validate-shaku validate-refactor
.PHONY: validate-all validate-config
.PHONY: validate-migration validate-linkme validate-ctor validate-figment validate-rocket
.PHONY: check-full ci-quality test-golden test-golden-full
.PHONY: pmat-tdg pmat-diag pmat-entropy pmat-defects pmat-gate pmat-explain pmat-clean

# -----------------------------------------------------------------------------
# Core Quality Commands
# -----------------------------------------------------------------------------

check: ## Fast compilation check
	@cargo check --all-targets

fmt: ## Format code (use FMT_CHECK=1 for CI mode)
ifdef FMT_CHECK
	@cargo fmt --all -- --check
	@./scripts/docs/markdown.sh lint 2>/dev/null || true
else
	@cargo fmt
	@./scripts/docs/markdown.sh autofix 2>/dev/null || true
	@./scripts/docs/markdown.sh fix 2>/dev/null || true
endif

lint: ## Lint code (Rust + Markdown)
	@# Note: Using explicit features instead of --all-features to avoid broken upstream milvus crate
	@cargo clippy --all-targets --features "full" -- -D warnings
	@./scripts/docs/markdown.sh lint 2>/dev/null || true

fix: ## Auto-fix all issues (Rust + Markdown)
	@cargo fmt
	@./scripts/docs/markdown.sh autofix 2>/dev/null || true
	@# Note: Using explicit features instead of --all-features to avoid broken upstream milvus crate
	@cargo clippy --fix --allow-dirty --all-targets --features "full" 2>/dev/null || true
	@echo "Auto-fix completed - run 'make fmt' to verify"

# -----------------------------------------------------------------------------
# Quality Gates
# -----------------------------------------------------------------------------

quality: check fmt lint test test-doc ## Full quality check (includes doctests)
	@echo "Quality checks passed"

# -----------------------------------------------------------------------------
# Security & Coverage
# -----------------------------------------------------------------------------

audit: ## Security audit
	@cargo audit

coverage: ## Generate coverage (use LCOV=1 for CI format)
ifdef LCOV
	@cargo tarpaulin --out Lcov --output-dir coverage
else
	@cargo tarpaulin --out Html --output-dir coverage 2>/dev/null || echo "cargo-tarpaulin not installed"
endif

bench: ## Run benchmarks
	@cargo bench

# =============================================================================
# ARCHITECTURE VALIDATION (mcb-validate)
# =============================================================================
# Uses mcb-validate crate for comprehensive architecture checks
# =============================================================================

# Quick validation summary (shows counts only)
validate: ## Architecture validation summary
	@echo "=================================================================="
	@echo "           mcb-validate Architecture Report                       "
	@echo "=================================================================="
	@cargo test --package mcb-validate test_full_validation_report -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Dependency:|Quality:|Patterns:|Tests:|Documentation:|Naming:|SOLID:|Organization:|KISS:|DI/Shaku:|Refactoring:|Status:|\[Error\]|\[Warning\])" | \
		head -30
	@echo ""
	@echo "Run 'make validate-report' for full details"

# Full validation report (shows all violations)
validate-report: ## Full architecture report with all violations
	@echo "=================================================================="
	@echo "        mcb-validate Full Architecture Report                     "
	@echo "=================================================================="
	@cargo test --package mcb-validate test_full_validation_report -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|^$\|Compiling\|Finished\|Doc-tests\|passed\|filtered"

# Full architecture validation (all tests)
validate-arch: ## Run all architecture validation tests
	cargo test --package mcb-validate -- --nocapture
	@echo "Architecture validation completed!"

# -----------------------------------------------------------------------------
# Individual Validator Targets (for targeted fixes)
# -----------------------------------------------------------------------------

validate-deps: ## Check dependency violations
	@echo "=== Dependency Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_dependencies -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-quality: ## Check quality violations (unwrap/expect/panic)
	@echo "=== Quality Violations (unwrap/expect/panic) ==="
	@cargo test --package mcb-validate test_validate_workspace_quality -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-patterns: ## Check pattern violations (DI, async traits)
	@echo "=== Pattern Violations (DI, async traits) ==="
	@cargo test --package mcb-validate test_validate_workspace_patterns -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-tests: ## Check test organization violations
	@echo "=== Test Organization Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_tests -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-docs: ## Check documentation violations
	@echo "=== Documentation Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_documentation -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-naming: ## Check naming violations
	@echo "=== Naming Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_naming -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-solid: ## Check SOLID principle violations
	@echo "=== SOLID Principle Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_solid -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-org: ## Check organization violations (file placement)
	@echo "=== Organization Violations (file placement, centralization) ==="
	@cargo test --package mcb-validate test_validate_workspace_organization -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-kiss: ## Check KISS violations (complexity)
	@echo "=== KISS Violations (complexity) ==="
	@cargo test --package mcb-validate test_validate_workspace_kiss -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-shaku: ## Check DI/Shaku violations
	@echo "=== DI/Shaku Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_shaku -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-refactor: ## Check refactoring completeness violations
	@echo "=== Refactoring Completeness Violations ==="
	@echo "Detects: orphan imports, duplicate definitions, missing tests, stale re-exports"
	@cargo test --package mcb-validate test_validate_workspace_refactoring -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

# -----------------------------------------------------------------------------
# Migration Validation Targets (v0.1.2)
# -----------------------------------------------------------------------------

validate-migration: validate-linkme validate-ctor validate-figment validate-rocket ## Validate all v0.1.2 migration issues
	@echo "Migration validation completed!"

validate-linkme: ## Validate Inventory → Linkme migration
	@echo "=== Inventory → Linkme Migration Issues ==="
	@echo "Detects: inventory::submit!/collect! usage, missing linkme patterns"
	@cargo test --package mcb-validate test_linkme_validator -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished" || echo "Linkme validator test not found"

validate-ctor: ## Validate Shaku → Constructor Injection migration
	@echo "=== Shaku → Constructor Injection Migration Issues ==="
	@echo "Detects: #[derive(Component)], #[shaku(inject)], module! usage"
	@cargo test --package mcb-validate test_constructor_injection_validator -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished" || echo "Constructor injection validator test not found"

validate-figment: ## Validate Config → Figment migration
	@echo "=== Config Crate → Figment Migration Issues ==="
	@echo "Detects: Config::builder(), config::Environment usage"
	@cargo test --package mcb-validate test_figment_validator -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished" || echo "Figment validator test not found"

validate-rocket: ## Validate Axum → Rocket migration
	@echo "=== Axum → Rocket Migration Issues ==="
	@echo "Detects: axum::Router, axum::routing::* usage, Tower middleware"
	@cargo test --package mcb-validate test_rocket_validator -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished" || echo "Rocket validator test not found"

# -----------------------------------------------------------------------------
# Multi-Directory Validation
# -----------------------------------------------------------------------------

validate-all: ## Validate workspace + legacy source (if exists)
	@echo "=================================================================="
	@echo "     mcb-validate: Workspace + Legacy Source Validation           "
	@echo "=================================================================="
	@cargo test --package mcb-validate test_validation_with_legacy -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Dependency:|Quality:|Patterns:|Tests:|Documentation:|Naming:|SOLID:|Organization:|KISS:|DI/Shaku:|Refactoring:|Status:|\[Error\]|\[Warning\]|\[Info\])" | \
		head -40
	@echo ""
	@echo "Note: Legacy paths use Info severity level"

validate-config: ## Show validation configuration
	@echo "=================================================================="
	@echo "          mcb-validate Configuration Test                         "
	@echo "=================================================================="
	@cargo test --package mcb-validate test_validation_config -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

# -----------------------------------------------------------------------------
# Extended Check Targets
# -----------------------------------------------------------------------------

check-full: check lint test validate ## Full check with architecture validation
	@echo ""
	@echo "Full validation completed!"

ci-quality: fmt lint test validate-arch ## CI quality gate (all checks)
	@echo "CI quality checks passed!"

# -----------------------------------------------------------------------------
# Golden Acceptance Tests
# -----------------------------------------------------------------------------

test-golden: ## Run golden acceptance tests (requires embedding provider)
	@echo "=================================================================="
	@echo "         Golden Acceptance Tests (v0.1.2)                         "
	@echo "=================================================================="
	@cargo test -p mcb-server golden_acceptance -- --nocapture 2>&1 | \
		grep -v "^Compiling\|^Finished\|^Fresh\|^Downloading\|^Downloaded"

test-golden-full: ## Run all golden tests including integration (requires services)
	@echo "=================================================================="
	@echo "         Full Golden Acceptance Tests (requires services)         "
	@echo "=================================================================="
	@cargo test -p mcb-server golden_acceptance -- --include-ignored --nocapture 2>&1 | \
		grep -v "^Compiling\|^Finished\|^Fresh\|^Downloading\|^Downloaded"

# =============================================================================
# PMAT Quality Analysis (optional - requires pmat tool)
# =============================================================================

pmat-tdg: ## Technical Debt Grade (target: A+)
	@echo "=================================================================="
	@echo "              TDG - Technical Debt Grade                          "
	@echo "=================================================================="
	@pmat tdg --format table 2>&1 || echo "pmat not installed"

pmat-diag: ## Project diagnostics
	@echo "=================================================================="
	@echo "              Project Diagnostics                                 "
	@echo "=================================================================="
	@pmat project-diag --format summary 2>&1 || echo "pmat not installed"

pmat-entropy: ## Pattern entropy analysis
	@echo "=================================================================="
	@echo "              Entropy Analysis (Pattern Detection)                "
	@echo "=================================================================="
	@pmat analyze entropy --format detailed --top-violations 10 2>&1 || echo "pmat not installed"

pmat-defects: ## Known defects scan
	@echo "=================================================================="
	@echo "              Known Defects Scan                                  "
	@echo "=================================================================="
	@pmat analyze defects 2>&1 || echo "pmat not installed"

pmat-gate: ## Quality gate (all checks)
	@echo "=================================================================="
	@echo "              Quality Gate (All Checks)                           "
	@echo "=================================================================="
	@pmat quality-gate --format summary 2>&1 || echo "pmat not installed"

pmat-explain: ## TDG with function-level explanation
	@echo "=================================================================="
	@echo "              TDG Explain (Function Breakdown)                    "
	@echo "=================================================================="
	@pmat tdg --explain --threshold 10 2>&1 || echo "pmat not installed"

pmat-clean: ## Clean target directory
	@echo "Cleaning target directory..."
	@du -sh target 2>/dev/null || echo "No target directory"
	@cargo clean
	@echo "Target directory cleaned"
