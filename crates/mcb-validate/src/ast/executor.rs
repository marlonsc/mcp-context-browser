//! AST Query Executor Module
//!
//! Executes YAML rules that use AST selectors or ast_query for tree-sitter-based validation.
//! This is the Phase 2 deliverable: YAML rule → Tree-sitter → violations pipeline.

use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

use crate::rules::yaml_loader::{AstSelector, ValidatedRule};
use crate::{Result, ValidationError};

/// Violation from AST query execution (Phase 2)
/// Lighter weight than AstQueryViolation from types.rs - doesn't include full AstNode
#[derive(Debug, Clone)]
pub struct AstQueryViolation {
    /// Rule ID that generated this violation
    pub rule_id: String,
    /// File where the violation occurred
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Violation message
    pub message: String,
    /// Severity level
    pub severity: String,
    /// The matched source text (context)
    pub context: String,
}

/// Execute YAML rules that use AST selectors for tree-sitter-based validation
///
/// This is the Phase 2 deliverable: YAML rule → Tree-sitter → violations pipeline
pub struct AstQueryExecutor;

impl AstQueryExecutor {
    /// Execute a rule's AST selectors against files
    ///
    /// # Arguments
    /// * `rule` - The validated YAML rule with selectors or ast_query
    /// * `files` - Files to check
    ///
    /// # Returns
    /// Violations that match the rule's AST patterns
    pub async fn execute_rule(rule: &ValidatedRule, files: &[&Path]) -> Result<Vec<AstQueryViolation>> {
        // Skip if no AST selectors or ast_query
        if rule.selectors.is_empty() && rule.ast_query.is_none() {
            return Ok(vec![]);
        }

        // Skip if rule is disabled
        if !rule.enabled {
            return Ok(vec![]);
        }

        let mut all_violations = Vec::new();

        for file in files {
            // Detect language from file extension
            let language = Self::detect_language(file);
            if language.is_none() {
                continue;
            }
            let lang = language.unwrap();

            // Check if this file matches any selector's language
            let matching_selectors: Vec<_> = rule
                .selectors
                .iter()
                .filter(|s| s.language == lang)
                .collect();

            if matching_selectors.is_empty() && rule.ast_query.is_none() {
                continue;
            }

            // Read file content
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Execute selectors
            for selector in matching_selectors {
                let violations =
                    Self::execute_selector(selector, &content, file, rule).await?;
                all_violations.extend(violations);
            }

            // Execute ast_query if present
            if let Some(ref query_str) = rule.ast_query {
                let violations =
                    Self::execute_ast_query(query_str, &lang, &content, file, rule).await?;
                all_violations.extend(violations);
            }
        }

        Ok(all_violations)
    }

    /// Execute a single AST selector against file content
    async fn execute_selector(
        selector: &AstSelector,
        content: &str,
        file: &Path,
        rule: &ValidatedRule,
    ) -> Result<Vec<AstQueryViolation>> {
        // Build the query string from selector
        let query_str = if let Some(ref pattern) = selector.pattern {
            pattern.clone()
        } else {
            // Default query: match all nodes of the specified type
            format!("({}) @match", selector.node_type)
        };

        Self::execute_ast_query(&query_str, &selector.language, content, file, rule).await
    }

    /// Execute a Tree-sitter query against file content
    async fn execute_ast_query(
        query_str: &str,
        language: &str,
        content: &str,
        file: &Path,
        rule: &ValidatedRule,
    ) -> Result<Vec<AstQueryViolation>> {
        // Get Tree-sitter language
        let ts_language = Self::get_tree_sitter_language(language)?;

        // Create parser
        let mut parser = Parser::new();
        parser
            .set_language(&ts_language)
            .map_err(|e| ValidationError::Config(format!("Failed to set language: {e}")))?;

        // Parse content
        let tree = parser.parse(content, None).ok_or_else(|| ValidationError::Parse {
            file: file.to_path_buf(),
            message: "Failed to parse file".into(),
        })?;

        // Compile query
        let query = Query::new(&ts_language, query_str).map_err(|e| {
            ValidationError::Config(format!("Failed to compile query '{}': {}", query_str, e))
        })?;

        // Execute query
        let mut violations = Vec::new();
        let mut cursor = QueryCursor::new();
        let source_bytes = content.as_bytes();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        // Find the @match capture index (or first capture)
        let capture_idx = query
            .capture_index_for_name("match")
            .or_else(|| query.capture_index_for_name("call"))
            .or_else(|| query.capture_index_for_name("func"))
            .unwrap_or(0);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                if capture.index == capture_idx || query.capture_names().len() == 1 {
                    let node = capture.node;
                    let start_pos = node.start_position();

                    // Get context (source text)
                    let context = node
                        .utf8_text(source_bytes)
                        .unwrap_or("")
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    let message = rule
                        .message
                        .clone()
                        .unwrap_or_else(|| rule.description.clone());

                    violations.push(AstQueryViolation {
                        rule_id: rule.id.clone(),
                        file: file.to_string_lossy().to_string(),
                        line: start_pos.row + 1,
                        column: start_pos.column + 1,
                        message,
                        severity: rule.severity.clone(),
                        context,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Detect programming language from file extension
    ///
    /// Returns only languages that have Tree-sitter support in this crate
    fn detect_language(file: &Path) -> Option<String> {
        let ext = file.extension()?.to_str()?;
        match ext {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "go" => Some("go".to_string()),
            // java, c, cpp not included - require additional tree-sitter crates
            _ => None,
        }
    }

    /// Get Tree-sitter language from language name
    ///
    /// Currently supports: rust, python, javascript, typescript, go
    /// Additional languages can be added by including the tree-sitter-* crate
    fn get_tree_sitter_language(language: &str) -> Result<tree_sitter::Language> {
        match language {
            "rust" => Ok(tree_sitter_rust::LANGUAGE.into()),
            "python" => Ok(tree_sitter_python::LANGUAGE.into()),
            "javascript" => Ok(tree_sitter_javascript::LANGUAGE.into()),
            "typescript" => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "go" => Ok(tree_sitter_go::LANGUAGE.into()),
            // Note: java, c, cpp can be added by including their tree-sitter crates
            _ => Err(ValidationError::Config(format!(
                "Unsupported language: {}. Supported: rust, python, javascript, typescript, go",
                language
            ))),
        }
    }
}
