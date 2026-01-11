# =============================================================================
# HELP & INFO - Command documentation and usage information
# =============================================================================

.PHONY: help all

# Default target - complete workflow
all: release ## Complete development workflow

# Help system
help: ## Show all available commands
	@echo "\033[1;36mMCP Context Browser v0.1.0 - Professional Makefile\033[0m"
	@echo "\033[1;33m==================================================\033[0m"
	@echo "\033[1;32mAvailable commands:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v '^help' | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[1;34m%-15s\033[0m %s\n", $$1, $$2}'