//! End-to-End Integration Tests
//!
//! Full system integration tests including MCP protocol,
//! Docker containers, and external service integration.

mod admin_e2e_test;
mod docker;
mod integration_docker;
mod integration_logic;
mod mcp_e2e;
mod mcp_full_integration_test;
mod mcp_protocol;
mod nats_event_bus_integration;
mod ollama_integration;
mod redis_cache_integration;
