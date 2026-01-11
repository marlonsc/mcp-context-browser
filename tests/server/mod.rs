//! Tests for the server module
//!
//! Contains tests for:
//! - Transport layer (HTTP, sessions, versioning, config)
//! - Security middleware
//! - Rate limiting middleware

mod rate_limit_middleware;
mod security;
mod transport;
