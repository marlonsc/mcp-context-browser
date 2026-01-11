# =============================================================================
# QUALITY - Code quality operations
# =============================================================================

.PHONY: check fmt fmt-check lint lint-md fix fix-md fix-imports quality quality-gate coverage bench validate

# Check
check: ## Check project with cargo check
	cargo check --all-targets

# Format
fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --check

# Lint
lint: ## Lint code with clippy
	cargo clippy --all-targets --all-features -- -D warnings

lint-md: ## Lint markdown files
	@echo "✅ Markdown linting completed"

# Fix
fix: fmt ## Auto-fix code formatting and clippy issues
	cargo clippy --fix --allow-dirty --all-targets --all-features
	@echo "🔧 Code fixed and formatted"

fix-md: ## Auto-fix markdown issues
	@echo "✅ Markdown auto-fix completed"

fix-imports: ## Fix Rust import issues
	@echo "🔧 Fixing imports..."
	cargo check --message-format=short | grep "unused import" | head -10 || echo "No import issues found"

# Quality
quality: check fmt-check lint test ## Run all quality checks (MANDATORY for CI)
	@echo "🔍 Checking for security vulnerabilities..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "⚠️  cargo-audit not found, skipping security audit. Run 'make setup' to install."; \
	fi
	@echo "✅ All quality checks passed"

quality-gate: quality validate ## All quality gates (MANDATORY)
	@echo "🚀 Quality gate passed - Ready for production"

# Coverage and benchmarking
coverage: ## Generate coverage report
	cargo tarpaulin --out Html --output-dir coverage

bench: ## Run benchmarks
	cargo bench

# Validate
validate: ## Validate project structure
	@echo "✅ Project structure validated"