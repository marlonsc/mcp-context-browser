# MCP Context Browser - Version Management
# ========================================

# Version management system for MCP Context Browser
# This file contains targets to update version references across the codebase
#
# Usage:
#   make version-check                    # Check current version and files
#   make version-dry-run                  # Show what would be changed (dry-run)
#   make version-update                   # Apply version updates
#   make version-bump-patch               # Bump patch version (0.1.x -> 0.1.x+1)
#   make version-bump-minor               # Bump minor version (0.x.y -> 0.x+1.0)
#   make version-bump-major               # Bump major version (x.y.z -> x+1.0.0)
#   make version-release                  # Full release workflow

.PHONY: version-update version-check version-dry-run version-bump-patch version-bump-minor version-bump-major version-release

# Configuration
VERSION_BACKUP_DIR := .version_backup_$(shell date +%Y%m%d_%H%M%S)
DRY_RUN := false

# Extract current version from Cargo.toml
CURRENT_VERSION := $(shell grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
NEXT_PATCH_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}')
NEXT_MINOR_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."($$2+1)".0"}')
NEXT_MAJOR_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print ($$1+1)".0.0"}')

# Files that contain version references that need to be updated
VERSION_FILES := \
	Cargo.toml \
	README.md \
	CLAUDE.md \
	src/server/admin/web/templates/base.html \
	docs/VERSION_HISTORY.md \
	Makefile \
	make/Makefile.help.mk \
	scripts/docs/mdbook-sync.sh \
	scripts/setup/install-adr-tools.sh \
	k8s/kustomization.yaml \
	k8s/deploy.sh \
	workspace-next/crates/mcb-validate/Cargo.toml \
	docs/user-guide/QUICKSTART.md \
	docs/user-guide/README.md \
	docs/developer/ROADMAP.md \
	docs/architecture/ARCHITECTURE.md \
	docs/operations/CHANGELOG.md

# Validation patterns for different file types
VALIDATION_PATTERNS := \
	'*.toml:version = "0\.1\.[0-9]*"' \
	'*.md:version-0\.1\.[0-9]*-blue' \
	'*.md:## Current Status: v0\.1\.[0-9]*' \
	'*.md:**v0\.1\.[0-9]* production-ready' \
	'*.md:## Current Version: v0\.1\.[0-9]*' \
	'*.html:version-badge">v0\.1\.[0-9]*</' \
	'Makefile:# MCP Context Browser - Makefile v0\.1\.[0-9]*' \
	'*.mk:MCP Context Browser v0\.1\.[0-9]* - Make Commands' \
	'*.sh:version-badge">v0\.1\.[0-9]*</' \
	'*.sh:# ADR Tools Installation Script - v0\.1\.[0-9]*' \
	'*.yaml:version: v0\.1\.[0-9]*' \
	'*.yaml:newTag: v0\.1\.[0-9]*' \
	'*.sh:VERSION="v0\.1\.[0-9]*"' \
	'*.rs:version = "0\.1\.[0-9]*"'

# version-check: Check current version and show files that need updates
version-check:
	@echo "🔍 MCP Context Browser Version Check"
	@echo "====================================="
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Next patch version: $(NEXT_PATCH_VERSION)"
	@echo "Next minor version: $(NEXT_MINOR_VERSION)"
	@echo "Next major version: $(NEXT_MAJOR_VERSION)"
	@echo ""
	@echo "Files containing version references:"
	@for file in $(VERSION_FILES); do \
		if [ -f "$$file" ]; then \
			echo "  ✅ $$file"; \
		else \
			echo "  ❌ $$file (not found)"; \
		fi \
	done
	@echo ""
	@echo "Version references found:"
	@grep -r "0\.1\." --include="*.md" --include="*.rs" --include="*.sh" --include="*.yaml" --include="*.toml" --include="Makefile*" --include="*.mk" --exclude-dir=target --exclude-dir=.git --exclude-dir=node_modules . | grep -v Cargo.lock | head -10
	@echo ""
	@echo "💡 Use 'make version-dry-run' to see what would be changed"
	@echo "💡 Use 'make version-update' to apply the changes"

# version-validate: Validate that all required files exist and are accessible
version-validate:
	@echo "🔍 Validating version management prerequisites..."
	@missing_files=0; \
	for file in $(VERSION_FILES); do \
		if [ ! -f "$$file" ]; then \
			echo "  ❌ Missing file: $$file"; \
			missing_files=$$((missing_files + 1)); \
		fi; \
	done; \
	if [ $$missing_files -gt 0 ]; then \
		echo "❌ Validation failed: $$missing_files files missing"; \
		exit 1; \
	else \
		echo "✅ All required files present"; \
	fi

