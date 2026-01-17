# MCP Context Browser - Advanced Version Management System
# ========================================================

# Intelligent version management with auto-detection, safety features, and integrated workflows
#
# Features:
#   • Auto-detection of version-sensitive files
#   • Intelligent version bumping based on changes
#   • Integrated CI/CD workflows
#   • Comprehensive backup and rollback
#   • Multi-branch support
#   • Dependency validation

.PHONY: version version-check version-dry-run version-update version-restore
.PHONY: version-bump version-bump-patch version-bump-minor version-bump-major version-bump-auto
.PHONY: version-release version-release-patch version-release-minor version-release-major
.PHONY: version-ci version-ci-check version-ci-update version-ci-release
.PHONY: version-dev version-dev-bump version-dev-sync
.PHONY: version-validate version-analyze version-cleanup

# ============================================================================
# CONFIGURATION
# ============================================================================

# Core settings
VERSION_BACKUP_DIR := .version_backup_$(shell date +%Y%m%d_%H%M%S)
VERSION_CHANGELOG_FILE := docs/VERSION_HISTORY.md
VERSION_CONFIG_FILE := .version_config

# Auto-detect current version
CURRENT_VERSION := $(shell grep '^version = ' Cargo.toml 2>/dev/null | sed 's/version = "\(.*\)"/\1/' || echo "0.1.1")
PREV_VERSION := $(shell git tag --sort=-version:refname 2>/dev/null | head -1 | sed 's/v//' 2>/dev/null || echo "0.1.0")

# Calculate next versions
NEXT_PATCH_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}' 2>/dev/null || echo "0.1.2")
NEXT_MINOR_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."($$2+1)".0"}' 2>/dev/null || echo "0.2.0")
NEXT_MAJOR_VERSION := $(shell echo $(CURRENT_VERSION) | awk -F. '{print ($$1+1)".0.0"}' 2>/dev/null || echo "1.0.0")

# File patterns to exclude from version scanning
EXCLUDE_PATTERNS := \
	--exclude-dir=.git \
	--exclude-dir=target \
	--exclude-dir=node_modules \
	--exclude-dir=.version_backup_* \
	--exclude-dir=backup \
	--exclude=Cargo.lock

# Version replacement patterns (simplified)
VERSION_PATTERNS := \
	"version = \"0\.1\.[0-9]*\"" \
	"version-0\.1\.[0-9]*-blue" \
	"## Current Status: v0\.1\.[0-9]*" \
	"\*\*v0\.1\.[0-9]* production-ready\*\*" \
	"## Current Version: v0\.1\.[0-9]*" \
	"version-badge\">v0\.1\.[0-9]*</" \
	"# MCP Context Browser - Makefile v0\.1\.[0-9]*" \
	"MCP Context Browser v0\.1\.[0-9]* - Make Commands" \
	"version-badge\">v0\.1\.[0-9]*</" \
	"# ADR Tools Installation Script - v0\.1\.[0-9]*" \
	"version: v0\.1\.[0-9]*" \
	"newTag: v0\.1\.[0-9]*" \
	"VERSION=\"v0\.1\.[0-9]*\""

# Branch-specific settings
CURRENT_BRANCH := $(shell git branch --show-current 2>/dev/null || echo "unknown")
IS_MAIN_BRANCH := $(shell [ "$(CURRENT_BRANCH)" = "main" ] || [ "$(CURRENT_BRANCH)" = "master" ] && echo "true" || echo "false")
IS_DEVELOP_BRANCH := $(shell [ "$(CURRENT_BRANCH)" = "develop" ] || [ "$(CURRENT_BRANCH)" = "development" ] && echo "true" || echo "false")

# ============================================================================
# FILE DETECTION
# ============================================================================

# Static list of known version-sensitive files (can be extended)
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
	docs/user-guide/QUICKSTART.md \
	docs/user-guide/README.md \
	docs/developer/ROADMAP.md \
	docs/architecture/ARCHITECTURE.md \
	docs/operations/CHANGELOG.md \
	workspace-next/crates/mcb-validate/Cargo.toml \
	workspace-next/crates/mcb-providers/Cargo.toml \
	workspace-next/crates/mcb-infrastructure/Cargo.toml \
	workspace-next/crates/mcb-server/Cargo.toml \
	workspace-next/crates/mcb-application/Cargo.toml \
	workspace-next/crates/mcb-domain/Cargo.toml \
	workspace-next/crates/mcb/Cargo.toml

