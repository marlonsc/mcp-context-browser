# =============================================================================
# DEVELOPMENT - Development and configuration workflows
# =============================================================================

.PHONY: dev dev-metrics dev-sync setup ci dev-cycle dev-ready dev-deploy docker-up docker-down docker-logs test-integration-docker test-docker-full docker-status

# Development server
dev: ## Run development server
	cargo watch -x run

dev-metrics: ## Development with metrics
	@echo "🚀 Starting development server with metrics..."
	cargo watch -x "run -- --metrics"

dev-sync: ## Development with sync testing
	@echo "🔄 Starting development with sync features..."
	cargo watch -x "run -- --sync-test"

# Development setup
setup: ## Setup development tools (MANDATORY)
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	@echo "📦 Installing markdownlint-cli (required for markdown linting)..."
	@if ! command -v npm >/dev/null 2>&1; then \
		echo "❌ ERROR: npm required for markdownlint-cli installation"; \
		echo "Install Node.js and npm first: https://nodejs.org/"; \
		exit 1; \
	fi
	@if ! npm install -g markdownlint-cli; then \
		echo "❌ ERROR: Failed to install markdownlint-cli"; \
		echo "Check npm permissions or install manually: npm install -g markdownlint-cli"; \
		exit 1; \
	fi
	@if ! command -v markdownlint >/dev/null 2>&1; then \
		echo "❌ ERROR: markdownlint-cli not found after installation"; \
		exit 1; \
	fi
	@echo "✅ Development environment ready with full markdown linting"

# Continuous integration
ci: clean validate test build docs ## Run full CI pipeline

# Development cycles
dev-cycle: fix test-quiet ## Development iteration: fix + test

dev-ready: dev-cycle quality ## Development ready: iteration + quality

dev-deploy: dev-ready version-all github-release ## Development deploy: ready + version + release

# Docker integration testing
docker-up: ## Start Docker test services (OpenAI mock, Ollama, Milvus)
	@echo "🚀 Starting Docker test services..."
	@docker-compose up -d
	@echo "⏳ Waiting for services to be ready..."
	@sleep 30
	@echo "✅ Docker services are ready"

docker-down: ## Stop Docker test services
	@echo "🛑 Stopping Docker test services..."
	@docker-compose down -v

docker-logs: ## Show Docker test services logs
	@docker-compose logs -f

test-integration-docker: ## Run integration tests with Docker containers
	@echo "🧪 Running integration tests with Docker containers..."
	@OPENAI_BASE_URL=http://localhost:1080 \
	OLLAMA_BASE_URL=http://localhost:11434 \
	MILVUS_ADDRESS=http://localhost:19530 \
	cargo test --test integration_docker -- --nocapture

test-docker-full: docker-up test-integration-docker docker-down ## Run full Docker test cycle (up -> test -> down)

docker-status: ## Check status of Docker test services
	@echo "🔍 Checking Docker services status..."
	@docker-compose ps
	@echo ""
	@echo "🔗 Service endpoints:"
	@echo "  OpenAI Mock: http://localhost:1080"
	@echo "  Ollama: http://localhost:11434"
	@echo "  Milvus: http://localhost:19530"