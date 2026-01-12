# =============================================================================
# QUALITY - Code quality operations
# =============================================================================
# Same commands work everywhere: local, CI, Docker, any OS
# =============================================================================

.PHONY: check fmt lint fix quality validate audit coverage bench

# -----------------------------------------------------------------------------
# Core Quality Commands
# -----------------------------------------------------------------------------

check: ## Fast compilation check
	@cargo check --all-targets

fmt: ## Format code (use FMT_CHECK=1 for CI mode)
ifdef FMT_CHECK
	@cargo fmt --all -- --check
else
	@cargo fmt
	@./scripts/docs/markdown.sh fix 2>/dev/null || true
endif

lint: ## Lint code (Rust + Markdown)
	@cargo clippy --all-targets --all-features -- -D warnings
	@./scripts/docs/markdown.sh lint 2>/dev/null || true

fix: ## Auto-fix all issues
	@cargo fmt
	@cargo clippy --fix --allow-dirty --all-targets --all-features 2>/dev/null || true

# -----------------------------------------------------------------------------
# Quality Gates
# -----------------------------------------------------------------------------

quality: check fmt lint test ## Full quality check
	@echo "✅ Quality checks passed"

validate: quality audit docs-check ## Complete validation
	@echo "✅ All validations passed"

# -----------------------------------------------------------------------------
# Security & Coverage
# -----------------------------------------------------------------------------

audit: ## Security audit
	@cargo audit

coverage: ## Generate coverage (use LCOV=1 for CI format)
ifdef LCOV
	@cargo tarpaulin --out Lcov --output-dir coverage
else
	@cargo tarpaulin --out Html --output-dir coverage 2>/dev/null || echo "⚠️  cargo-tarpaulin not installed"
endif

bench: ## Run benchmarks
	@cargo bench
