# =============================================================================
# DEVELOPMENT - Development workflows and Docker integration
# =============================================================================

.PHONY: dev dev-metrics dev-sync setup
.PHONY: docker-up docker-down docker-logs docker-status test-docker

# -----------------------------------------------------------------------------
# Development Server
# -----------------------------------------------------------------------------

dev: ## Run development server with auto-reload
	cargo watch -x run

dev-metrics: ## Development with metrics enabled
	@echo "🚀 Starting development server with metrics..."
	cargo watch -x "run -- --metrics"

dev-sync: ## Development with sync testing
	@echo "🔄 Starting development with sync features..."
	cargo watch -x "run -- --sync-test"

# -----------------------------------------------------------------------------
# Setup
# -----------------------------------------------------------------------------

setup: ci-setup-tools ## Install development tools
	@echo "📦 Installing Rust dev tools..."
	@cargo install cargo-watch --locked 2>/dev/null || true
	@echo "📝 Installing markdown tools..."
	@npm install -g markdownlint-cli 2>/dev/null || echo "⚠️  markdownlint-cli skipped (npm unavailable)"
	@echo "✅ Development environment ready"

# -----------------------------------------------------------------------------
# Docker Integration Testing
# -----------------------------------------------------------------------------

docker-up: ## Start test services (Ollama, Milvus, OpenAI mock)
	@echo "🚀 Starting Docker test services..."
	@docker-compose up -d
	@echo "⏳ Waiting for services..."
	@sleep 30
	@echo "✅ Services ready"

docker-down: ## Stop test services
	@echo "🛑 Stopping Docker test services..."
	@docker-compose down -v

docker-logs: ## Stream Docker logs
	@docker-compose logs -f

docker-status: ## Show service status and endpoints
	@echo "🔍 Docker Services:"
	@docker-compose ps
	@echo ""
	@echo "🔗 Endpoints:"
	@echo "  OpenAI Mock: http://localhost:1080"
	@echo "  Ollama:      http://localhost:11434"
	@echo "  Milvus:      http://localhost:19530"

test-integration-docker: ## Run integration tests with Docker containers (for use inside test-runner)
	@echo "🧪 Running integration tests with Docker services..."
	@OPENAI_BASE_URL=http://mcp-openai-mock:1080 \
	OLLAMA_BASE_URL=http://mcp-ollama:11434 \
	MILVUS_ADDRESS=http://mcp-milvus-standalone:19530 \
	REDIS_URL=redis://host.docker.internal:6379 \
	NATS_URL=nats://host.docker.internal:4222 \
	cargo test --test integration_docker redis_cache_integration nats_event_bus_integration -- --nocapture

test-docker: docker-up ## Run integration tests with Docker (uses host Redis/NATS and Docker services)
	@echo "🧪 Running Docker integration tests..."
	@echo "📍 Docker services: OpenAI mock, Ollama, Milvus"
	@echo "📍 Host services: Redis, NATS (from host machine)"
	@OPENAI_BASE_URL=http://localhost:1080 \
	OLLAMA_BASE_URL=http://localhost:11434 \
	MILVUS_ADDRESS=http://localhost:19530 \
	REDIS_URL=${REDIS_URL:-redis://127.0.0.1:6379} \
	NATS_URL=${NATS_URL:-nats://127.0.0.1:4222} \
	cargo test --test integration_docker redis_cache_integration nats_event_bus_integration -- --nocapture || true
	@$(MAKE) docker-down
