# =============================================================================
# DOCS - Documentation management
# =============================================================================
# Essential targets only. Each verb does ONE action.
# =============================================================================

.PHONY: docs docs-serve docs-check docs-setup docs-sync docs-build diagrams rust-docs adr adr-new info

# Path to mdbook
MDBOOK := $(HOME)/.cargo/bin/mdbook

# =============================================================================
# DOCS - Build all documentation
# =============================================================================

docs: ## Build all documentation (Rust API + mdbook)
	@echo "Building documentation..."
	@cargo doc --no-deps --workspace
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/ 2>/dev/null || true; fi
	@echo "Documentation built"

# Workflow targets (used by .github/workflows/docs.yml)
docs-check: ## Validate documentation files exist
	@if [ ! -d "docs" ]; then echo "ERROR: docs/ directory not found"; exit 1; fi

docs-setup: ## Setup documentation (creates mdbook config if needed)
	@mkdir -p book
	@if [ ! -f "book/book.toml" ]; then echo "ERROR: book/book.toml not found"; exit 1; fi

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
	@./scripts/docs/create-adr.sh

# =============================================================================
# INFO - Project metrics
# =============================================================================

info: ## Display project metrics
	@./scripts/docs/extract-metrics.sh --markdown 2>/dev/null || echo "Metrics script not found"
