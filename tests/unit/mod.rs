//! Unit tests organized by Clean Architecture layer
//!
//! Structure mirrors src/ directory:
//! - domain/ - Domain layer tests
//! - application/ - Application layer tests
//! - adapters/ - Adapter layer tests
//! - infrastructure/ - Infrastructure layer tests
//! - server/ - Server layer tests

mod adapters;
mod application;
mod domain;
mod infrastructure;
mod server;

// Legacy unit tests at root level
mod property_based;
mod rate_limiting;
mod reproduce_freeze;
mod security;
mod unit_test;
