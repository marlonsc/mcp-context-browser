//! AST Analysis Module
//!
//! Provides unified AST parsing and querying across multiple programming languages
//! using Tree-sitter parsers.

pub mod core;
pub mod decoder;
pub mod engine;
pub mod executor;
pub mod languages;
pub mod query;
pub mod types;
pub mod unwrap_detector;

// Re-export public types and interfaces
pub use core::{AstNode, AstParseResult, Position, Span};
pub use decoder::AstDecoder;
pub use engine::{AstEngine, AstParser};
pub use executor::{AstQueryExecutor, AstQueryViolation};
pub use languages::*;
pub use query::{AstQuery, AstQueryBuilder, AstQueryPatterns, QueryCondition};
pub use types::AstViolation;
pub use unwrap_detector::{UnwrapDetection, UnwrapDetector};
