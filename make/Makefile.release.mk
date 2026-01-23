# =============================================================================
# RELEASE - Build release, package, install, version
# =============================================================================
# Parameters: RELEASE, BUMP (from main Makefile)
# =============================================================================

.PHONY: release install version

# Get version from mcb crate Cargo.toml
VERSION := $(shell grep '^version' crates/mcb/Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

# Installation directory - single location for MCP and systemd
INSTALL_DIR := $(HOME)/.local/bin
INSTALL_BINARY := mcb
BINARY_NAME := mcb-server

# =============================================================================
# RELEASE - Full release pipeline
# =============================================================================

release: ## Full release pipeline (lint + test + validate + build)
	@echo "Creating release v$(VERSION)..."
	@$(MAKE) lint CI_MODE=1
	@$(MAKE) test
	@$(MAKE) validate QUICK=1
	@$(MAKE) build RELEASE=1
	@echo "Packaging..."
	@mkdir -p dist
	@cp target/release/$(BINARY_NAME) dist/ 2>/dev/null || echo "Binary not found"
	@cd dist && tar -czf $(BINARY_NAME)-$(VERSION).tar.gz $(BINARY_NAME) 2>/dev/null || true
	@echo "Release v$(VERSION) ready: dist/$(BINARY_NAME)-$(VERSION).tar.gz"

# =============================================================================
# INSTALL - Install binary to system
# =============================================================================

install: ## Install binary to ~/.local/bin/mcp-context-browser (RELEASE=1 for release)
ifeq ($(RELEASE),1)
	@echo "Installing release binary v$(VERSION)..."
	@$(MAKE) build RELEASE=1
else
	@echo "Installing debug binary v$(VERSION)..."
	@$(MAKE) build
endif
	@echo "Stopping running MCP processes..."
	@-pkill -f "mcp-context-browser" 2>/dev/null || true
	@sleep 1
	@mkdir -p $(INSTALL_DIR)
ifeq ($(RELEASE),1)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_BINARY)
else
	@cp target/debug/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_BINARY)
endif
	@chmod +x $(INSTALL_DIR)/$(INSTALL_BINARY)
	@echo ""
	@echo "✓ Installed v$(VERSION) to $(INSTALL_DIR)/$(INSTALL_BINARY)"
	@echo "✓ Old processes killed - new binary will be used on next MCP call"
	@ls -lh $(INSTALL_DIR)/$(INSTALL_BINARY) | awk '{print "  Size: "$$5"  Modified: "$$6" "$$7" "$$8}'
ifneq ($(RELEASE),1)
	@echo ""
	@echo "Tip: Use 'make install RELEASE=1' for a smaller optimized binary (~70MB vs ~500MB)"
endif

# =============================================================================
# INSTALL-VALIDATE - Install and validate the binary works
# =============================================================================

install-validate: install ## Install and run quick validation
	@echo ""
	@echo "Validating installation..."
	@$(INSTALL_DIR)/$(INSTALL_BINARY) --version 2>/dev/null && echo "✓ Binary runs successfully" || echo "⚠ Binary validation failed"

# =============================================================================
# VERSION (BUMP=patch|minor|major|check)
# =============================================================================

# Calculate next versions
NEXT_PATCH := $(shell echo $(VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}')
NEXT_MINOR := $(shell echo $(VERSION) | awk -F. '{print $$1"."($$2+1)".0"}')
NEXT_MAJOR := $(shell echo $(VERSION) | awk -F. '{print ($$1+1)".0.0"}')

version: ## Show version (BUMP=patch|minor|major to bump)
ifeq ($(BUMP),patch)
	@echo "Bumping to $(NEXT_PATCH)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_PATCH)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_PATCH)"
else ifeq ($(BUMP),minor)
	@echo "Bumping to $(NEXT_MINOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MINOR)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MINOR)"
else ifeq ($(BUMP),major)
	@echo "Bumping to $(NEXT_MAJOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MAJOR)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MAJOR)"
else
	@echo "Current version: $(VERSION)"
	@echo "Next patch:      $(NEXT_PATCH)"
	@echo "Next minor:      $(NEXT_MINOR)"
	@echo "Next major:      $(NEXT_MAJOR)"
endif
