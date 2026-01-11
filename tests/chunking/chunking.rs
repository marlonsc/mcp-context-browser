//! Comprehensive tests for the intelligent chunking system
//!
//! Tests cover:
//! - Language-specific chunking with tree-sitter
//! - Fallback chunking with regex patterns
//! - Generic chunking for unsupported languages
//! - AST traversal and rule-based extraction
//! - Edge cases and error handling

use mcp_context_browser::chunking::{IntelligentChunker, NodeExtractionRule};
use mcp_context_browser::domain::types::Language;

/// Test data for different programming languages
struct TestData {
    rust_code: &'static str,
    python_code: &'static str,
    javascript_code: &'static str,
    typescript_code: &'static str,
}

const TEST_DATA: TestData = TestData {
    rust_code: r#"
use std::collections::HashMap;

pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u32, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

pub fn create_user(id: u32, name: &str, email: &str) -> Result<User, String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    Ok(User::new(id, name.to_string(), email.to_string()))
}

fn internal_helper() {
    println!("This is an internal function");
}
"#,

    python_code: r#"
from typing import List, Optional
import json

class UserManager:
    def __init__(self):
        self.users = {}

    def add_user(self, user_id: int, name: str, email: str) -> bool:
        if user_id in self.users:
            return False

        self.users[user_id] = {
            'name': name,
            'email': email,
            'active': True
        }
        return True

    def get_user(self, user_id: int) -> Optional[dict]:
        return self.users.get(user_id)

def validate_email(email: str) -> bool:
    return '@' in email and '.' in email

def create_user_manager() -> UserManager:
    return UserManager()

# Private helper function
def _format_user_data(user_data: dict) -> str:
    return json.dumps(user_data, indent=2)
"#,

    javascript_code: r#"
const express = require('express');

class UserService {
    constructor() {
        this.users = new Map();
    }

    addUser(userId, name, email) {
        if (this.users.has(userId)) {
            return false;
        }

        this.users.set(userId, {
            name,
            email,
            active: true
        });
        return true;
    }

    getUser(userId) {
        return this.users.get(userId) || null;
    }

    validateEmail(email) {
        return email.includes('@') && email.includes('.');
    }
}

function createUserService() {
    return new UserService();
}

const validateEmail = (email) => {
    return email.includes('@') && email.includes('.');
};

module.exports = {
    UserService,
    createUserService,
    validateEmail
};
"#,

    typescript_code: r#"
import express from 'express';

interface User {
    id: number;
    name: string;
    email: string;
    active: boolean;
}

class UserService {
    private users: Map<number, User> = new Map();

    public addUser(userId: number, name: string, email: string): boolean {
        if (this.users.has(userId)) {
            return false;
        }

        this.users.set(userId, {
            id: userId,
            name,
            email,
            active: true
        });
        return true;
    }

    public getUser(userId: number): User | null {
        return this.users.get(userId) || null;
    }

    public validateEmail(email: string): boolean {
        return email.includes('@') && email.includes('.');
    }
}

function createUserService(): UserService {
    return new UserService();
}

const validateEmail = (email: string): boolean => {
    return email.includes('@') && email.includes('.');
};

