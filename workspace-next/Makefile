# Workspace-next Makefile
# For building the refactored crate structure

.PHONY: build test lint fmt clean check verify release
.PHONY: validate-arch validate-deps validate-quality validate-patterns validate-docs check-full ci

# Build all crates
build:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Full check (build + lint + test)
check: build lint test
	@echo "All checks passed!"

# Check compilation only (fast)
verify:
	cargo check --workspace

# Build in release mode
release:
	cargo build --workspace --release

# ============================================
# Architecture Validation Targets
# ============================================

# Full architecture validation
validate-arch:
	cargo test --package mcb-validate -- --nocapture
	@echo "Architecture validation passed!"

# Dependency validation only
validate-deps:
	cargo test --package mcb-validate dependency -- --nocapture

# Quality validation only (unwrap/expect/panic detection)
validate-quality:
	cargo test --package mcb-validate quality -- --nocapture

# Pattern validation only (DI patterns, async traits)
validate-patterns:
	cargo test --package mcb-validate patterns -- --nocapture

# Documentation validation only
validate-docs:
	cargo test --package mcb-validate documentation -- --nocapture

# Extended check (build + lint + test + arch validation)
check-full: check validate-arch
	@echo "Full validation passed!"

# CI target (all checks including architecture)
ci: fmt lint test validate-arch
	@echo "CI checks passed!"
