//! End-to-End Integration Tests Aggregator
//!
//! Full system integration tests including MCP protocol,
//! Docker containers, and external services.
//!
//! Run with: cargo test --test e2e

#[path = "e2e/mod.rs"]
mod e2e;
