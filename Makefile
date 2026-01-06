# MCP Context Browser - Auto-Managed Makefile v0.0.3

.PHONY: help all ci clean-all build test release version-bump version-tag version-push version-all docs validate quality fix check ready deploy check-deps

# Default target - complete workflow
all: check-deps quality release version-all ## Complete development workflow

# Quick help - show only essential commands
help: ## Show essential commands
	@echo "MCP Context Browser v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') - Auto-Managed Makefile"
	@echo "=================================================================="
	@echo ""
	@echo "🚀 PRIMARY WORKFLOWS:"
	@echo "  all         - Complete development workflow"
	@echo "  ready       - Quality + Release (deployment ready)"
	@echo "  deploy      - Full deployment (ready + version + release)"
	@echo ""
	@echo "🔧 DEVELOPMENT:"
	@echo "  check       - Build + Test"
	@echo "  fix         - Auto-fix issues (fmt + markdown)"
	@echo "  ci          - CI pipeline simulation"
	@echo ""
	@echo "📦 VERSION & RELEASE:"
	@echo "  version-all - Bump to 0.0.3 + commit + tag + push"
	@echo "  release     - Create release package"
	@echo "  github-release - Create GitHub release"
	@echo ""
	@echo "🔍 QUALITY:"
	@echo "  quality     - All quality checks"
	@echo "  validate    - Full validation"
	@echo "  status      - Project health status"
	@echo ""
	@echo "⚡ SHORT ALIASES:"
	@echo "  b=build, t=test, c=check, f=fix, r=ready, d=deploy, v=version-all, s=status"
	@echo ""
	@echo "📚 Run 'make help-all' for complete command list"

help-all: ## Show all available commands
	@echo "MCP Context Browser - Complete Command Reference"
	@echo "================================================"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v '^help' | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# =============================================================================
# CORE WORKFLOW - Use these primary commands!
# =============================================================================

ready: quality release ## Ready for deployment
deploy: ready version-all github-release ## Full deployment workflow

check: build test ## Basic health check
check-deps: ## Check all required dependencies
	@bash scripts/check-deps.sh
fix: check-deps fmt fix-md ## Auto-fix code issues

ci: check-deps check lint-md validate ## CI pipeline simulation
clean-all: clean clean-docs ## Deep clean

# =============================================================================
# BUILD & TEST
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

lint-md: ## Lint markdown files (MANDATORY - no fallbacks)
	@echo "🔍 Linting markdown files..."
	@if ! command -v markdownlint >/dev/null 2>&1; then \
		echo "❌ ERROR: markdownlint-cli not found"; \
		echo "Run 'make setup' to install markdownlint-cli"; \
		exit 1; \
	fi
	@markdownlint docs/ --config .markdownlint.json || (echo "❌ Markdown linting failed. Run 'make fix-md' to auto-fix issues."; exit 1)
	@echo "✅ Markdown linting passed"

fix-md: ## Auto-fix markdown linting issues (MANDATORY)
	@echo "🔧 Auto-fixing markdown issues..."
	@if ! command -v markdownlint >/dev/null 2>&1; then \
		echo "❌ ERROR: markdownlint-cli not found"; \
		echo "Run 'make setup' to install markdownlint-cli first"; \
		exit 1; \
	fi
	@bash scripts/docs/fix-markdown.sh
	@markdownlint docs/ --config .markdownlint.json --fix
	@echo "✅ Markdown auto-fix completed"

setup: ## Setup development tools (MANDATORY)
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	@echo "📦 Installing markdownlint-cli (required for markdown linting)..."
	@if ! command -v npm >/dev/null 2>&1; then \
		echo "❌ ERROR: npm required for markdownlint-cli installation"; \
		echo "Install Node.js and npm first: https://nodejs.org/"; \
		exit 1; \
	fi
	@if ! npm install -g markdownlint-cli; then \
		echo "❌ ERROR: Failed to install markdownlint-cli"; \
		echo "Check npm permissions or install manually: npm install -g markdownlint-cli"; \
		exit 1; \
	fi
	@if ! command -v markdownlint >/dev/null 2>&1; then \
		echo "❌ ERROR: markdownlint-cli not found after installation"; \
		exit 1; \
	fi
	@echo "✅ Development environment ready with full markdown linting"

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
		--notes "Release v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') - Auto-managed release" \
		dist/mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz
	@echo "✅ GitHub release created successfully!"