# version-dry-run: Show what would be changed without making actual changes
version-dry-run: version-validate
	@echo "🔍 MCP Context Browser Version Update (DRY RUN)"
	@echo "================================================"
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Target version: $(CURRENT_VERSION)"
	@echo ""
	@echo "📋 Files that would be modified:"
	@echo ""
	@# Check Cargo.toml workspace versions
	@echo "🔧 Cargo.toml workspace versions:"
	@if [ -d "workspace-next/crates" ]; then \
		for crate in workspace-next/crates/*/Cargo.toml; do \
			if [ -f "$$crate" ] && grep -q 'version = "0\.1\.[0-9]*"' "$$crate"; then \
				echo "  • $$crate"; \
			fi; \
		done; \
	fi
	@echo ""
	@echo "📄 Documentation and configuration files:"
	@for file in $(VERSION_FILES); do \
		if [ -f "$$file" ]; then \
			case "$$file" in \
				*.md) \
					if grep -q "0\.1\.[0-9]" "$$file" 2>/dev/null; then \
						echo "  • $$file (documentation)"; \
					fi ;; \
				*.html) \
					if grep -q "v0\.1\.[0-9]" "$$file" 2>/dev/null; then \
						echo "  • $$file (template)"; \
					fi ;; \
				*.sh) \
					if grep -q "0\.1\.[0-9]" "$$file" 2>/dev/null; then \
						echo "  • $$file (script)"; \
					fi ;; \
				*.yaml) \
					if grep -q "v0\.1\.[0-9]" "$$file" 2>/dev/null; then \
						echo "  • $$file (kubernetes)"; \
					fi ;; \
				Makefile*) \
					if grep -q "v0\.1\.[0-9]" "$$file" 2>/dev/null; then \
						echo "  • $$file (makefile)"; \
					fi ;; \
			esac; \
		fi; \
	done
	@echo ""
	@echo "🔍 Source code files with version references:"
	@find workspace-next -name "*.rs" -exec grep -l "0\.1\.[0-9]" {} \; 2>/dev/null | head -5 | sed 's/^/  • /'
	@echo ""
	@echo "✅ This was a DRY RUN - no files were modified"
	@echo "💡 Run 'make version-update' to apply these changes"

# version-backup: Create backup of files before modification
version-backup:
	@echo "💾 Creating backup of version-sensitive files..."
	@mkdir -p $(VERSION_BACKUP_DIR)
	@for file in $(VERSION_FILES); do \
		if [ -f "$$file" ]; then \
			cp "$$file" "$(VERSION_BACKUP_DIR)/$$(basename $$file).backup" 2>/dev/null || true; \
		fi; \
	done
	@if [ -d "workspace-next/crates" ]; then \
		mkdir -p $(VERSION_BACKUP_DIR)/workspace-next/crates; \
		for crate in workspace-next/crates/*/Cargo.toml; do \
			if [ -f "$$crate" ]; then \
				crate_name=$$(basename $$(dirname $$crate)); \
				mkdir -p "$(VERSION_BACKUP_DIR)/workspace-next/crates/$$crate_name"; \
				cp "$$crate" "$(VERSION_BACKUP_DIR)/workspace-next/crates/$$crate_name/Cargo.toml.backup" 2>/dev/null || true; \
			fi; \
		done; \
	fi
	@echo "✅ Backup created in $(VERSION_BACKUP_DIR)"

# version-restore: Restore files from backup in case of error
version-restore:
	@if [ -d "$(VERSION_BACKUP_DIR)" ]; then \
		echo "🔄 Restoring files from backup..."; \
		for backup in $(VERSION_BACKUP_DIR)/*.backup; do \
			if [ -f "$$backup" ]; then \
				original_name=$$(basename $$backup .backup); \
				if [ -f "$$original_name" ]; then \
					cp "$$backup" "$$original_name"; \
					echo "  ✅ Restored $$original_name"; \
				fi; \
			fi; \
		done; \
		if [ -d "$(VERSION_BACKUP_DIR)/workspace-next" ]; then \
			for backup in $(VERSION_BACKUP_DIR)/workspace-next/crates/*/*.backup; do \
				if [ -f "$$backup" ]; then \
					crate_name=$$(basename $$(dirname $$backup)); \
					original_file="workspace-next/crates/$$crate_name/Cargo.toml"; \
					if [ -f "$$original_file" ]; then \
						cp "$$backup" "$$original_file"; \
						echo "  ✅ Restored $$original_file"; \
					fi; \
				fi; \
			done; \
		fi; \
		echo "✅ Files restored from backup"; \
	else \
		echo "❌ No backup directory found"; \
	fi

