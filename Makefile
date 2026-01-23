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
# CI (compound target using prerequisites)
# =============================================================================
.PHONY: ci

ci: ## Complete CI pipeline (lint + test + validate + audit)
	@echo "Running CI pipeline..."
	@$(MAKE) lint CI_MODE=1
	@$(MAKE) test SCOPE=all
	@$(MAKE) validate QUICK=1
	@$(MAKE) audit
	@echo "CI pipeline passed!"
