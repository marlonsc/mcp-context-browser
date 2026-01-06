# MCP Context Browser - Simplified Makefile

.PHONY: help build test docs clean ci setup dev fmt lint release adr-new adr-list diagrams validate git-status git-add-all git-commit-force git-push-force git-force-all force-commit

# Default target
help: ## Show available commands
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'

# =============================================================================
# CORE COMMANDS - Use these!
# =============================================================================

build: ## Build project
	cargo build

test: ## Run all tests
	cargo test

test-quiet: ## Run tests quietly
	cargo test --quiet

docs: ## Generate all documentation
	@echo "🎨 Generating diagrams..."
	@bash scripts/docs/generate-diagrams.sh all
	@echo "🦀 Generating Rust docs..."
	@cargo doc --no-deps --document-private-items
	@echo "📖 Generating docs index..."
	@bash scripts/docs/generate-index.sh
	@echo "✅ Documentation generated"

validate: ## Validate everything
	@echo "🔍 Validating diagrams..."
	@bash scripts/docs/generate-diagrams.sh validate
	@echo "📋 Validating docs structure..."
	@bash scripts/docs/validate-structure.sh
	@echo "🔗 Validating links..."
	@bash scripts/docs/validate-links.sh
	@echo "🔄 Checking sync..."
	@bash scripts/docs/check-sync.sh
	@echo "📋 Validating ADRs..."
	@bash scripts/docs/validate-adrs.sh
	@echo "📝 Linting markdown..."
	@make lint-md
	@echo "✅ All validations passed"

ci: clean validate test build docs ## Run full CI pipeline
	@echo "🚀 CI pipeline completed"

clean: ## Clean everything
	cargo clean
	rm -rf docs/architecture/diagrams/generated/
	rm -rf target/doc/
	rm -rf docs/build/
	rm -rf coverage/
	rm -rf dist/

# =============================================================================
# DEVELOPMENT COMMANDS
# =============================================================================

dev: ## Run development server
	cargo watch -x run

fmt: ## Format code
	cargo fmt

lint: ## Lint code
	cargo clippy -- -D warnings

lint-md: ## Lint markdown files
	@echo "🔍 Linting markdown files..."
	@markdownlint docs/ --config .markdownlint.json || (echo "❌ Markdown linting failed. Run 'make fix-md' to auto-fix issues."; exit 1)
	@echo "✅ Markdown linting passed"

fix-md: ## Auto-fix markdown linting issues
	@echo "🔧 Auto-fixing markdown issues..."
	@bash scripts/docs/fix-markdown.sh
	@markdownlint docs/ --config .markdownlint.json --fix
	@echo "✅ Markdown auto-fix completed"

setup: ## Setup development tools
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	npm install -g markdownlint-cli
	@echo "✅ Development environment ready"

# =============================================================================
# DOCUMENTATION COMMANDS
# =============================================================================

adr-new: ## Create new ADR
	@bash scripts/docs/create-adr.sh

adr-list: ## List ADRs
	@echo "📋 ADRs:"
	@ls -1 docs/architecture/adr/ | grep -E '\.md$$' | sed 's/\.md$$//' | sort

diagrams: ## Generate diagrams only
	@bash scripts/docs/generate-diagrams.sh all

# =============================================================================
# RELEASE COMMANDS
# =============================================================================

release: test build-release package ## Create release

build-release: ## Build release binary
	cargo build --release

package: ## Package release
	@mkdir -p dist
	@cp target/release/mcp-context-browser dist/
	@cp docs/user-guide/README.md dist/README.md
	@cp LICENSE dist/
	@cd dist && tar -czf mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz mcp-context-browser README.md LICENSE
	@echo "📦 Release created: dist/mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz"

github-release: release ## Create GitHub release
	@echo "🚀 Creating GitHub release v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')..."
	@gh release create v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') \
		--title "MCP Context Browser v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')" \
		--notes "Release v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') - TDD Complete Implementation" \
		dist/mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz
	@echo "✅ GitHub release created successfully!"

# =============================================================================
# QUALITY COMMANDS
# =============================================================================

coverage: ## Generate coverage report
	cargo tarpaulin --out Html --output-dir coverage

audit: ## Security audit
	cargo audit

bench: ## Run benchmarks
	cargo bench

quality: fmt lint lint-md test audit validate ## Run all quality checks

# =============================================================================
# GIT COMMANDS - Force commit operations
# =============================================================================

git-status: ## Show git repository status
	@echo "Git repository status:"
	@git status --short

git-add-all: ## Add all changes to git
	@echo "Adding all changes to git..."
	@git add -A
	@echo "All changes added"

git-commit-force: ## Force commit all changes
	@echo "Committing all changes with force..."
	@git commit --allow-empty -m "Force commit: $(shell date '+%Y-%m-%d %H:%M:%S') - Automated update" || echo "No changes to commit"

git-push-force: ## Force push to remote repository
	@echo "Pushing changes with force..."
	@git push --force-with-lease origin main || git push --force origin main
	@echo "Changes pushed successfully"

git-tag: ## Create and push git tag
	@echo "Creating and pushing tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)..."
	@git tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)
	@git push origin v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)
	@echo "Tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2) created and pushed!"

git-force-all: git-add-all git-commit-force git-push-force ## Add, commit and push all changes with force
	@echo "Force commit and push completed!"

force-commit: ## Run force commit script (alternative method)
	@echo "Running force commit script..."
	@bash scripts/force-commit.sh