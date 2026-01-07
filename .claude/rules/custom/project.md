# Project: MCP Context Browser

**Last Updated:** Wednesday Jan 7, 2026

## Overview

A Model Context Protocol server for semantic code analysis using vector embeddings. MCP Context Browser v0.0.3 is an advanced MCP server with enterprise-grade architecture, multi-provider routing, and comprehensive security features.

## Technology Stack

-   **Language:** Rust (2021 edition)
-   **Framework:** Tokio async runtime, Axum HTTP server
-   **Build Tool:** Cargo with modular Makefile system
-   **Testing:** Built-in Rust testing with Docker integration
-   **Package Manager:** Cargo
-   **Configuration:** TOML-based configuration with schema validation
-   **Deployment:** Docker + Kubernetes

## Directory Structure

```text
├── src/                    # Main Rust source code
│   ├── core/              # Core functionality (auth, cache, database, etc.)
│   ├── providers/         # Multi-provider system (embedding, vector_store, routing)
│   ├── chunking/          # AST-based code chunking engine
│   ├── services/          # Business logic services
│   ├── server/            # HTTP server and middleware
│   ├── metrics/           # Monitoring and metrics collection
│   └── sync/              # Synchronization and locking
├── docs/                  # Comprehensive documentation
│   ├── adr/               # Architectural Decision Records
│   ├── architecture/      # Technical architecture docs
│   └── modules/           # Module-specific documentation
├── tests/                 # Test suite
├── k8s/                   # Kubernetes deployment manifests
├── monitoring/            # Prometheus/Grafana monitoring
└── scripts/               # Development and maintenance scripts
```

## Key Files

-   **Configuration:** `src/config.rs`, `src/config_example.rs`
-   **Entry Points:** `src/main.rs`, `src/lib.rs`
-   **Core Engine:** `src/chunking/engine.rs`, `src/services/search.rs`
-   **Providers:** `src/providers/embedding/`, `src/providers/vector_store/`
-   **Tests:** `tests/` directory with comprehensive test coverage

## Development Commands

-   **Install:** `make setup` (sets up all dependencies)
-   **Dev:** `make dev` (development mode)
-   **Build:** `cargo build` or `make build`
-   **Build Release:** `cargo build --release` or `make build-release`
-   **Test:** `make test` (all tests) or `cargo test`
-   **Quality:** `make quality` (fmt + lint + test + audit + validate)
-   **Lint:** `make lint` (Rust clippy)
-   **Format:** `make fmt` (Rust fmt)

## Architecture Notes

**Advanced Dependency Injection:** Provider Registry with thread-safe management using downcast-rs and async-trait patterns.

**Multi-Provider Routing:** Intelligent routing system with health monitoring, circuit breakers (governor crate), and automatic failover between OpenAI, Ollama, and other embedding providers.

**Async-First Design:** Built on Tokio with streams, concurrent processing using Rayon, and advanced async utilities.

**Enterprise Security:** JWT authentication, encryption at rest (AES-GCM), rate limiting, and comprehensive security middleware.

**Hybrid Search:** Combines BM25 text search with vector similarity search using Milvus/EdgeVec databases.

## Additional Context

This project follows strict engineering practices with ADR-driven development, comprehensive testing (TDD), and automated quality gates. It includes Docker-based integration testing with real services (Ollama, Milvus) and supports production deployment with Kubernetes manifests and monitoring stacks.
