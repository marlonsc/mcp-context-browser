//! AST Analysis Module
//!
//! Provides unified AST parsing and querying across multiple programming languages.
//! Uses rust-code-analysis (RCA) as the primary backend for parsing and traversal.
//!
//! # RCA Integration
//!
//! This module uses RCA's `action()`, `find()`, `guess_language()` directly:
//! - Language detection: `rust_code_analysis::guess_language()`
//! - Parsing: `rust_code_analysis::action::<Callback>(lang, source, path, None, cfg)`
//! - Node search: `rust_code_analysis::find(parser, &filters)`

pub mod core;
pub mod decoder;
pub mod query;
pub mod types;
pub mod unwrap_detector;

// Re-export public types and interfaces
pub use core::{AstNode, AstParseResult, Position, Span};
pub use decoder::AstDecoder;
pub use query::{AstQuery, AstQueryBuilder, AstQueryPatterns, QueryCondition};
pub use types::AstViolation;
pub use unwrap_detector::{UnwrapDetection, UnwrapDetector};

// Re-export RCA types for direct usage (NO wrappers)
pub use rust_code_analysis::{
    Callback, LANG, Node, ParserTrait, Search, action, find, guess_language,
};
