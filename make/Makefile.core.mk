# =============================================================================
# CORE - Build, test and clean operations
# =============================================================================

.PHONY: build build-release test test-unit test-integration clean run

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------

build: ## Build project (debug mode)
	@cargo build

build-release: ## Build project (release mode)
	@cargo build --release

# -----------------------------------------------------------------------------
# Test
# -----------------------------------------------------------------------------

# Test-specific port (unified: Admin + Metrics + MCP on single port)
# Uses 13001 to avoid conflicts with development server (default 3001)
export MCP_PORT ?= 13001

test: ## Run all tests (uses port 13001 to avoid conflicts)
	@MCP_PORT=$(MCP_PORT) cargo test --all-targets --all-features

test-unit: ## Run unit tests only
	@MCP_PORT=$(MCP_PORT) cargo test --lib --all-features

test-integration: ## Run integration tests only
	@MCP_PORT=$(MCP_PORT) cargo test --test '*'

# -----------------------------------------------------------------------------
# Run
# -----------------------------------------------------------------------------

run: ## Build and run the server
	@cargo run

# -----------------------------------------------------------------------------
# Clean
# -----------------------------------------------------------------------------

clean: ## Clean all build artifacts
	@echo "🧹 Cleaning..."
	@cargo clean
	@rm -rf docs/generated/ docs/build/ coverage/ dist/
	@echo "✅ Clean complete"