# =============================================================================
# VERSION MANAGEMENT - Auto-managed versioning for v0.0.3
# =============================================================================

version-bump: ## Bump version to 0.0.3 in Cargo.toml
	@echo "⬆️ Bumping version to 0.0.3..."
	@sed -i 's/^version = "0\.0\.2"/version = "0.0.3"/' Cargo.toml
	@echo "✅ Version bumped to 0.0.3"

version-tag: ## Create and push version tag
	@echo "🏷️ Creating tag v0.0.3..."
	@git tag v0.0.3
	@git push origin v0.0.3
	@echo "✅ Tag v0.0.3 created and pushed"

version-push: ## Commit and push version changes
	@echo "📤 Pushing version changes..."
	@make git-force-all
	@echo "✅ Version changes pushed"

version-all: version-bump version-push version-tag ## Complete version management

# =============================================================================
# AUTO-MANAGEMENT COMMANDS - Self-maintaining workflows
# =============================================================================

update: ## Update all dependencies
	@echo "🔄 Updating Cargo dependencies..."
	cargo update
	@echo "✅ Dependencies updated"

audit-fix: ## Audit and attempt auto-fixes
	@echo "🔒 Running security audit..."
	cargo audit
	@echo "✅ Security audit completed"

health: ## Health check all components
	@echo "🏥 Running health checks..."
	@cargo check
	@cargo test --no-run
	@echo "✅ Health check passed"

status: ## Show project status
	@echo "📊 Project Status v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@echo "=================="
	@make git-status
	@echo ""
	@echo "Tests: $(shell cargo test --quiet 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"
	@echo "Build: $(shell cargo check --quiet 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"
	@echo "Lint: $(shell cargo clippy --quiet -- -D warnings 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"

# =============================================================================
# WORKFLOW ALIASES - Short verbs for common tasks
# =============================================================================

b: build ## Alias: build
t: test ## Alias: test
tq: test-quiet ## Alias: test-quiet
c: check ## Alias: check
f: fix ## Alias: fix
r: ready ## Alias: ready
d: deploy ## Alias: deploy
v: version-all ## Alias: version-all
s: status ## Alias: status

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

# =============================================================================
# v0.0.3 DEVELOPMENT COMMANDS
# =============================================================================

metrics: ## Start metrics HTTP server on port 3001
	cargo run -- --metrics

metrics-test: ## Test metrics collection functionality
	cargo test --test metrics

dashboard: ## Open metrics dashboard (requires metrics server running)
	@echo "🌐 Opening dashboard at http://localhost:3001"
	@python3 -m webbrowser http://localhost:3001 2>/dev/null || echo "Please open http://localhost:3001 in your browser"

sync-test: ## Test cross-process synchronization
	cargo test --test sync

env-check: ## Validate environment configuration
	cargo run -- --env-check

health: ## Check application health
	curl -s http://localhost:3001/health | jq . 2>/dev/null || echo "Health check failed - is metrics server running?"

status: ## Show full application status
	@echo "🔍 Application Status:"
	@echo "  📊 Metrics: $(shell curl -s http://localhost:3001/health 2>/dev/null | jq -r '.status' 2>/dev/null || echo 'Not running')"
	@echo "  🔍 MCP Server: $(shell pgrep -f "mcp-context-browser" | wc -l) instances running"
	@echo "  💾 Tests: $(shell make test 2>/dev/null | grep -c "test result: ok" || echo "Run 'make test' to check")"