# version-update: Update all version references to current Cargo.toml version
version-update: version-validate version-backup
	@echo "🔄 Updating version references to $(CURRENT_VERSION)"
	@echo "=================================================="
	@echo "💾 Backup created in $(VERSION_BACKUP_DIR)"
	@echo ""
	@# Update with error handling
	@errors=0; \
	update_file() { \
		file="$$1"; \
		if [ -f "$$file" ]; then \
			cp "$$file" "$$file.bak" 2>/dev/null || true; \
			if ! sed -i.bak -e "$$2" "$$file" 2>/dev/null; then \
				echo "  ❌ Failed to update $$file"; \
				cp "$$file.bak" "$$file" 2>/dev/null || true; \
				return 1; \
			else \
				echo "  ✅ Updated $$file"; \
				rm -f "$$file.bak" 2>/dev/null || true; \
			fi; \
		fi; \
		return 0; \
	}; \
	\
	# Update Cargo.toml workspace versions \
	echo "🔧 Updating Cargo.toml workspace versions..."; \
	if [ -d "workspace-next/crates" ]; then \
		for crate in workspace-next/crates/*/Cargo.toml; do \
			if [ -f "$$crate" ]; then \
				update_file "$$crate" 's/version = "0\.1\.[0-9]*"/version = "$(CURRENT_VERSION)"/g' || errors=$$((errors + 1)); \
			fi; \
		done; \
	fi; \
	\
	# Update Rust source files \
	echo "🔧 Updating Rust source files..."; \
	find workspace-next -name "*.rs" -exec sed -i 's/version = "0\.1\.[0-9]*"/version = "$(CURRENT_VERSION)"/g' {} \; 2>/dev/null || true; \
	\
	# Update documentation files \
	echo "📄 Updating documentation files..."; \
	update_file "README.md" 's/version-0\.1\.[0-9]*-blue/version-$(CURRENT_VERSION)-blue/g' || errors=$$((errors + 1)); \
	update_file "README.md" 's/## Current Status: v0\.1\.[0-9]* ✅ RELEASED/## Current Status: v$(CURRENT_VERSION) ✅ RELEASED/g' || errors=$$((errors + 1)); \
	update_file "CLAUDE.md" 's/\*\*v0\.1\.[0-9]* production-ready/\*\*v$(CURRENT_VERSION) production-ready/g' || errors=$$((errors + 1)); \
	update_file "CLAUDE.md" 's/## Current Version: v0\.1\.[0-9]*/## Current Version: v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "docs/user-guide/README.md" 's/version-0\.1\.[0-9]*-blue/version-$(CURRENT_VERSION)-blue/g' || errors=$$((errors + 1)); \
	update_file "docs/user-guide/README.md" 's/## 🎯 Current Capabilities (v0\.1\.[0-9]*)/## 🎯 Current Capabilities (v$(CURRENT_VERSION))/g' || errors=$$((errors + 1)); \
	update_file "docs/user-guide/QUICKSTART.md" 's/Get MCP Context Browser v0\.1\.[0-9]*/Get MCP Context Browser v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "docs/architecture/ARCHITECTURE.md" 's/version-0\.1\.[0-9]*-blue/version-$(CURRENT_VERSION)-blue/g' || errors=$$((errors + 1)); \
	update_file "docs/architecture/ARCHITECTURE.md" 's/\*\*Version\*\*: 0\.1\.[0-9]*/\*\*Version\*\*: $(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	\
	# Update templates \
	echo "🎨 Updating templates..."; \
	update_file "src/server/admin/web/templates/base.html" 's/version-badge">v0\.1\.[0-9]*</version-badge">v$(CURRENT_VERSION)</g' || errors=$$((errors + 1)); \
	\
	# Update Makefiles \
	echo "🔧 Updating Makefiles..."; \
	update_file "Makefile" 's/# MCP Context Browser - Makefile v0\.1\.[0-9]*/# MCP Context Browser - Makefile v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "make/Makefile.help.mk" 's/MCP Context Browser v0\.1\.[0-9]* - Make Commands/MCP Context Browser v$(CURRENT_VERSION) - Make Commands/g' || errors=$$((errors + 1)); \
	\
	# Update scripts \
	echo "📜 Updating scripts..."; \
	update_file "scripts/docs/mdbook-sync.sh" 's/version-badge">v0\.1\.[0-9]*</version-badge">v$(CURRENT_VERSION)</g' || errors=$$((errors + 1)); \
	update_file "scripts/setup/install-adr-tools.sh" 's/# ADR Tools Installation Script - v0\.1\.[0-9]*/# ADR Tools Installation Script - v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "scripts/setup/install-adr-tools.sh" 's/the v0\.1\.[0-9]* ".*Release"/the v$(CURRENT_VERSION) "Maintenance Release"/g' || errors=$$((errors + 1)); \
	update_file "scripts/setup/install-adr-tools.sh" 's/Installing ADR Tools for MCP Context Browser v0\.1\.[0-9]*/Installing ADR Tools for MCP Context Browser v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	\
	# Update Kubernetes files \
	echo "☸️ Updating Kubernetes manifests..."; \
	update_file "k8s/kustomization.yaml" 's/version: v0\.1\.[0-9]*/version: v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "k8s/kustomization.yaml" 's/newTag: v0\.1\.[0-9]*/newTag: v$(CURRENT_VERSION)/g' || errors=$$((errors + 1)); \
	update_file "k8s/deploy.sh" 's/VERSION="v0\.1\.[0-9]*"/VERSION="v$(CURRENT_VERSION)"/g' || errors=$$((errors + 1)); \
	\
	if [ $$errors -eq 0 ]; then \
		echo ""; \
		echo "✅ Version update completed successfully!"; \
		echo ""; \
		echo "📋 Summary of changes:"; \
		echo "   • Cargo.toml workspace versions updated"; \
		echo "   • README.md badge and status updated"; \
		echo "   • CLAUDE.md references updated"; \
		echo "   • HTML templates updated"; \
		echo "   • Makefiles updated"; \
		echo "   • Scripts updated"; \
		echo "   • Kubernetes manifests updated"; \
		echo "   • Documentation files updated"; \
		echo "   • Rust source files updated"; \
		echo ""; \
		echo "🔍 Run 'make version-check' to verify all changes"; \
		echo "🗑️  Backup available in $(VERSION_BACKUP_DIR)"; \
	else \
		echo ""; \
		echo "❌ Version update completed with $$errors errors"; \
		echo "🔄 Run 'make version-restore' to restore from backup"; \
		exit 1; \
	fi

# version-bump-patch: Bump patch version (0.1.x -> 0.1.x+1)
version-bump-patch: version-check
	@echo "📈 Bumping patch version: $(CURRENT_VERSION) → $(NEXT_PATCH_VERSION)"
	@sed -i 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_PATCH_VERSION)"/g' Cargo.toml
	@echo "✅ Version bumped in Cargo.toml"
	@echo "🔄 Run 'make version-update' to update all references"