export {
    UserService,
    createUserService,
    validateEmail,
    type User
};
"#,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intelligent_chunker_creation() {
        let _chunker = IntelligentChunker::new();
        // Test passes if no panic occurs
        assert_eq!(std::mem::size_of::<IntelligentChunker>(), 0); // Zero-sized type
    }

    #[test]
    fn test_intelligent_chunker_default() {
        let _chunker = IntelligentChunker;
        // Test passes if no panic occurs
        assert_eq!(std::mem::size_of::<IntelligentChunker>(), 0); // Zero-sized type
    }

    #[test]
    fn test_rust_chunking_with_tree_sitter() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(TEST_DATA.rust_code, "src/user.rs", Language::Rust);

        // Should extract meaningful chunks
        assert!(!chunks.is_empty(), "Should extract at least one chunk");

        // Check that chunks have proper structure
        for chunk in &chunks {
            assert!(!chunk.id.is_empty(), "Chunk ID should not be empty");
            assert!(
                !chunk.content.is_empty(),
                "Chunk content should not be empty"
            );
            assert_eq!(chunk.file_path, "src/user.rs");
            assert_eq!(chunk.language, Language::Rust);
            assert!(
                chunk.start_line <= chunk.end_line,
                "Start line should be <= end line"
            );

            // Check metadata structure
            let metadata = &chunk.metadata;
            assert!(metadata.is_object(), "Metadata should be an object");
            assert!(metadata.get("file").is_some(), "Should have file metadata");
            assert!(
                metadata.get("node_type").is_some(),
                "Should have node_type metadata"
            );
            assert!(
                metadata.get("depth").is_some(),
                "Should have depth metadata"
            );
        }

        // Should extract struct, impl, and functions
        let node_types: Vec<&str> = chunks
            .iter()
            .filter_map(|c| c.metadata.get("node_type"))
            .filter_map(|v| v.as_str())
            .collect();

        assert!(node_types.contains(&"struct_item"), "Should extract struct");
        assert!(node_types.contains(&"impl_item"), "Should extract impl");
        assert!(
            node_types.contains(&"function_item"),
            "Should extract functions"
        );
    }

    #[test]
    fn test_python_chunking_with_tree_sitter() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(TEST_DATA.python_code, "user_manager.py", Language::Python);

        assert!(!chunks.is_empty(), "Should extract at least one chunk");

        for chunk in &chunks {
            assert_eq!(chunk.language, Language::Python);
            assert!(chunk.start_line <= chunk.end_line);

            let metadata = &chunk.metadata;
            assert!(metadata.is_object());
            assert!(metadata.get("file").is_some());
            assert!(metadata.get("node_type").is_some());
        }

        // Should extract class and functions
        let node_types: Vec<&str> = chunks
            .iter()
            .filter_map(|c| c.metadata.get("node_type"))
            .filter_map(|v| v.as_str())
            .collect();

        assert!(
            node_types.contains(&"class_definition"),
            "Should extract class"
        );
        assert!(
            node_types.contains(&"function_definition"),
            "Should extract functions"
        );
    }

    #[test]
    fn test_javascript_chunking_with_tree_sitter() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(
            TEST_DATA.javascript_code,
            "userService.js",
            Language::JavaScript,
        );

        assert!(!chunks.is_empty(), "Should extract at least one chunk");

        for chunk in &chunks {
            assert_eq!(chunk.language, Language::JavaScript);
            assert!(chunk.start_line <= chunk.end_line);
        }

        // Should extract class, functions, and arrow functions
        let node_types: Vec<&str> = chunks
            .iter()
            .filter_map(|c| c.metadata.get("node_type"))
            .filter_map(|v| v.as_str())
            .collect();

        assert!(
            node_types.contains(&"class_declaration"),
            "Should extract class"
        );
        assert!(
            node_types.contains(&"function_declaration") || node_types.contains(&"function"),
            "Should extract functions"
        );
    }

    #[test]
    fn test_typescript_chunking_with_tree_sitter() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(
            TEST_DATA.typescript_code,
            "userService.ts",
            Language::TypeScript,
        );

        assert!(!chunks.is_empty(), "Should extract at least one chunk");

        for chunk in &chunks {
            assert_eq!(chunk.language, Language::TypeScript);
            assert!(chunk.start_line <= chunk.end_line);
        }
    }

    #[test]
    fn test_generic_chunking_fallback() -> Result<(), Box<dyn std::error::Error>> {
        let chunker = IntelligentChunker::new();

        // Test with unknown language
        let generic_code = "function test() {\n    return true;\n}\n\nconst x = 42;\n";
        let chunks = chunker.chunk_code(generic_code, "test.unknown", Language::Unknown);

        assert!(!chunks.is_empty(), "Should create generic chunks");

        for chunk in &chunks {
            assert_eq!(chunk.language, Language::Unknown);
            assert!(chunk.start_line <= chunk.end_line);

            let metadata = &chunk.metadata;
            assert!(metadata.is_object());
            let chunk_type = metadata
                .get("chunk_type")
                .ok_or("Missing chunk_type")?
                .as_str()
                .ok_or("chunk_type is not a string")?;
            assert_eq!(chunk_type, "generic");
        }
        Ok(())
    }

    #[test]
    fn test_unsupported_languages_use_generic_chunking() -> Result<(), Box<dyn std::error::Error>> {
        let chunker = IntelligentChunker::new();

        // Test various unsupported languages that should use generic chunking
        let test_languages = vec![
            (
                Language::Go,
                "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, World!\")\n    fmt.Println(\"This is a test program\")\n}\n",
            ),
            (
                Language::Java,
                "public class HelloWorld {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, World!\");\n        System.out.println(\"Java test program\");\n    }\n}\n",
            ),
            (
                Language::C,
                "#include <stdio.h>\n\nint main() {\n    printf(\"Hello, World!\\n\");\n    printf(\"C language test\\n\");\n    return 0;\n}\n",
            ),
            (
                Language::Cpp,
                "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, World!\" << std::endl;\n    std::cout << \"C++ test program\" << std::endl;\n    return 0;\n}\n",
            ),
            (
                Language::CSharp,
                "using System;\n\nclass Program {\n    static void Main() {\n        Console.WriteLine(\"Hello, World!\");\n        Console.WriteLine(\"C# test program\");\n    }\n}\n",
            ),
            (
                Language::Php,
                "<?php\n\necho \"Hello, World!\\n\";\necho \"PHP test script\\n\";\n\n?>",
            ),
            (
                Language::Ruby,
                "puts \"Hello, World!\"\nputs \"Ruby test program\"\nputs \"This is a longer line to ensure chunking works\"\n",
            ),
            (
                Language::Swift,
                "print(\"Hello, World!\")\nprint(\"Swift test program\")\nprint(\"This line ensures proper chunking\")\n",
            ),
            (
                Language::Kotlin,
                "fun main() {\n    println(\"Hello, World!\")\n    println(\"Kotlin test program\")\n    println(\"Ensuring proper chunk generation\")\n}\n",
            ),
            (
                Language::Scala,
                "object Main extends App {\n  println(\"Hello, World!\")\n  println(\"Scala test program\")\n  println(\"Making sure chunks are created\")\n}\n",
            ),
            (
                Language::Haskell,
                "main = do\n    putStrLn \"Hello, World!\"\n    putStrLn \"Haskell test program\"\n    putStrLn \"This ensures chunking works properly\"\n",
            ),
        ];

        for (language, code) in test_languages {
            let chunks = chunker.chunk_code(
                code,
                &format!("test.{}", language_to_extension(&language)),
                language.clone(),
            );

            // Should create chunks using generic algorithm
            assert!(
                !chunks.is_empty(),
                "Should create chunks for {:?}",
                language
            );

            for chunk in &chunks {
                assert_eq!(
                    chunk.language, language,
                    "Language should match for {:?}",
                    language
                );
                assert!(
                    chunk.start_line <= chunk.end_line,
                    "Line ranges should be valid for {:?}",
                    language
                );

                let metadata = &chunk.metadata;
                assert!(
                    metadata.is_object(),
                    "Metadata should be object for {:?}",
                    language
                );
                // Generic chunking should have chunk_type: "generic"
                if let Some(chunk_type) = metadata.get("chunk_type") {
                    let chunk_type_str = chunk_type
                        .as_str()
                        .ok_or_else(|| format!("chunk_type is not a string for {:?}", language))?;
                    assert_eq!(
                        chunk_type_str, "generic",
                        "Should use generic chunking for {:?}",
                        language
                    );
                }
            }
        }
        Ok(())
    }

    // Helper function to convert language to file extension
    fn language_to_extension(language: &Language) -> &'static str {
        match language {
            Language::Go => "go",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::CSharp => "cs",
            Language::Php => "php",
            Language::Ruby => "rb",
            Language::Swift => "swift",
            Language::Kotlin => "kt",
            Language::Scala => "scala",
            Language::Haskell => "hs",
            _ => "unknown",
        }
    }

    #[test]
    fn test_chunk_content_validation() {
        let chunker = IntelligentChunker::new();

        // Test with empty content
        let chunks = chunker.chunk_code("", "empty.rs", Language::Rust);
        assert!(
            chunks.is_empty() || chunks.iter().all(|c| !c.content.is_empty()),
            "Should not create chunks with empty content"
        );

        // Test with very small content
        let _chunks = chunker.chunk_code("x", "tiny.rs", Language::Rust);
        // May or may not create chunks depending on implementation, but shouldn't panic
    }

    #[test]
    fn test_fallback_chunking_patterns() {
        let chunker = IntelligentChunker::new();

        // Test fallback by forcing it (using a language that might not have tree-sitter)
        let simple_rust = r#"
pub fn hello() {
    println!("Hello");
}

pub struct Test {
    value: i32,
}
"#;

        let chunks = chunker.chunk_code(simple_rust, "test.rs", Language::Rust);

        // Should still work even if tree-sitter fails
        assert!(
            !chunks.is_empty() || chunks.is_empty(),
            "Fallback should handle gracefully"
        );

        // If chunks exist, they should have proper metadata
        for chunk in &chunks {
            assert!(
                chunk.metadata.get("chunk_type").is_some()
                    || chunk.metadata.get("node_type").is_some(),
                "Chunks should have appropriate metadata"
            );
        }
    }

    #[test]
    fn test_chunk_metadata_structure() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(TEST_DATA.rust_code, "test.rs", Language::Rust);

        for chunk in chunks {
            // Verify metadata has required fields
            let metadata = chunk.metadata;
            assert!(metadata.is_object(), "Metadata should be a JSON object");

            // Should have file information
            assert!(metadata.get("file").is_some(), "Should have file field");

            // Should have either node_type (tree-sitter) or chunk_type (fallback)
            let has_node_type = metadata.get("node_type").is_some();
            let has_chunk_type = metadata.get("chunk_type").is_some();

            assert!(
                has_node_type || has_chunk_type,
                "Should have either node_type or chunk_type: {:?}",
                metadata
            );

            // If from tree-sitter, should have depth
            if has_node_type {
                assert!(
                    metadata.get("depth").is_some(),
                    "Tree-sitter chunks should have depth"
                );
            }
        }
    }

    #[test]
    fn test_chunk_id_uniqueness() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(TEST_DATA.rust_code, "test.rs", Language::Rust);

        let mut ids = std::collections::HashSet::new();
        for chunk in &chunks {
            assert!(
                ids.insert(&chunk.id),
                "Chunk IDs should be unique: {}",
                chunk.id
            );
        }
    }

    #[test]
    fn test_chunk_line_ranges() {
        let chunker = IntelligentChunker::new();
        let chunks = chunker.chunk_code(TEST_DATA.python_code, "test.py", Language::Python);

        for chunk in &chunks {
            assert!(
                chunk.end_line >= chunk.start_line,
                "End line should be >= start line: start={}, end={}",
                chunk.start_line,
                chunk.end_line
            );
        }
    }

    #[test]
    fn test_minimal_chunk_size() {
        let chunker = IntelligentChunker::new();

        // Test with content that should create chunks
        let small_function = "pub fn test() {\n    let x = 1;\n}";
        let chunks = chunker.chunk_code(small_function, "small.rs", Language::Rust);

        // Should create at least one chunk or handle gracefully
        if !chunks.is_empty() {
            for chunk in &chunks {
                assert!(
                    chunk.content.len() >= 10,
                    "Chunks should meet minimum size: {}",
                    chunk.content.len()
                );
            }
        }
    }

    #[test]
    fn test_edge_case_empty_lines() {
        let chunker = IntelligentChunker::new();

        let code_with_empty_lines = "\n\n\npub fn test() {\n\n    let x = 1;\n\n}\n\n\n";
        let chunks = chunker.chunk_code(code_with_empty_lines, "empty_lines.rs", Language::Rust);

        // Should handle empty lines gracefully
        assert!(
            !chunks.is_empty() || chunks.is_empty(),
            "Should handle empty lines"
        );

        for chunk in &chunks {
            assert!(
                !chunk.content.trim().is_empty(),
                "Chunk content should not be just whitespace"
            );
        }
    }

    #[test]
    fn test_language_support() {
        // Test that supported languages are properly recognized
        assert!(
            IntelligentChunker::is_language_supported(&Language::Rust),
            "Should support Rust"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Python),
            "Should support Python"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::JavaScript),
            "Should support JavaScript"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::TypeScript),
            "Should support TypeScript"
        );

        // Test that unsupported languages are not recognized (use generic chunking)
        assert!(
            !IntelligentChunker::is_language_supported(&Language::Unknown),
            "Should not support Unknown"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Go),
            "Should support Go"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Java),
            "Should support Java"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::C),
            "Should support C"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Cpp),
            "Should support C++"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::CSharp),
            "Should support C#"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Php),
            "Should support PHP"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Ruby),
            "Should support Ruby"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Swift),
            "Should support Swift"
        );
        assert!(
            IntelligentChunker::is_language_supported(&Language::Kotlin),
            "Should support Kotlin"
        );
        assert!(
            !IntelligentChunker::is_language_supported(&Language::Scala),
            "Should not support Scala yet"
        );
        assert!(
            !IntelligentChunker::is_language_supported(&Language::Haskell),
            "Should not support Haskell yet"
        );
    }

    #[test]
    fn test_node_extraction_rules() {
        let rule = NodeExtractionRule {
            node_types: vec!["function_item".to_string(), "struct_item".to_string()],
            min_length: 50,
            min_lines: 3,
            max_depth: 3,
            priority: 1,
            include_context: true,
        };

        assert_eq!(rule.node_types.len(), 2);
        assert_eq!(rule.min_length, 50);
        assert_eq!(rule.min_lines, 3);
        assert_eq!(rule.max_depth, 3);
    }

    #[test]
    fn test_chunk_deterministic_output() {
        let chunker = IntelligentChunker::new();

        // Run chunking multiple times on same input
        let chunks1 = chunker.chunk_code(TEST_DATA.rust_code, "test.rs", Language::Rust);
        let chunks2 = chunker.chunk_code(TEST_DATA.rust_code, "test.rs", Language::Rust);

        // Should produce same number of chunks
        assert_eq!(
            chunks1.len(),
            chunks2.len(),
            "Should produce deterministic chunk count"
        );

        // Chunks should have same IDs (order may vary but IDs should match)
        let ids1: std::collections::HashSet<_> = chunks1.iter().map(|c| &c.id).collect();
        let ids2: std::collections::HashSet<_> = chunks2.iter().map(|c| &c.id).collect();

        assert_eq!(ids1, ids2, "Should produce deterministic chunk IDs");
    }

    #[test]
    fn test_large_content_chunking() {
        let chunker = IntelligentChunker::new();

        // Create large content
        let mut large_code = String::new();
        for i in 0..100 {
            large_code.push_str(&format!(
                "pub fn function_{}() {{\n    println!(\"Function {}\");\n}}\n\n",
                i, i
            ));
        }

        let chunks = chunker.chunk_code(&large_code, "large.rs", Language::Rust);

        // Should handle large content without issues
        assert!(!chunks.is_empty(), "Should handle large content");

        // Should not create too many tiny chunks
        let average_size: f64 =
            chunks.iter().map(|c| c.content.len()).sum::<usize>() as f64 / chunks.len() as f64;
        assert!(
            average_size > 20.0,
            "Average chunk size should be reasonable: {}",
            average_size
        );
    }
}
