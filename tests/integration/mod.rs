//! Integration tests

mod docker;
#[allow(clippy::module_inception)]
mod integration;
mod integration_docker;
// integration_unit module removed - contained only placeholder tests
mod mcp_e2e;
mod mcp_full_integration_test;
mod mcp_protocol;
mod ollama_integration;
