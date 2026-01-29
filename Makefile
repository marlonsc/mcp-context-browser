# =============================================================================
# MCP Context Browser - Makefile v2.0
# =============================================================================
# Modular structure with single-action verbs
# Each verb does ONE thing. Use prerequisites for composition.
# Run `make help` for command reference
# =============================================================================

# =============================================================================
# Global Parameters (limitations only, not for multi-function)
# =============================================================================
export RELEASE ?= 1
export SCOPE ?= all
export FIX ?= 0
export STRICT ?= 0
export QUICK ?= 0
export LCOV ?= 0
export CI_MODE ?= 0
export BUMP ?=
export TEST_THREADS ?= 0

# Rust 2024 Edition lints
export RUST_2024_LINTS := -D unsafe_op_in_unsafe_fn -D rust_2024_compatibility -W static_mut_refs

# =============================================================================
# Include Modules
# =============================================================================
include make/Makefile.core.mk
include make/Makefile.quality.mk
include make/Makefile.docs.mk
include make/Makefile.dev.mk
include make/Makefile.release.mk
include make/Makefile.git.mk
include make/Makefile.help.mk

# Default target
.DEFAULT_GOAL := help

# =============================================================================
# CI (compound targets using prerequisites)
# =============================================================================
.PHONY: ci ci-full ci-local

ci: ## Complete CI pipeline (lint + test + validate + audit)
	@echo "Running CI pipeline..."
	@$(MAKE) lint CI_MODE=1
	@$(MAKE) test SCOPE=all
	@$(MAKE) validate QUICK=1
	@$(MAKE) audit
	@echo "CI pipeline passed!"

ci-full: ## Full CI validation matching GitHub Actions (lint + test + validate + audit + docs + coverage)
	@echo "==================================================================="
	@echo "Running FULL CI pipeline (matches GitHub Actions exactly)"
	@echo "==================================================================="
	@echo ""
	@echo "Step 1/6: Linting (Rust 2024 compliance)..."
	@$(MAKE) lint CI_MODE=1
	@echo ""
	@echo "Step 2/6: Unit and integration tests (4 threads to prevent timeouts)..."
	@$(MAKE) test SCOPE=all TEST_THREADS=4
	@echo ""
	@echo "Step 3/6: Architecture validation (strict)..."
	@$(MAKE) validate STRICT=1
	@echo ""
	@echo "Step 4/6: Golden acceptance tests (2 threads for acceptance tests)..."
	@$(MAKE) test SCOPE=golden TEST_THREADS=2
	@echo ""
	@echo "Step 5/6: Security audit..."
	@$(MAKE) audit
	@echo ""
	@echo "Step 6/6: Documentation build..."
	@$(MAKE) docs
	@echo ""
	@echo "==================================================================="
	@echo "✓ FULL CI pipeline passed!"
	@echo "==================================================================="

ci-local: ## Local pre-commit validation (lint + validate QUICK, no tests)
	@echo "Running LOCAL pre-commit validation..."
	@echo "  → Linting (Rust 2024 compliance)..."
	@$(MAKE) lint CI_MODE=1
	@echo "  → Architecture validation (QUICK mode, skipping tests)..."
	@$(MAKE) validate QUICK=1
	@echo "✓ Pre-commit validation passed!"
	@echo ""
	@echo "Tip: Run 'make ci-full' for complete CI checks including all tests."