# version-bump-minor: Bump minor version (0.x.y -> 0.x+1.0)
version-bump-minor: version-check
	@echo "📈 Bumping minor version: $(CURRENT_VERSION) → $(NEXT_MINOR_VERSION)"
	@sed -i 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_MINOR_VERSION)"/g' Cargo.toml
	@echo "✅ Version bumped in Cargo.toml"
	@echo "🔄 Run 'make version-update' to update all references"

# version-bump-major: Bump major version (x.y.z -> x+1.0.0)
version-bump-major: version-check
	@echo "📈 Bumping major version: $(CURRENT_VERSION) → $(NEXT_MAJOR_VERSION)"
	@sed -i 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_MAJOR_VERSION)"/g' Cargo.toml
	@echo "✅ Version bumped in Cargo.toml"
	@echo "🔄 Run 'make version-update' to update all references"

# version-release: Full release workflow (bump version, update all files, commit)
version-release: version-bump-patch version-update
	@echo "🚀 Creating release commit for v$(CURRENT_VERSION)"
	@git add -A
	@git commit -m "Release v$(CURRENT_VERSION): Version management unification" -m "" -m "- Unified version management across codebase" -m "- Automated version update system in Makefile" -m "- Updated all version references to v$(CURRENT_VERSION)" -m "- Added VERSION constant in lib.rs using env!()" -m "" -m "This release introduces unified version management with a single source" -m "of truth in Cargo.toml and automated update capabilities."
	@git tag "v$(CURRENT_VERSION)"
	@echo "✅ Release v$(CURRENT_VERSION) created and tagged!"
	@echo ""
	@echo "📋 Next steps:"
	@echo "   1. Push to repository: git push && git push --tags"
	@echo "   2. Create GitHub release with changelog"
	@echo "   3. Update release notes in VERSION_HISTORY.md if needed"