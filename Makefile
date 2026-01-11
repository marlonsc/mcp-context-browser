# MCP Context Browser - Professional Makefile v0.1.0
# Organized, colorized and optimized for developer productivity

# Include all modular makefiles in correct dependency order
include make/Makefile.help.mk
include make/Makefile.core.mk
include make/Makefile.quality.mk
include make/Makefile.development.mk
include make/Makefile.release.mk
include make/Makefile.documentation.mk
include make/Makefile.maintenance.mk
include make/Makefile.git.mk
include make/Makefile.aliases.mk

# Ensure all targets are correctly declared as PHONY
.PHONY: help all build build-release test test-quiet test-unit test-integration test-security test-cache test-metrics \
        clean clean-target clean-docs clean-deep run check fmt fmt-check lint lint-md \
        fix fix-imports quality quality-gate coverage bench validate dev dev-metrics dev-sync \
        setup ci dev-cycle dev-ready dev-deploy docker-up docker-down docker-logs \
        test-integration-docker test-docker-full docker-status release package github-release \
        version-bump version-tag version-push version-all docs docs-auto docs-manual \
        module-docs api-docs status-docs sync-docs sync-docs-update rust-docs index-docs \
        adr-new adr-list diagrams update audit health maintain verify env-check status \
        metrics metrics-test sync-test daemon-test dashboard git-status git-add-all \
        git-commit-force git-push-force git-tag git-force-all sync force-commit \
        b t tq c f q r d v s m y z

# Default target
.DEFAULT_GOAL := help