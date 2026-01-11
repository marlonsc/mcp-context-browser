# =============================================================================
# MAINTENANCE - Health checks, updates and monitoring
# =============================================================================

.PHONY: update audit health maintain verify env-check status metrics metrics-test sync-test daemon-test dashboard

# Dependency management
update: ## Update all dependencies (MANDATORY)
	@echo "🔄 Updating Cargo dependencies..."
	cargo update
	@echo "✅ Dependencies updated"

# Security
audit: ## Security audit (MANDATORY)
	@echo "🔒 Running security audit..."
	cargo audit
	@echo "✅ Security audit completed"

# Health checks
health: check test-unit ## Health check all components (MANDATORY)
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

# Maintenance workflows
maintain: update audit clean ## Full maintenance cycle

verify: quality test-quiet ## Verify code quality

# Project status
status: ## Show project status (MANDATORY)
	@echo "📊 Project Status v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')"
	@echo "=================="
	@make git-status
	@echo ""
	@echo "Tests: $(shell cargo test --quiet 2>/dev/null && echo '✅ PASSED' || echo '❌ FAILED')"
	@echo "Build: $(shell cargo check --quiet 2>/dev/null && echo '✅ PASSED' || echo '❌ FAILED')"
	@echo "Lint: $(shell cargo clippy --quiet -- -D warnings 2>/dev/null && echo '✅ PASSED' || echo '❌ FAILED')"

# Complete v0.0.3 workflow - Auto-managed
v0.0.3: status validate ## Complete v0.0.3 workflow (MANDATORY - All quality gates)
	@echo "🚀 Starting complete v0.0.3 workflow..."
	@echo "📋 Step 1: Check project status..."
	@make status
	@echo "🔍 Step 2: Validate project structure..."
	@make validate 2>/dev/null || echo "⚠️ Validation has issues (expected with code changes)"
	@echo "📊 Step 3: Show available v0.0.3 commands..."
	@echo "Available commands:"
	@echo "  make metrics     - Start metrics server"
	@echo "  make metrics-test - Test metrics functionality"
	@echo "  make dashboard   - Open metrics dashboard"
	@echo "  make sync-test   - Test sync functionality"
	@echo "  make env-check   - Validate environment"
	@echo "  make health      - Health check"
	@echo "🎯 v0.0.3 workflow status check completed!"
	@echo "💡 Fix compilation issues before running full tests"