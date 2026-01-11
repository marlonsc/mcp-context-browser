# =============================================================================
# DOCUMENTATION - Automation using existing tools
# =============================================================================

.PHONY: docs docs-generate docs-validate docs-quality docs-check-adr docs-setup rust-docs adr-new adr-list adr-generate adr-status

# Main documentation targets
docs: docs-generate docs-validate ## Generate and validate all documentation
	@echo "🤖 Documentation automation completed"

# Generate automated documentation using existing tools
docs-generate: ## Generate automated documentation from source code
	@echo "📊 Generating automated documentation..."
	@./scripts/docs/automation.sh generate

# Validate documentation and ADR compliance
docs-validate: ## Validate documentation quality and ADR compliance
	@echo "🔍 Running documentation validation..."
	@./scripts/docs/automation.sh validate

# Quality checks using existing tools
docs-quality: ## Run quality checks on documentation
	@echo "✨ Running documentation quality checks..."
	@./scripts/docs/automation.sh quality

# ADR compliance checking
docs-check-adr: ## Check ADR compliance and validation
	@echo "📋 Checking ADR compliance..."
	@./scripts/docs/automation.sh adr-check

# Setup documentation tools
docs-setup: ## Install and configure all documentation tools
	@echo "🔧 Setting up documentation tools..."
	@./scripts/docs/automation.sh setup

# Interactive documentation with mdbook
docs-book: ## Build interactive documentation with mdbook
	@echo "📖 Building interactive documentation..."
	@./scripts/docs/generate-mdbook.sh build

docs-serve: ## Serve interactive documentation with live reload
	@echo "🌐 Serving interactive documentation..."
	@./scripts/docs/generate-mdbook.sh serve

# Legacy aliases for backward compatibility
docs-auto: docs-generate ## Legacy alias for docs-generate

# Documentation synchronization
sync-docs: docs-validate ## Check documentation synchronization
sync-docs-update: docs-generate ## Update auto-generated docs

# Rust documentation
rust-docs: ## Generate Rust API documentation
	@echo "🦀 Generating Rust docs..."
	@cargo doc --no-deps --document-private-items

# ADR management using adrs tool
ADRS_CMD ?= $(HOME)/.cargo/bin/adrs

adr-new: ## Create new ADR using adrs tool
	@echo "📝 Creating new ADR..."
	@$(ADRS_CMD) new

adr-list: ## List ADRs using adrs tool
	@echo "📋 ADRs:"
	@$(ADRS_CMD) list

adr-generate: ## Generate ADR summary documentation
	@echo "📊 Generating ADR summary..."
	@$(ADRS_CMD) generate toc > docs/adr/README.md
	@$(ADRS_CMD) generate graph > docs/adr/adr-graph.md || true

adr-status: ## Show ADR status and lifecycle
	@echo "📈 ADR Status:"
	@$(ADRS_CMD) list --status || echo "Status tracking not available in this version of adrs"

# Legacy diagram generation (if scripts exist)
diagrams: ## Generate diagrams (if available)
	@if [ -f scripts/docs/generate-diagrams.sh ]; then \
		bash scripts/docs/generate-diagrams.sh all; \
	else \
		echo "⚠️  Diagram generation script not found"; \
	fi