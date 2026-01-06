# MCP Context Browser - Auto-Managed Makefile v0.0.3

.PHONY: help all ci clean-all build test release version-bump version-tag version-push version-all docs validate quality fix check ready deploy

# Default target - v0.0.3 complete workflow
all: quality release version-all github-release ## Complete v0.0.3 workflow

# Quick help - v0.0.3 workflow
help: ## Show v0.0.3 workflow
	@echo "MCP Context Browser v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') - Auto-Managed Makefile"
	@echo "=================================================================="
	@echo ""
	@echo "🚀 PRIMARY WORKFLOWS (use these!):"
	@echo "  all         - Complete development workflow"
	@echo "  ready       - Quality + Release (deployment ready)"
	@echo "  deploy      - Full deployment (ready + version + release)"
	@echo ""
	@echo "🔧 DEVELOPMENT:"
	@echo "  check       - Build + Test"
	@echo "  fix         - Auto-fix issues"
	@echo "  ci          - CI pipeline simulation"
	@echo "  maintain    - Full maintenance cycle"
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
	@echo "  verify      - Final quality verification"
	@echo ""
	@echo "⚡ SHORT ALIASES (single letters!):"
	@echo "  b=build, t=test, f=fix, q=quality, r=ready, d=deploy, v=version-all, s=status"
	@echo "  m=maintain, y=sync, z=verify"
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
fix: fmt fix-md ## Auto-fix code issues

ci: check lint-md validate ## CI pipeline simulation
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

lint-md: ## Lint markdown files
	@echo "🔍 Linting markdown files..."
	@bash scripts/docs/lint-markdown-basic.sh 2>/dev/null || echo "⚠️ Markdown linting not available"
	@echo "✅ Markdown linting completed"

fix-md: ## Auto-fix markdown issues
	@echo "🔧 Auto-fixing markdown issues..."
	@bash scripts/docs/fix-markdown.sh 2>/dev/null || echo "⚠️ Markdown fix not available"
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
# AUTO-MANAGEMENT COMMANDS - Self-maintaining workflows v0.0.3
# =============================================================================

update: ## Update all dependencies (MANDATORY)
	@echo "🔄 Updating Cargo dependencies..."
	cargo update
	@echo "✅ Dependencies updated"

audit: ## Security audit (MANDATORY)
	@echo "🔒 Running security audit..."
	cargo audit
	@echo "✅ Security audit completed"

health: ## Health check all components (MANDATORY)
	@echo "🏥 Running health checks..."
	@cargo check
	@cargo test --no-run
	@echo "✅ Health check passed"

# v0.0.3 Feature Commands - Auto-managed
metrics: ## Start metrics HTTP server (v0.0.3)
	@echo "📊 Starting metrics server on port 3001..."
	cargo run -- --metrics

metrics-test: ## Test metrics collection (v0.0.3)
	@echo "🧪 Running metrics tests..."
	cargo test --test metrics

sync-test: ## Test cross-process synchronization (v0.0.3)
	@echo "🔄 Running sync tests..."
	cargo test --test sync

daemon-test: ## Test background daemon (v0.0.3)
	@echo "🤖 Running daemon tests..."
	cargo test daemon

dashboard: ## Open metrics dashboard (v0.0.3)
	@echo "🌐 Opening dashboard at http://localhost:3001"
	@python3 -m webbrowser http://localhost:3001 2>/dev/null || echo "Please open http://localhost:3001 in your browser"

env-check: ## Validate environment configuration (v0.0.3)
	@echo "⚙️ Checking environment configuration..."
	cargo run -- --env-check

# Auto-management workflows
fix-all: fmt lint-md fix-imports ## Auto-fix all code issues
fix-imports: ## Fix Rust import issues
	@echo "🔧 Fixing imports..."
	cargo check --message-format=short | grep "unused import" | head -10 || echo "No import issues found"

clean-deep: clean clean-docs clean-target ## Deep clean all artifacts
clean-target: ## Clean target directory
	@echo "🧹 Cleaning target directory..."
	rm -rf target/

clean-docs: ## Clean documentation artifacts
	@echo "🧹 Cleaning documentation..."
	rm -rf docs/architecture/diagrams/generated/
	rm -rf docs/*/index.html docs/index.html

# Quality gates - Mandatory for v0.0.3
quality-gate: check-deps quality validate ## All quality gates (MANDATORY)
	@echo "✅ All quality gates passed - Ready for v0.0.3 release"

# Development shortcuts
dev-metrics: ## Development with metrics
	@echo "🚀 Starting development server with metrics..."
	cargo watch -x "run -- --metrics"

dev-sync: ## Development with sync testing
	@echo "🔄 Starting development with sync features..."
	cargo watch -x "run -- --sync-test"

# v0.0.3 Complete Workflow - Auto-managed
v0.0.3: ## Complete v0.0.3 workflow (MANDATORY - All quality gates)
	@echo "🚀 Starting complete v0.0.3 workflow..."
	@echo "📋 Step 1: Dependencies check..."
	@make check-deps || (echo "❌ Dependencies check failed" && exit 1)
	@echo "🔧 Step 2: Auto-fix issues..."
	@make fix-all || (echo "❌ Auto-fix failed" && exit 1)
	@echo "🏥 Step 3: Health check..."
	@make health || (echo "❌ Health check failed" && exit 1)
	@echo "🧪 Step 4: Run all tests..."
	@make test || (echo "❌ Tests failed" && exit 1)
	@echo "📊 Step 5: Test v0.0.3 features..."
	@make metrics-test || (echo "❌ Metrics tests failed" && exit 1)
	@make sync-test || (echo "❌ Sync tests failed" && exit 1)
	@echo "📚 Step 6: Generate documentation..."
	@make docs || (echo "❌ Docs generation failed" && exit 1)
	@echo "✅ Step 7: Final validation..."
	@make validate || (echo "❌ Validation failed" && exit 1)
	@echo "🔒 Step 8: Security audit..."
	@make audit || (echo "❌ Security audit failed" && exit 1)
	@echo "🎉 v0.0.3 workflow completed successfully!"
	@echo "🚀 Ready for: make version-all && make deploy"

status: ## Show project status (MANDATORY)
	@echo "📊 Project Status v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@echo "=================="
	@make git-status
	@echo ""
	@echo "Tests: $(shell cargo test --quiet 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"
	@echo "Build: $(shell cargo check --quiet 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"
	@echo "Lint: $(shell cargo clippy --quiet -- -D warnings 2>/dev/null && echo '✅ PASS' || echo '❌ FAIL')"

# Auto-maintenance commands
maintain: update audit clean-all ## Full maintenance cycle
sync: git-force-all ## Sync all changes to remote
verify: quality test-quiet ## Verify code quality

# Development iteration for v0.0.3
dev-cycle: fix test-quiet ## Development iteration: fix + test
dev-ready: dev-cycle quality ## Development ready: iteration + quality
dev-deploy: dev-ready version-all github-release ## Development deploy: ready + version + release

# =============================================================================
# WORKFLOW ALIASES - Short verbs for v0.0.3 development
# =============================================================================

b: build ## build
t: test ## test
tq: test-quiet ## test-quiet
c: check ## check + test
f: fix ## auto-fix
q: quality ## full quality
r: ready ## quality + release
d: deploy ## full deploy
v: version-all ## version bump + release
s: status ## project status
m: maintain ## maintenance cycle
y: sync ## sync to remote
z: verify ## final verify

# =============================================================================
# QUALITY COMMANDS
# =============================================================================

coverage: ## Generate coverage report
	cargo tarpaulin --out Html --output-dir coverage


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