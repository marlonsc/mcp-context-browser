# =============================================================================
# DOCS - Documentation management
# =============================================================================
# Professional, minimal verb set - each verb does ONE thing
# =============================================================================

.PHONY: docs docs-serve docs-check docs-setup docs-sync docs-build rust-docs diagrams adr adr-new docs-lint docs-validate docs-auto docs-fix docs-metrics

# Path to mdbook
MDBOOK := $(HOME)/.cargo/bin/mdbook

# =============================================================================
# DOCS - Main documentation target
# =============================================================================

docs: ## Build all documentation (auto-updates metrics, Rust API docs, mdbook)
	@echo "Building documentation..."
	@echo "  → Updating metrics in documentation..."
	@./scripts/docs/inject-metrics.sh 2>/dev/null || true
	@echo "  → Building Rust API docs..."
	@cargo doc --no-deps --workspace
	@echo "  → Syncing mdbook..."
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/ 2>/dev/null || true; fi
	@echo "✓ Documentation built"

# =============================================================================
# Workflow targets (used by .github/workflows/docs.yml)
# =============================================================================

docs-check: ## Validate documentation files exist
	@if [ ! -d "docs" ]; then echo "ERROR: docs/ directory not found"; exit 1; fi

docs-setup: ## Setup documentation (validates mdbook config)
	@mkdir -p book
	@if [ ! -f "book.toml" ]; then echo "ERROR: book.toml not found in root"; exit 1; fi

docs-sync: ## Sync documentation files from source
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true

docs-build: ## Build mdbook HTML
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/ 2>/dev/null || true; fi

rust-docs: ## Build Rust API documentation
	@cargo doc --no-deps --workspace

diagrams: ## Generate architecture diagrams with PlantUML
	@mkdir -p docs/architecture/diagrams/generated
	@if command -v plantuml >/dev/null 2>&1; then \
		for f in docs/architecture/diagrams/*.puml; do \
			if [ -f "$$f" ]; then \
				plantuml -o generated "$$f" 2>/dev/null || true; \
			fi; \
		done; \
	fi

docs-serve: ## Serve documentation with live reload
	@echo "Starting documentation server..."
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) serve book/ --open; else echo "mdbook not installed (cargo install mdbook)"; fi

# =============================================================================
# ADR - Architecture Decision Records
# =============================================================================

adr: ## List Architecture Decision Records
	@echo "Architecture Decision Records:"
	@ls -1 docs/adr/[0-9]*.md 2>/dev/null | while read f; do \
		num=$$(basename "$$f" .md | cut -d- -f1); \
		title=$$(head -1 "$$f" | sed 's/^# ADR [0-9]*: //'); \
		printf "  %s: %s\n" "$$num" "$$title"; \
	done

adr-new: ## Create new ADR
	@./scripts/docs/create-adr.sh 2>/dev/null || echo "create-adr.sh not found"

# =============================================================================
# MARKDOWN LINTING - Check and fix markdown files
# =============================================================================

docs-lint: ## Lint markdown files (FIX=1 to auto-fix)
ifeq ($(FIX),1)
	@echo "Auto-fixing markdown issues..."
	@./scripts/docs/markdown.sh fix
else
	@echo "Checking markdown files..."
	@./scripts/docs/markdown.sh lint
endif

# =============================================================================
# DOCUMENTATION VALIDATION - Comprehensive checks
# =============================================================================

docs-validate: ## Validate documentation (ADRs, structure, links). QUICK=1 skips external link checks.
	@echo "Validating documentation..."
	@QUICK="$(QUICK)" ./scripts/docs/validate.sh all

# =============================================================================
# DOCUMENTATION AUTO-UPDATE - For CI automation
# =============================================================================

docs-auto: docs-metrics docs-lint ## Auto-update docs (metrics + lint check) - used by CI
	@echo "✅ Documentation auto-updated"

docs-fix: docs-metrics ## Fix markdown (metrics + markdownlint -f). Run before commit.
	@$(MAKE) docs-lint FIX=1
	@echo "✅ Documentation fixed"

# =============================================================================
# Metrics - Auto-update documentation with current project metrics
# =============================================================================

docs-metrics: ## Update all documentation with current metrics (single source of truth)
	@./scripts/docs/inject-metrics.sh
