# MCP Context Browser - Build and Development Makefile

.PHONY: all build test clean docs diagrams help git-status git-add-all git-commit-force git-push-force git-force-all force-commit

# Default target
all: build test

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Testing
test:
	cargo test

test-integration:
	cargo test --test integration

test-unit:
	cargo test --lib

# Linting and formatting
fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

lint: fmt clippy

# Documentation generation
docs: diagrams
	@echo "Generating comprehensive documentation..."
	@cargo doc --no-deps --document-private-items
	@echo "Documentation generated in target/doc/"

diagrams:
	@echo "Generating architecture diagrams..."
	@bash scripts/docs/generate-diagrams.sh all

validate-diagrams:
	@echo "Validating PlantUML diagrams..."
	@bash scripts/docs/generate-diagrams.sh validate

# ADR management
adr-list:
	@echo "Architecture Decision Records:"
	@ls -1 docs/adr/ | grep -E '\.md$$' | sed 's/\.md$$//' | sort

adr-new:
	@echo "Creating new ADR..."
	@read -p "ADR Title: " title; \
	slug=$$(echo "$$title" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-\|-$//g'); \
	count=$$(ls docs/adr/ | grep -E '^[0-9]+\.md$$' | wc -l); \
	num=$$(printf "%03d" $$((count + 1))); \
	file="docs/adr/$${num}-$${slug}.md"; \
	cp docs/adr/TEMPLATE.md "$$file"; \
	echo "Created ADR: $$file"; \
	echo "Edit the file to add your ADR content"

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	@rustup component add rustfmt clippy
	@rustup component add llvm-tools-preview
	@cargo install cargo-watch
	@cargo install cargo-tarpaulin
	@echo "Development environment ready!"

# Run development server
dev:
	cargo watch -x run

# Performance profiling
profile:
	cargo build --release --features profiling
	@echo "Binary built with profiling: target/release/mcp-context-browser"
	@echo "Run with perf: perf record -F 99 -g ./target/release/mcp-context-browser"
	@echo "Analyze with: perf report -g"

# Benchmarking
bench:
	cargo bench

# Security scanning
audit:
	cargo audit

# Coverage reporting
coverage:
	cargo tarpaulin --out Html --output-dir target/coverage

# Docker targets
docker-build:
	docker build -t mcp-context-browser .

docker-run:
	docker run -p 3000:3000 mcp-context-browser

# Cleaning
clean:
	cargo clean
	rm -rf target/
	rm -rf docs/diagrams/generated/

clean-docs:
	rm -rf docs/diagrams/generated/
	rm -rf target/doc/

# CI/CD simulation
ci: lint test build docs
	@echo "CI pipeline completed successfully!"

# Release preparation
release-prep: clean lint test build-release docs
	@echo "Release preparation completed!"

# Git operations
git-status:
	@echo "Git repository status:"
	@git status --short

git-add-all:
	@echo "Adding all changes to git..."
	@git add -A
	@echo "All changes added"

git-commit-force:
	@echo "Committing all changes with force..."
	@git commit --allow-empty -m "Force commit: $(shell date '+%Y-%m-%d %H:%M:%S') - Automated update" || echo "No changes to commit"

git-push-force:
	@echo "Pushing changes with force..."
	@git push --force-with-lease origin main || git push --force origin main
	@echo "Changes pushed successfully"

git-force-all: git-add-all git-commit-force git-push-force
	@echo "Force commit and push completed!"

# Alternative script-based force commit
force-commit:
	@echo "Running force commit script..."
	@bash scripts/force-commit.sh

# Help target
help:
	@echo "MCP Context Browser - Development Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all              - Build and test (default)"
	@echo "  build            - Build debug version"
	@echo "  build-release    - Build release version"
	@echo "  test             - Run all tests"
	@echo "  test-unit        - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  lint             - Format and lint code"
	@echo "  fmt              - Format code with rustfmt"
	@echo "  clippy           - Run clippy linter"
	@echo "  docs             - Generate all documentation"
	@echo "  diagrams         - Generate architecture diagrams"
	@echo "  validate-diagrams - Validate PlantUML syntax"
	@echo "  adr-list         - List all ADRs"
	@echo "  adr-new          - Create new ADR interactively"
	@echo "  dev-setup        - Setup development environment"
	@echo "  dev              - Run with auto-reload"
	@echo "  profile          - Build for performance profiling"
	@echo "  bench            - Run benchmarks"
	@echo "  audit            - Run security audit"
	@echo "  coverage         - Generate test coverage report"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-run       - Run Docker container"
	@echo "  clean            - Clean build artifacts"
	@echo "  clean-docs       - Clean documentation artifacts"
	@echo "  ci               - Simulate CI pipeline"
	@echo "  release-prep     - Prepare for release"
	@echo "  git-status       - Show git repository status"
	@echo "  git-add-all      - Add all changes to git"
	@echo "  git-commit-force - Force commit all changes"
	@echo "  git-push-force   - Force push to remote repository"
	@echo "  git-force-all    - Add, commit and push all changes with force"
	@echo "  force-commit     - Run force commit script (alternative method)"
	@echo "  help             - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make build && make test          # Build and test"
	@echo "  make diagrams                    # Generate diagrams"
	@echo "  make docs                        # Generate full docs"
	@echo "  make adr-new                     # Create new ADR"
	@echo "  make git-status                  # Check git status"
	@echo "  make git-force-all               # Force commit and push all changes"

# Default target reminder
.DEFAULT_GOAL := help