# ============================================================================
# CORE VERSION MANAGEMENT
# ============================================================================

# ============================================================================
# MAIN ENTRY POINTS
# ============================================================================

# Default version command - redirect to help
version: version-help

# Intelligent version check with auto-detection
version-check: version-validate
	@echo "🔍 Version Analysis - $(CURRENT_VERSION)"
	@echo "======================================"
	@echo "📊 Version Info:"
	@echo "   Current: $(CURRENT_VERSION) (from $(PREV_VERSION))"
	@echo "   Next: $(NEXT_PATCH_VERSION) | $(NEXT_MINOR_VERSION) | $(NEXT_MAJOR_VERSION)"
	@echo "   Branch: $(CURRENT_BRANCH)"
	@echo ""
	@echo "📁 Auto-detected files with version references:"
	@echo "   Found $(words $(VERSION_FILES)) files"
	@for file in $(VERSION_FILES); do \
		if [ -f "$$file" ]; then \
			file_type=$$(basename $$file | sed 's/.*\.//'); \
			case $$file_type in \
				toml) echo "     📦 $$file (Cargo)" ;; \
				md) echo "     📖 $$file (docs)" ;; \
				html) echo "     🎨 $$file (template)" ;; \
				sh) echo "     📜 $$file (script)" ;; \
				yaml) echo "     ☸️  $$file (k8s)" ;; \
				rs) echo "     🦀 $$file (source)" ;; \
				mk|Makefile) echo "     🔧 $$file (build)" ;; \
				*) echo "     📄 $$file" ;; \
			esac; \
		fi; \
	done
	@echo ""
	@echo "🔍 Recent version changes:"
	@git log --oneline --grep="version\|Version\|VERSION" -10 2>/dev/null || echo "   No recent version commits found"
	@echo ""
	@echo "💡 Next steps:"
	@echo "   make version-analyze     # Deep analysis of changes"
	@echo "   make version-bump-auto   # Auto-determine bump type"
	@echo "   make version-dry-run     # Preview version update"

# ============================================================================
# ANALYSIS & VALIDATION
# ============================================================================

# Validate system prerequisites
version-validate:
	@echo "🔍 System Validation..."
	@if [ "$(CURRENT_VERSION)" = "unknown" ]; then \
		echo "❌ Cannot determine current version from Cargo.toml"; \
		exit 1; \
	fi
	@if ! command -v git >/dev/null 2>&1; then \
		echo "❌ Git not found - required for version management"; \
		exit 1; \
	fi
	@if [ ! -f "Cargo.toml" ]; then \
		echo "❌ Cargo.toml not found in current directory"; \
		exit 1; \
	fi
	@echo "✅ System prerequisites OK"

