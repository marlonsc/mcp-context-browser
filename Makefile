# =============================================================================
# MCP Context Browser - Makefile v0.1.0
# =============================================================================
# Streamlined, integrated with scripts/docs/*.sh
# Run `make help` for command reference
# =============================================================================

# Include modular makefiles
include make/Makefile.help.mk
include make/Makefile.core.mk
include make/Makefile.quality.mk
include make/Makefile.development.mk
include make/Makefile.release.mk
include make/Makefile.documentation.mk
include make/Makefile.maintenance.mk
include make/Makefile.git.mk
include make/Makefile.aliases.mk

# Default target
.DEFAULT_GOAL := help

# =============================================================================
# PHONY declarations (consolidated)
# =============================================================================
.PHONY: help all
.PHONY: build build-release test test-unit test-integration clean run
.PHONY: check fmt lint fix quality validate audit coverage bench
.PHONY: lint-md fix-md
.PHONY: docs docs-build docs-serve docs-check docs-fix docs-setup docs-metrics docs-sync info
.PHONY: adr-new adr-list adr-check rust-docs diagrams
.PHONY: status commit push tag sync
.PHONY: dev dev-metrics dev-sync setup
.PHONY: docker-up docker-down docker-logs docker-status test-docker
.PHONY: release package github-release
.PHONY: update health maintain
.PHONY: b t c f l q r d s y p D S u a
