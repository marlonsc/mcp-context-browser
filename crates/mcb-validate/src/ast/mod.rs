//! AST Analysis Module
//!
//! Provides unified AST parsing and querying across multiple programming languages
//! using Tree-sitter parsers.

pub mod decoder;
pub mod languages;
pub mod query;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::Result;

/// Unified AST node representation across all languages
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    /// Node type (function, class, variable, etc.)
    pub kind: String,
    /// Node name (function name, variable name, etc.)
    pub name: Option<String>,
    /// Source code span (start/end positions)
    pub span: Span,
    /// Child nodes
    pub children: Vec<AstNode>,
    /// Additional metadata (language-specific)
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Source code position span
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// Position in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// AST parsing result
#[derive(Debug)]
pub struct AstParseResult {
    pub root: AstNode,
    pub errors: Vec<String>,
}

/// AST query for pattern matching
#[derive(Debug, Clone)]
pub struct AstQuery {
    pub language: String,
    pub pattern: String,
    pub message: String,
    pub severity: String,
}

/// Language-specific AST parser
pub trait AstParser: Send + Sync {
    fn language(&self) -> &'static str;
    fn parse_file(&self, path: &Path) -> Result<AstParseResult>;
    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult>;
}

/// Unified AST engine for multi-language analysis
pub struct AstEngine {
    parsers: HashMap<String, Arc<dyn AstParser>>,
    queries: HashMap<String, Vec<AstQuery>>,
}

impl AstEngine {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Register language parsers
        parsers.insert("rust".to_string(), Arc::new(languages::RustParser::new()));
        parsers.insert("python".to_string(), Arc::new(languages::PythonParser::new()));
        parsers.insert("javascript".to_string(), Arc::new(languages::JavaScriptParser::new()));
        parsers.insert("typescript".to_string(), Arc::new(languages::TypeScriptParser::new()));
        parsers.insert("go".to_string(), Arc::new(languages::GoParser::new()));

        Self {
            parsers,
            queries: HashMap::new(),
        }
    }

    pub fn register_query(&mut self, rule_id: String, query: AstQuery) {
        self.queries.entry(rule_id).or_insert_with(Vec::new).push(query);
    }

    pub fn get_parser(&self, language: &str) -> Option<&Arc<dyn AstParser>> {
        self.parsers.get(language)
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.parsers.keys().map(|s| s.as_str()).collect()
    }

    pub fn detect_language(&self, path: &Path) -> Option<&str> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "tsx" => Some("typescript"),
            "go" => Some("go"),
            _ => None,
        }
    }
}

impl Default for AstEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// AST-based violation
#[derive(Debug, Clone)]
pub struct AstViolation {
    pub rule_id: String,
    pub file: String,
    pub node: AstNode,
    pub message: String,
    pub severity: String,
}

pub use decoder::AstDecoder;
pub use languages::*;
pub use query::{AstQuery, AstQueryBuilder, AstQueryPatterns, QueryCondition};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_engine_creation() {
        let engine = AstEngine::new();
        assert!(!engine.supported_languages().is_empty());
    }

    #[test]
    fn test_language_detection() {
        let engine = AstEngine::new();

        assert_eq!(engine.detect_language(Path::new("main.rs")), Some("rust"));
        assert_eq!(engine.detect_language(Path::new("script.py")), Some("python"));
        assert_eq!(engine.detect_language(Path::new("app.js")), Some("javascript"));
        assert_eq!(engine.detect_language(Path::new("component.ts")), Some("typescript"));
        assert_eq!(engine.detect_language(Path::new("server.go")), Some("go"));
        assert_eq!(engine.detect_language(Path::new("unknown.xyz")), None);
    }
}