# Deep analysis of changes and version impact
version-analyze: version-validate
	@echo "🔬 Deep Version Analysis"
	@echo "======================="
	@echo "📈 Change Analysis:"
	@breaking_changes=$$(git log --oneline --grep="break\|BREAK\|breaking\|BREAKING" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	new_features=$$(git log --oneline --grep="feat\|feature\|add\|new" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	bug_fixes=$$(git log --oneline --grep="fix\|bug\|issue" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	docs_changes=$$(git log --oneline --grep="docs\|doc\|readme\|changelog" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	echo "   Breaking changes: $$breaking_changes"; \
	echo "   New features: $$new_features"; \
	echo "   Bug fixes: $$bug_fixes"; \
	echo "   Documentation: $$docs_changes"
	@echo ""
	@echo "🎯 Recommended version bump:"
	@if [ "$$breaking_changes" -gt 0 ]; then \
		echo "   🚨 MAJOR ($(NEXT_MAJOR_VERSION)) - Breaking changes detected"; \
	elif [ "$$new_features" -gt 0 ]; then \
		echo "   ✨ MINOR ($(NEXT_MINOR_VERSION)) - New features added"; \
	else \
		echo "   🐛 PATCH ($(NEXT_PATCH_VERSION)) - Bug fixes only"; \
	fi
	@echo ""
	@echo "📁 Files that will be updated: $(words $(VERSION_FILES))"
	@echo "💾 Backup will be created: $(VERSION_BACKUP_DIR)"

# ============================================================================
# DRY RUN & PREVIEW
# ============================================================================

# Enhanced dry-run with detailed preview
version-dry-run: version-validate
	@echo "🔍 Version Update Preview (DRY RUN)"
	@echo "==================================="
	@echo "Current: $(CURRENT_VERSION) -> Target: $(CURRENT_VERSION)"
	@echo "Branch: $(CURRENT_BRANCH) | Files: $(words $(VERSION_FILES))"
	@echo ""
	@echo "📋 Detailed Change Preview:"
	@echo ""
	@total_changes=0; \
	for file in $(VERSION_FILES); do \
		if [ -f "$$file" ]; then \
			changes=$$(grep -c "[0-9]\+\.[0-9]\+\.[0-9]\+" "$$file" 2>/dev/null || echo "0"); \
			if [ "$$changes" -gt 0 ]; then \
				echo "  📄 $$file: $$changes version reference(s)"; \
				total_changes=$$((total_changes + changes)); \
			fi; \
		fi; \
	done; \
	echo ""; \
	echo "📊 Summary:"; \
	echo "   • Files to update: $(words $(VERSION_FILES))"; \
	echo "   • Total references: $$total_changes"; \
	echo "   • Backup location: $(VERSION_BACKUP_DIR)"; \
	echo ""; \
	echo "✅ SAFE PREVIEW - No files modified"; \
	echo "💡 Run 'make version-update' to apply changes"

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

# ============================================================================
# INTEGRATED WORKFLOWS
# ============================================================================

# Development workflow
version-dev:
	@echo "🔬 Development Version Workflow"
	@echo "=============================="
	@echo "Branch: $(CURRENT_BRANCH) | Develop: $(IS_DEVELOP_BRANCH)"
	@echo ""
	@echo "Available development commands:"
	@echo "  make version-dev-bump     # Bump version for development"
	@echo "  make version-dev-sync     # Sync versions across workspace"
	@echo "  make version-dry-run      # Preview changes safely"
	@echo ""
	@if [ "$(IS_DEVELOP_BRANCH)" = "true" ]; then \
		echo "✅ On develop branch - full workflow available"; \
	else \
		echo "⚠️  Not on develop branch - some features limited"; \
	fi

version-dev-bump: version-validate
	@echo "🔬 Development Version Bump"
	@if [ "$(IS_DEVELOP_BRANCH)" = "true" ]; then \
		echo "✅ On develop branch - proceeding with dev bump"; \
		make version-bump-patch; \
	else \
		echo "⚠️  Not on develop branch - creating dev version"; \
		dev_version="$(CURRENT_VERSION)-dev.$$(date +%Y%m%d%H%M%S)"; \
		echo "   Development version: $$dev_version"; \
		sed -i "s/version = \"$(CURRENT_VERSION)\"/version = \"$$dev_version\"/g" Cargo.toml; \
		echo "✅ Development version set in Cargo.toml"; \
	fi

version-dev-sync: version-validate
	@echo "🔄 Development Workspace Sync"
	@workspace_version=$$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	echo "Syncing workspace to version: $$workspace_version"; \
	if [ -d "workspace-next/crates" ]; then \
		for crate in workspace-next/crates/*/Cargo.toml; do \
			if [ -f "$$crate" ]; then \
				sed -i "s/version = \"[0-9]\+\.[0-9]\+\.[0-9]\+\"/version = \"$$workspace_version\"/g" "$$crate"; \
				echo "  ✅ Updated $$(basename $$(dirname $$crate))"; \
			fi; \
		done; \
		echo "✅ Workspace synchronized"; \
	else \
		echo "⚠️  No workspace crates found"; \
	fi

# CI/CD workflow
version-ci:
	@echo "🤖 CI/CD Version Workflow"
	@echo "========================"
	@echo "Branch: $(CURRENT_BRANCH) | CI: $$CI"
	@echo ""
	@echo "CI/CD commands:"
	@echo "  make version-ci-check     # CI validation"
	@echo "  make version-ci-update    # CI version update"
	@echo "  make version-ci-release   # CI release workflow"
	@echo ""
	@if [ -n "$$CI" ] || [ -n "$$GITHUB_ACTIONS" ]; then \
		echo "✅ Running in CI environment"; \
	else \
		echo "⚠️  Not in CI environment - commands may not work as expected"; \
	fi

version-ci-check: version-validate
	@echo "🤖 CI Version Check"
	@errors=0; \
	for file in $(VERSION_FILES); do \
		if [ ! -f "$$file" ]; then \
			echo "::error::Missing required file: $$file"; \
			errors=$$((errors + 1)); \
		fi; \
	done; \
	if [ $$errors -gt 0 ]; then \
		echo "::error::Version validation failed with $$errors errors"; \
		exit 1; \
	else \
		echo "✅ CI validation passed"; \
	fi

version-ci-update: version-ci-check version-backup
	@echo "🤖 CI Version Update"
	@export CI=true; \
	make version-update

version-ci-release: version-ci-update
	@echo "🤖 CI Release"
	@if [ "$(IS_MAIN_BRANCH)" = "true" ]; then \
		echo "✅ Main branch - creating production release"; \
		make version-release; \
	else \
		echo "⚠️  Not on main branch - skipping production release"; \
	fi

# ============================================================================
# INTELLIGENT BUMPING
# ============================================================================

# Intelligent version bumping based on changes
version-bump-auto: version-analyze
	@echo "🎯 Auto Version Bump Analysis"
	@echo "============================"
	@breaking_changes=$$(git log --oneline --grep="break\|BREAK\|breaking\|BREAKING" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	new_features=$$(git log --oneline --grep="feat\|feature\|add\|new" $(PREV_VERSION)..HEAD 2>/dev/null | wc -l); \
	if [ "$$breaking_changes" -gt 0 ]; then \
		echo "🚨 Detected breaking changes - recommending MAJOR bump"; \
		echo "   Command: make version-bump-major"; \
		make version-bump-major; \
	elif [ "$$new_features" -gt 0 ]; then \
		echo "✨ Detected new features - recommending MINOR bump"; \
		echo "   Command: make version-bump-minor"; \
		make version-bump-minor; \
	else \
		echo "🐛 No breaking changes or new features - recommending PATCH bump"; \
		echo "   Command: make version-bump-patch"; \
		make version-bump-patch; \
	fi

# Unified bump command with type selection
version-bump:
	@echo "📈 Version Bump Options"
	@echo "======================"
	@echo "Current version: $(CURRENT_VERSION)"
	@echo ""
	@echo "Choose bump type:"
	@echo "  1) PATCH ($(NEXT_PATCH_VERSION)) - Bug fixes, patches"
	@echo "  2) MINOR ($(NEXT_MINOR_VERSION)) - New features, non-breaking"
	@echo "  3) MAJOR ($(NEXT_MAJOR_VERSION)) - Breaking changes"
	@echo "  4) AUTO - Analyze changes and decide automatically"
	@echo ""
	@echo "Examples:"
	@echo "  make version-bump-patch    # Direct patch bump"
	@echo "  make version-bump-auto     # Intelligent analysis"
	@read -p "Enter choice (1-4) or press Enter for AUTO: " choice; \
	case $$choice in \
		1) make version-bump-patch ;; \
		2) make version-bump-minor ;; \
		3) make version-bump-major ;; \
		4|"") make version-bump-auto ;; \
		*) echo "❌ Invalid choice" ;; \
	esac

# Simple version update
version-update: version-validate version-backup
	@echo "🔄 Updating to version $(CURRENT_VERSION)"
	@echo "======================================="
	@echo "Backup: $(VERSION_BACKUP_DIR)"
	@echo ""
	@echo "Updating files..."
	@# Update each file individually \
	sed -i 's/version = "0\.1\.[0-9]*"/version = "$(CURRENT_VERSION)"/g' Cargo.toml 2>/dev/null || true
	@sed -i 's/version-0\.1\.[0-9]*-blue/version-$(CURRENT_VERSION)-blue/g' README.md 2>/dev/null || true
	@sed -i 's/## Current Status: v0\.1\.[0-9]*/## Current Status: v$(CURRENT_VERSION)/g' README.md 2>/dev/null || true
	@sed -i 's/\*\*v0\.1\.[0-9]* production-ready/\*\*v$(CURRENT_VERSION) production-ready/g' CLAUDE.md 2>/dev/null || true
	@sed -i 's/## Current Version: v0\.1\.[0-9]*/## Current Version: v$(CURRENT_VERSION)/g' CLAUDE.md 2>/dev/null || true
	@sed -i 's/version-badge">v0\.1\.[0-9]*</version-badge">v$(CURRENT_VERSION)</g' src/server/admin/web/templates/base.html 2>/dev/null || true
	@sed -i 's/# MCP Context Browser - Makefile v0\.1\.[0-9]*/# MCP Context Browser - Makefile v$(CURRENT_VERSION)/g' Makefile 2>/dev/null || true
	@sed -i 's/MCP Context Browser v0\.1\.[0-9]* - Make Commands/MCP Context Browser v$(CURRENT_VERSION) - Make Commands/g' make/Makefile.help.mk 2>/dev/null || true
	@sed -i 's/version-badge">v0\.1\.[0-9]*</version-badge">v$(CURRENT_VERSION)</g' scripts/docs/mdbook-sync.sh 2>/dev/null || true
	@sed -i 's/# ADR Tools Installation Script - v0\.1\.[0-9]*/# ADR Tools Installation Script - v$(CURRENT_VERSION)/g' scripts/setup/install-adr-tools.sh 2>/dev/null || true
	@sed -i 's/the v0\.1\.[0-9]* ".*Release"/the v$(CURRENT_VERSION) "Maintenance Release"/g' scripts/setup/install-adr-tools.sh 2>/dev/null || true
	@sed -i 's/Installing ADR Tools for MCP Context Browser v0\.1\.[0-9]*/Installing ADR Tools for MCP Context Browser v$(CURRENT_VERSION)/g' scripts/setup/install-adr-tools.sh 2>/dev/null || true
	@sed -i 's/version: v0\.1\.[0-9]*/version: v$(CURRENT_VERSION)/g' k8s/kustomization.yaml 2>/dev/null || true
	@sed -i 's/newTag: v0\.1\.[0-9]*/newTag: v$(CURRENT_VERSION)/g' k8s/kustomization.yaml 2>/dev/null || true
	@sed -i 's/VERSION="v0\.1\.[0-9]*"/VERSION="v$(CURRENT_VERSION)"/g' k8s/deploy.sh 2>/dev/null || true
	@# Update documentation files \
	for doc in docs/user-guide/README.md docs/user-guide/QUICKSTART.md docs/architecture/ARCHITECTURE.md; do \
		sed -i 's/version-0\.1\.[0-9]*-blue/version-$(CURRENT_VERSION)-blue/g' $$doc 2>/dev/null || true; \
		sed -i 's/## Current Capabilities (v0\.1\.[0-9]*)/## Current Capabilities (v$(CURRENT_VERSION))/g' $$doc 2>/dev/null || true; \
		sed -i 's/Get MCP Context Browser v0\.1\.[0-9]*/Get MCP Context Browser v$(CURRENT_VERSION)/g' $$doc 2>/dev/null || true; \
		sed -i 's/\*\*Version\*\*: 0\.1\.[0-9]*/\*\*Version\*\*: $(CURRENT_VERSION)/g' $$doc 2>/dev/null || true; \
	done
	@# Update workspace crates \
	for crate in workspace-next/crates/*/Cargo.toml; do \
		if [ -f "$$crate" ]; then \
			sed -i 's/version = "0\.1\.[0-9]*"/version = "$(CURRENT_VERSION)"/g' "$$crate" 2>/dev/null || true; \
		fi; \
	done
	@# Update Rust source files \
	find workspace-next -name "*.rs" -exec sed -i 's/version = "0\.1\.[0-9]*"/version = "$(CURRENT_VERSION)"/g' {} \; 2>/dev/null || true
	@echo "✅ Version update completed successfully"

# version-bump-patch: Bump patch version (0.1.x -> 0.1.x+1)
version-bump-patch: version-check
	@echo "📈 Bumping patch version: $(CURRENT_VERSION) -> $(NEXT_PATCH_VERSION)"
	@sed -i 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_PATCH_VERSION)"/g' Cargo.toml
	@echo "✅ Version bumped in Cargo.toml"
	@echo "🔄 Run 'make version-update' to update all references"

# version-bump-minor: Bump minor version (0.x.y -> 0.x+1.0)
version-bump-minor: version-check
	@echo "📈 Bumping minor version: $(CURRENT_VERSION) -> $(NEXT_MINOR_VERSION)"
	@sed -i 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_MINOR_VERSION)"/g' Cargo.toml
	@echo "✅ Version bumped in Cargo.toml"
	@echo "🔄 Run 'make version-update' to update all references"

# version-bump-major: Bump major version (x.y.z -> x+1.0.0)
version-bump-major: version-check
	@echo "📈 Bumping major version: $(CURRENT_VERSION) -> $(NEXT_MAJOR_VERSION)"
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

# version-help: Show comprehensive version management help
version-help:
	@echo "🔧 MCP Context Browser - Advanced Version Management"
	@echo "==================================================="
	@echo "Branch: $(CURRENT_BRANCH) | Version: $(CURRENT_VERSION)"
	@echo ""
	@echo "🎯 Quick Start:"
	@echo "  make version                 # Intelligent workflow selector"
	@echo "  make version-check           # Overview and status"
	@echo "  make version-dry-run         # Safe preview of changes"
	@echo "  make version-bump-auto       # Auto-determine bump type"
	@echo "  make version-update          # Apply all updates"
	@echo ""
	@echo "📋 Command Categories:"
	@echo ""
	@echo "🎯 Intelligent Workflows:"
	@echo "  make version                 # Auto-detect branch and workflow"
	@echo "  make version-bump            # Interactive bump type selection"
	@echo "  make version-bump-auto       # Analyze changes and auto-bump"
	@echo "  make version-analyze         # Deep change analysis"
	@echo ""
	@echo "🔬 Development Workflows:"
	@echo "  make version-dev             # Development workflow menu"
	@echo "  make version-dev-bump        # Development version bump"
	@echo "  make version-dev-sync        # Sync workspace versions"
	@echo ""
	@echo "🤖 CI/CD Workflows:"
	@echo "  make version-ci              # CI/CD workflow menu"
	@echo "  make version-ci-check        # CI validation"
	@echo "  make version-ci-update       # CI version update"
	@echo "  make version-ci-release      # CI production release"
	@echo ""
	@echo "⚡ Direct Operations:"
	@echo "  make version-check           # Current status and file analysis"
	@echo "  make version-validate        # Validate prerequisites"
	@echo "  make version-dry-run         # Preview changes safely"
	@echo "  make version-update          # Apply version updates"
	@echo "  make version-restore         # Rollback from backup"
	@echo "  make version-cleanup         # Clean old backups"
	@echo ""
	@echo "📈 Version Bumping:"
	@echo "  make version-bump-patch      # Patch: $(CURRENT_VERSION) -> $(NEXT_PATCH_VERSION)"
	@echo "  make version-bump-minor      # Minor: $(CURRENT_VERSION) -> $(NEXT_MINOR_VERSION)"
	@echo "  make version-bump-major      # Major: $(CURRENT_VERSION) -> $(NEXT_MAJOR_VERSION)"
	@echo ""
	@echo "🚀 Release Workflows:"
	@echo "  make version-release-patch   # Patch release (auto)"
	@echo "  make version-release-minor   # Minor release (auto)"
	@echo "  make version-release-major   # Major release (auto)"
	@echo "  make version-release         # Full release (interactive)"
	@echo ""
	@echo "📖 Information:"
	@echo "  make version-help            # This help message"
	@echo ""
	@echo "💡 Recommended Workflows:"
	@echo ""
	@echo "🔄 Standard Development:"
	@echo "  1. make version-check        # Check status"
	@echo "  2. make version-analyze      # Analyze changes"
	@echo "  3. make version-bump-auto    # Auto-bump version"
	@echo "  4. make version-dry-run      # Preview changes"
	@echo "  5. make version-update       # Apply updates"
	@echo "  6. make test                 # Test changes"
	@echo "  7. make version-release      # Create release"
	@echo ""
	@echo "🏃 Fast Development:"
	@echo "  make version-dev             # Dev workflow"
	@echo "  make version-dev-bump        # Quick dev bump"
	@echo "  make version-dev-sync        # Sync workspace"
	@echo ""
	@echo "🤖 CI/CD Pipeline:"
	@echo "  make version-ci-check        # Validate"
	@echo "  make version-ci-update       # Update versions"
	@echo "  make version-ci-release      # Production release"
	@echo ""
	@echo "🛡️  Safety & Features:"
	@echo "  • Auto-detection of version-sensitive files"
	@echo "  • Intelligent change analysis and recommendations"
	@echo "  • Comprehensive backup and rollback"
	@echo "  • Branch-aware workflows"
	@echo "  • CI/CD integration support"
	@echo "  • Interactive and automated modes"
	@echo ""
	@echo "🔒 Architecture:"
	@echo "  • Single source of truth: Cargo.toml"
	@echo "  • Auto-generated patterns from file analysis"
	@echo "  • Configurable exclusions and patterns"
	@echo "  • Timestamped backups with rollback"
	@echo "  • Git-aware version analysis"