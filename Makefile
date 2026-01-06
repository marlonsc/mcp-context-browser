# MCP Context Browser - Development Makefile

.PHONY: help build test run clean check lint format doc release install deps update

# Default target
help: ## Show this help message
	@echo "MCP Context Browser - Development Commands"
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

# Build commands
build: ## Build the project in release mode
	cargo build --release

build-dev: ## Build the project in development mode
	cargo build

# Testing
test: ## Run all tests
	cargo test

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests
	cargo test --test integration

# Running
run: ## Run the application
	cargo run

run-release: ## Run the application in release mode
	cargo run --release

# Code quality
check: ## Check code without building
	cargo check

lint: ## Run clippy linter
	cargo clippy -- -D warnings

format: ## Format code with rustfmt
	cargo fmt

format-check: ## Check if code is formatted
	cargo fmt --check

# Documentation
doc: ## Generate documentation
	cargo doc --open --no-deps

# Dependencies
deps: ## Show dependency tree
	cargo tree

update: ## Update dependencies
	cargo update

# Cleaning
clean: ## Clean build artifacts
	cargo clean

clean-all: ## Clean everything including Cargo.lock
	cargo clean
	rm -f Cargo.lock

# Development workflow
dev: format lint test build ## Run full development workflow

# Release
release: clean format lint test build ## Prepare for release

# Installation
install: ## Install the binary
	cargo install --path .

# Docker (future)
docker-build: ## Build Docker image
	docker build -t mcp-context-browser .

docker-run: ## Run Docker container
	docker run --rm -it mcp-context-browser

# Git operations
git-status: ## Show git status
	git status

git-add: ## Add all files to git
	git add .

git-commit: ## Commit with message (usage: make git-commit MSG="your message")
	@echo "Committing with message: $(MSG)"
	git commit -m "$(MSG)"

git-push: ## Push to remote repository
	git push origin main

# Project info
info: ## Show project information
	@echo "MCP Context Browser v$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)"
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"

# Performance
bench: ## Run benchmarks
	cargo bench

flamegraph: ## Generate flamegraph (requires cargo-flamegraph)
	cargo flamegraph --bin mcp-context-browser

# Coverage (requires tarpaulin)
coverage: ## Generate test coverage report
	cargo tarpaulin --out Html

# CI/CD simulation
ci: clean format-check lint test build ## Simulate CI pipeline