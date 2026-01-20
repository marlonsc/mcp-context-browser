//! Test utilities for mcb-infrastructure integration tests
//!
//! Provides factories and helpers for creating real (not mocked) test contexts
//! that exercise the full DI container and provider stack.

pub mod real_providers;

pub use real_providers::*;
