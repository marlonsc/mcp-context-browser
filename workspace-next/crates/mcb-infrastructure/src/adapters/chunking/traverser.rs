//! AST traverser for extracting code chunks based on rules
//!
//! This module provides the AstTraverser that walks tree-sitter ASTs
//! and extracts code chunks according to configurable rules.

use super::config::NodeExtractionRule;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::Language;
use std::collections::HashMap;

/// Parameters for creating a code chunk
#[derive(Debug)]
struct ChunkParams<'a> {
    content: String,
    file_name: &'a str,
    node_type: &'a str,
    depth: usize,
    priority: i32,
    chunk_index: usize,
}

/// Generic AST node traverser with configurable rules
pub struct AstTraverser<'a> {
    rules: &'a [NodeExtractionRule],
    language: &'a Language,
    max_chunks: usize,
}

impl<'a> AstTraverser<'a> {
    /// Create a new AST traverser with extraction rules and language configuration
    pub fn new(rules: &'a [NodeExtractionRule], language: &'a Language) -> Self {
        Self {
            rules,
            language,
            max_chunks: 100,
        }
    }

    /// Configure the maximum number of chunks to extract
    pub fn with_max_chunks(mut self, max_chunks: usize) -> Self {
        self.max_chunks = max_chunks;
        self
    }

    /// Traverse the AST and extract code chunks according to the configured rules
    pub fn traverse_and_extract(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        file_name: &str,
        depth: usize,
        chunks: &mut Vec<CodeChunk>,
    ) {
        // Stop if we've reached the chunk limit
        if chunks.len() >= self.max_chunks {
            return;
        }

        loop {
            let node = cursor.node();
            let node_type = node.kind();

            // Check if this node matches any extraction rule
            for rule in self.rules {
                if rule.node_types.contains(&node_type.to_string()) {
                    let (code, context) = if rule.include_context {
                        Self::extract_node_with_context(node, content, 3)
                    } else {
                        (Self::extract_node_content(node, content).ok(), None)
                    };

                    if let Some(code) = code {
                        if code.len() >= rule.min_length && code.lines().count() >= rule.min_lines {
                            let chunk_params = ChunkParams {
                                content: code,
                                file_name,
                                node_type,
                                depth,
                                priority: rule.priority,
                                chunk_index: chunks.len(),
                            };
                            let mut chunk = self.create_chunk_from_node(node, chunk_params);

                            // Add context metadata if available
                            if let Some(context_lines) = context {
                                if let Some(metadata) = chunk.metadata.as_object_mut() {
                                    metadata.insert(
                                        "context_lines".to_string(),
                                        serde_json::json!(context_lines),
                                    );
                                }
                            }

                            chunks.push(chunk);

                            if chunks.len() >= self.max_chunks {
                                return;
                            }
                        }
                    }
                }
            }

            // Recurse into children if within depth limit
            for rule in self.rules {
                if depth < rule.max_depth && cursor.goto_first_child() {
                    self.traverse_and_extract(cursor, content, file_name, depth + 1, chunks);

                    if chunks.len() >= self.max_chunks {
                        return;
                    }

                    cursor.goto_parent();
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn extract_node_content(node: tree_sitter::Node, content: &str) -> Result<String> {
        let start = node.start_byte();
        let end = node.end_byte();

        if start >= content.len() || end > content.len() || start >= end {
            return Err(Error::internal("Invalid node range".to_string()));
        }

        let code = content[start..end].trim();
        if code.is_empty() {
            return Err(Error::internal("Empty node content".to_string()));
        }

        Ok(code.to_string())
    }

    fn extract_node_with_context(
        node: tree_sitter::Node,
        content: &str,
        context_lines: usize,
    ) -> (Option<String>, Option<usize>) {
        let start = node.start_byte();
        let end = node.end_byte();

        if start >= content.len() || end > content.len() || start >= end {
            return (None, None);
        }

        let code = content[start..end].trim();
        if code.is_empty() {
            return (None, None);
        }

        let lines: Vec<&str> = content.lines().collect();
        let start_line = content[..start].lines().count();
        let end_line = start_line + code.lines().count() - 1;

        let context_start = start_line.saturating_sub(context_lines);
        let context_end = (end_line + context_lines).min(lines.len());

        let context = lines[context_start..context_end].join("\n");

        (Some(context), Some(context_lines))
    }

    fn create_chunk_from_node(&self, node: tree_sitter::Node, params: ChunkParams) -> CodeChunk {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;

        CodeChunk {
            id: format!(
                "{}_{}_{}_{}_{}_{}",
                params.file_name,
                params.node_type,
                start_line,
                end_line,
                params.priority,
                params.chunk_index
            ),
            content: params.content,
            file_path: params.file_name.to_string(),
            start_line: start_line as u32,
            end_line: end_line as u32,
            language: self.language.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file".to_string(), serde_json::json!(params.file_name));
                meta.insert("node_type".to_string(), serde_json::json!(params.node_type));
                meta.insert("depth".to_string(), serde_json::json!(params.depth));
                meta.insert("priority".to_string(), serde_json::json!(params.priority));
                serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
            },
        }
    }
}
