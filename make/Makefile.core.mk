# =============================================================================
# CORE - Build, test, docs, clean
# =============================================================================
# Parameters: RELEASE, SCOPE (from main Makefile)
# =============================================================================

.PHONY: build test clean

# Test port (avoids conflicts with dev server on 3001)
export MCP_PORT ?= 13001

# Test thread count (parallelization - use fewer threads on CI to reduce timeout issues)
export TEST_THREADS ?= 0

# =============================================================================
# BUILD (RELEASE=1 for release)
# =============================================================================

build: ## Build project (RELEASE=1 for release)
ifeq ($(RELEASE),1)
	@echo "Building release..."
	cargo build --release --features "full"
else
	@echo "Building debug..."
	cargo build --features "full"
endif

# =============================================================================
# TEST (SCOPE=unit|doc|golden|all, TEST_THREADS=N for parallelization)
# =============================================================================

test: ## Run tests (SCOPE=unit|doc|golden|integration|modes|all, TEST_THREADS=N to limit parallelization)
ifeq ($(TEST_THREADS),0)
	# Use default thread count
	TEST_THREADS_FLAG=
else
	# Limit to specified number of threads
	TEST_THREADS_FLAG=--test-threads=$(TEST_THREADS)
endif
ifeq ($(SCOPE),unit)
	@echo "Running unit tests..."
	MCP_PORT=$(MCP_PORT) cargo test --workspace --lib --bins $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),doc)
	@echo "Running doctests..."
	MCP_PORT=$(MCP_PORT) cargo test --doc --workspace $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),golden)
	@echo "Running golden acceptance tests..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server golden_acceptance -- --nocapture $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),integration)
	@echo "Running integration tests..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server --test integration $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),modes)
	@echo "Running operating modes tests..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server operating_modes -- --nocapture $(TEST_THREADS_FLAG)
else
	@echo "Running all tests..."
	MCP_PORT=$(MCP_PORT) cargo test --workspace --features "full" $(TEST_THREADS_FLAG)
endif

# =============================================================================
# CLEAN
# =============================================================================

clean: ## Clean all build artifacts
	@echo "Cleaning..."
	cargo clean
	@echo "Clean complete"
