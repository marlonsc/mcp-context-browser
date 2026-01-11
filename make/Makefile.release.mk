# =============================================================================
# RELEASE - Build, packaging and release publishing
# =============================================================================

.PHONY: release package github-release version-bump version-tag version-push version-all

# Release
release: test build-release package ## Create release

# Packaging
package: ## Package release
	@mkdir -p dist
	@cp target/release/mcp-context-browser dist/
	@cp docs/user-guide/README.md dist/README.md
	@cp LICENSE dist/
	@cd dist && tar -czf mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz mcp-context-browser README.md LICENSE
	@echo "📦 Release created: dist/mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz"

# GitHub release
github-release: release ## Create GitHub release
	@echo "🚀 Creating GitHub release v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')..."
	@gh release create v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') \
		--title "MCP Context Browser v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')" \
		--notes "Release v$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/') - Auto-managed release" \
		dist/mcp-context-browser-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/').tar.gz
	@echo "✅ GitHub release created successfully!"

# =============================================================================
# VERSION - Auto-managed versioning for v0.0.3
# =============================================================================

version-bump: ## Bump version to 0.0.3 in Cargo.toml
	@echo "⬆️ Bumping version to 0.0.3..."
	@sed -i 's/^version = "0\.0\.2"/version = "0.0.3"/' Cargo.toml
	@echo "✅ Version bumped to 0.0.3"

version-tag: ## Create and push version tag
	@echo "🏷️ Creating tag v0.0.3..."
	@git tag v0.0.3
	@git push origin v0.0.3
	@echo "✅ Tag v0.0.3 created and pushed"

version-push: ## Commit and push version changes
	@echo "📤 Pushing version changes..."
	@git add -A
	@git commit --allow-empty -m "Force commit: $(shell date '+%Y-%m-%d %H:%M:%S') - Automated update" || echo "No changes to commit"
	@git push --force-with-lease origin main || git push --force origin main
	@echo "✅ Version changes pushed"

version-all: version-bump version-push version-tag ## Complete version management