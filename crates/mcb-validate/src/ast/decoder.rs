//! AST Decoder
//!
//! Converts Tree-sitter concrete syntax trees to unified AstNode format.

use std::collections::HashMap;
use tree_sitter::Node;

use super::{AstNode, AstParseResult, Position, Span};

/// Decoder for Tree-sitter AST to unified format
pub struct AstDecoder;

impl AstDecoder {
    /// Convert Tree-sitter tree to unified AstNode
    pub fn decode_tree(tree: &tree_sitter::Tree, source: &str) -> AstNode {
        Self::decode_node(tree.root_node(), source)
    }

    /// Convert Tree-sitter node to unified AstNode
    pub fn decode_node(node: Node, source: &str) -> AstNode {
        let kind = node.kind().to_string();
        let span = Self::decode_span(node, source);
        let name = Self::extract_name(&node, source);

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Skip comment and whitespace nodes for cleaner AST
            if !Self::is_noise_node(&child) {
                children.push(Self::decode_node(child, source));
            }
        }

        let metadata = Self::extract_metadata(&node, source);

        AstNode {
            kind,
            name,
            span,
            children,
            metadata,
        }
    }

    /// Check if node is noise (comments, whitespace, etc.)
    fn is_noise_node(node: &Node) -> bool {
        matches!(node.kind(),
            "comment" | "line_comment" | "block_comment" |
            "whitespace" | "newline" | "indent" | "dedent" |
            ";" | "," | "(" | ")" | "[" | "]" | "{" | "}"
        )
    }

    /// Extract node span from Tree-sitter
    fn decode_span(node: Node, source: &str) -> Span {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let start_pos = node.start_position();
        let end_pos = node.end_position();

        Span {
            start: Position {
                line: start_pos.row + 1, // 1-based indexing
                column: start_pos.column + 1,
                byte_offset: start_byte,
            },
            end: Position {
                line: end_pos.row + 1,
                column: end_pos.column + 1,
                byte_offset: end_byte,
            },
        }
    }

    /// Extract node name from various node types
    fn extract_name(node: &Node, source: &str) -> Option<String> {
        // Try different strategies to extract meaningful names
        Self::extract_identifier_name(node, source)
            .or_else(|| Self::extract_string_literal(node, source))
            .or_else(|| Self::extract_type_name(node, source))
    }

    /// Extract identifier names (function names, variable names, etc.)
    fn extract_identifier_name(node: &Node, source: &str) -> Option<String> {
        // Look for identifier children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                let text = child.utf8_text(source.as_bytes()).ok()?;
                return Some(text.to_string());
            }
        }

        // If node itself is an identifier
        if node.kind() == "identifier" || node.kind() == "type_identifier" {
            node.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Extract string literal content
    fn extract_string_literal(node: &Node, source: &str) -> Option<String> {
        if node.kind().contains("string") {
            node.utf8_text(source.as_bytes()).ok().map(|s| {
                // Remove quotes
                s.trim_matches('"').trim_matches('\'').to_string()
            })
        } else {
            None
        }
    }

    /// Extract type names
    fn extract_type_name(node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind().contains("type") && child.kind() != "type" {
                return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract language-specific metadata
    fn extract_metadata(node: &Node, source: &str) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();

        // Add basic node information
        metadata.insert("node_type".to_string(), node.kind().into());
        metadata.insert("is_named".to_string(), node.is_named().into());
        metadata.insert("is_missing".to_string(), node.is_missing().into());
        metadata.insert("has_error".to_string(), node.has_error().into());

        // Add source text for debugging
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            metadata.insert("source_text".to_string(), text.to_string().into());
        }

        // Language-specific metadata can be added here
        match node.kind() {
            "function_item" | "function_definition" | "function_declaration" => {
                metadata.insert("is_function".to_string(), true.into());
                Self::extract_function_metadata(node, source, &mut metadata);
            }
            "struct_item" | "class_definition" => {
                metadata.insert("is_type".to_string(), true.into());
            }
            _ => {}
        }

        metadata
    }

    /// Extract function-specific metadata
    fn extract_function_metadata(node: &Node, source: &str, metadata: &mut HashMap<String, serde_json::Value>) {
        // Count parameters
        let param_count = Self::count_function_parameters(node);
        metadata.insert("parameter_count".to_string(), param_count.into());

        // Check if async
        let is_async = Self::is_async_function(node);
        metadata.insert("is_async".to_string(), is_async.into());

        // Check if has return type
        let has_return_type = Self::has_return_type(node);
        metadata.insert("has_return_type".to_string(), has_return_type.into());

        // Count lines
        let line_count = node.end_position().row - node.start_position().row + 1;
        metadata.insert("line_count".to_string(), line_count.into());
    }

    /// Count function parameters
    fn count_function_parameters(node: &Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind().contains("parameter") || child.kind() == "parameters" {
                count += child.child_count();
            }
        }

        count
    }

    /// Check if function is async
    fn is_async_function(node: &Node) -> bool {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "async" || child.kind() == "await" {
                return true;
            }
        }

        false
    }

    /// Check if function has return type
    fn has_return_type(node: &Node) -> bool {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind().contains("return") || child.kind().contains("type") {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_span() {
        // Test span decoding (would need actual Tree-sitter node)
        // This is a placeholder for actual tests
        assert!(true);
    }

    #[test]
    fn test_extract_name() {
        // Test name extraction (would need actual Tree-sitter node)
        // This is a placeholder for actual tests
        assert!(true);
    }
}