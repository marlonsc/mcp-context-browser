# =============================================================================
# GIT - Forced commit operations and git management
# =============================================================================

.PHONY: git-status git-add-all git-commit-force git-push-force git-tag git-force-all sync force-commit

# Git status and basic operations
git-status: ## Show git repository status
	@echo "Git repository status:"
	@git status --short

git-add-all: ## Add all changes to git
	@echo "Adding all changes to git..."
	@git add -A
	@echo "All changes added"

# Force commit operations
git-commit-force: ## Force commit all changes
	@echo "Committing all changes with force..."
	@git commit --allow-empty -m "Force commit: $(shell date '+%Y-%m-%d %H:%M:%S') - Automated update" || echo "No changes to commit"

git-push-force: ## Force push to remote repository
	@echo "Pushing changes with force..."
	@git push --force-with-lease origin main || git push --force origin main
	@echo "Changes pushed successfully"

# Tagging operations
git-tag: ## Create and push git tag
	@echo "Creating and pushing tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)..."
	@git tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)
	@git push origin v$(shell grep '^version' Cargo.toml | cut -d'"' -f2)
	@echo "Tag v$(shell grep '^version' Cargo.toml | cut -d'"' -f2) created and pushed!"

# Combined operations
git-force-all: git-add-all git-commit-force git-push-force ## Add, commit and push all changes with force
	@echo "Force commit and push completed!"

# Alternative force commit method
force-commit: ## Run force commit script (alternative method)
	@echo "Running force commit script..."
	@bash scripts/force-commit.sh

# Sync alias
sync: git-force-all ## Sync all changes to remote