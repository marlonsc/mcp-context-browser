//! Declarative Rule Registry
//!
//! Provides declarative rule definitions for Clean Architecture validation.
//! Rules are defined as data structures rather than hardcoded logic.

pub mod registry;

pub use registry::{clean_architecture_rules, layer_boundary_rules, Rule, RuleRegistry};
