//! Configuration structures for intelligent code chunking
//!
//! This module defines the core configuration types used for language-specific
//! chunking rules and settings.

use crate::infrastructure::constants::{
    DEFAULT_CHUNK_SIZE, NODE_EXTRACTION_DEFAULT_PRIORITY, NODE_EXTRACTION_MAX_DEPTH,
    NODE_EXTRACTION_MIN_LENGTH, NODE_EXTRACTION_MIN_LINES,
};

/// Rule for extracting specific AST node types
#[derive(Debug, Clone)]
pub struct NodeExtractionRule {
    /// AST node types to extract (e.g., "function_item", "class_definition")
    pub node_types: Vec<String>,
    /// Minimum content length to consider for chunking
    pub min_length: usize,
    /// Minimum number of lines to consider for chunking
    pub min_lines: usize,
    /// Maximum depth to traverse in AST
    pub max_depth: usize,
    /// Priority for ordering chunks (higher = more important)
    pub priority: i32,
    /// Whether to include surrounding context
    pub include_context: bool,
}

/// Language-specific configuration for chunking
#[derive(Debug)]
pub struct LanguageConfig {
    /// Tree-sitter language function
    pub ts_language: tree_sitter::Language,
    /// Node extraction rules
    pub extraction_rules: Vec<NodeExtractionRule>,
    /// Fallback patterns for regex-based chunking
    pub fallback_patterns: Vec<String>,
    /// Chunk size for generic fallback
    pub chunk_size: usize,
}

impl LanguageConfig {
    /// Create a new language configuration
    pub fn new(language: tree_sitter::Language) -> Self {
        Self {
            ts_language: language,
            extraction_rules: Vec::new(),
            fallback_patterns: Vec::new(),
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    /// Add an extraction rule
    pub fn with_rule(mut self, rule: NodeExtractionRule) -> Self {
        self.extraction_rules.push(rule);
        self
    }

    /// Add multiple extraction rules
    pub fn with_rules(mut self, rules: Vec<NodeExtractionRule>) -> Self {
        self.extraction_rules.extend(rules);
        self
    }

    /// Add fallback patterns
    pub fn with_fallback_patterns(mut self, patterns: Vec<String>) -> Self {
        self.fallback_patterns = patterns;
        self
    }

    /// Set chunk size
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Get the tree-sitter language
    pub fn get_language(&self) -> tree_sitter::Language {
        self.ts_language.clone()
    }
}

/// Builder for NodeExtractionRule
pub struct NodeExtractionRuleBuilder {
    node_types: Vec<String>,
    min_length: usize,
    min_lines: usize,
    max_depth: usize,
    priority: i32,
    include_context: bool,
}

impl NodeExtractionRuleBuilder {
    /// Create a new rule builder
    pub fn new() -> Self {
        Self {
            node_types: Vec::new(),
            min_length: NODE_EXTRACTION_MIN_LENGTH,
            min_lines: NODE_EXTRACTION_MIN_LINES,
            max_depth: NODE_EXTRACTION_MAX_DEPTH,
            priority: NODE_EXTRACTION_DEFAULT_PRIORITY,
            include_context: false,
        }
    }

    /// Add node types to extract
    pub fn with_node_types(mut self, node_types: Vec<String>) -> Self {
        self.node_types = node_types;
        self
    }

    /// Add a single node type
    pub fn with_node_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_types.push(node_type.into());
        self
    }

    /// Set minimum length
    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = min_length;
        self
    }

    /// Set minimum lines
    pub fn with_min_lines(mut self, min_lines: usize) -> Self {
        self.min_lines = min_lines;
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set include context
    pub fn with_context(mut self, include_context: bool) -> Self {
        self.include_context = include_context;
        self
    }

    /// Build the rule
    pub fn build(self) -> NodeExtractionRule {
        NodeExtractionRule {
            node_types: self.node_types,
            min_length: self.min_length,
            min_lines: self.min_lines,
            max_depth: self.max_depth,
            priority: self.priority,
            include_context: self.include_context,
        }
    }
}

impl Default for NodeExtractionRuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}
