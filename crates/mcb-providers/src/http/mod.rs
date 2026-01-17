//! HTTP Client Abstractions
//!
//! Defines the HTTP client traits and utilities used by API-based providers.
//! The actual HTTP client implementation is provided by mcb-infrastructure.
//!
//! ## Design Rationale
//!
//! This module follows Clean Architecture by defining interfaces (traits) that
//! provider implementations depend on. The concrete HTTP client implementation
//! is injected via dependency injection from the infrastructure layer.
//!
//! ## Contents
//!
//! - `HttpClientProvider` - Trait for HTTP client operations
//! - `HttpClientConfig` - Configuration for HTTP client settings
//! - `HttpResponseUtils` - Utilities for handling HTTP responses (re-exported from utils)

pub mod provider;

pub use provider::{HttpClientConfig, HttpClientProvider};
// Re-export HttpResponseUtils from utils for backward compatibility
pub use crate::utils::HttpResponseUtils;
