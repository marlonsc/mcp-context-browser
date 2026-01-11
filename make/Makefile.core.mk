# =============================================================================
# CORE - Basic build, test and clean operations
# =============================================================================

.PHONY: build test clean run

# Build
build: ## Build project in debug mode
	cargo build

build-release: ## Build project in release mode
	cargo build --release

# Tests
test: ## Run all tests
	cargo test --all-targets --all-features

test-all: test ## Alias for test

test-quiet: ## Run tests quietly
	cargo test --quiet --all-targets --all-features

test-unit: ## Run only unit tests
	cargo test --lib --all-features

test-integration: ## Run only integration tests
	cargo test --test '*'

test-security: ## Run security tests
	cargo test security

test-cache: ## Run cache tests
	cargo test cache

test-metrics: ## Run metrics tests
	cargo test metrics

# Run
run: ## Build and run the project
	cargo run

# Clean
clean: ## Clean everything
	cargo clean
	rm -rf docs/architecture/diagrams/generated/
	rm -rf target/doc/
	rm -rf docs/build/
	rm -rf coverage/
	rm -rf dist/

clean-target: ## Clean target directory
	@echo "🧹 Cleaning target directory..."
	rm -rf target/

clean-docs: ## Clean documentation artifacts
	@echo "🧹 Cleaning documentation..."
	rm -rf docs/architecture/diagrams/generated/
	rm -rf docs/*/index.html docs/index.html

clean-deep: clean clean-docs clean-target ## Deep clean all artifacts