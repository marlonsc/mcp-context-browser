//! Tests for core types module
//!
//! This module tests the core data structures and their functionality.

use mcp_context_browser::domain::types::{CodeChunk, Embedding, Language, SearchResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension_rust() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
    }

    #[test]
    fn test_language_from_extension_python() {
        assert_eq!(Language::from_extension("py"), Language::Python);
    }

    #[test]
    fn test_language_from_extension_javascript() {
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
    }

    #[test]
    fn test_language_from_extension_typescript() {
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
    }

    #[test]
    fn test_language_from_extension_unknown() {
        assert_eq!(Language::from_extension("xyz"), Language::Unknown);
    }

    #[test]
    fn test_language_from_extension_case_insensitive() {
        assert_eq!(Language::from_extension("RS"), Language::Rust);
        assert_eq!(Language::from_extension("Py"), Language::Python);
    }

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk {
            id: "test-id".to_string(),
            content: "fn test() {}".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"author": "test"}),
        };

        assert_eq!(chunk.id, "test-id");
        assert_eq!(chunk.content, "fn test() {}");
        assert_eq!(chunk.file_path, "src/main.rs");
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 3);
        assert_eq!(chunk.language, Language::Rust);
        assert_eq!(chunk.metadata["author"], "test");
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: "test-id".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 42,
            content: "println!(\"Hello, world!\");".to_string(),
            score: 0.95,
            metadata: serde_json::json!({"context": "main function"}),
        };

        assert_eq!(result.id, "test-id");
        assert_eq!(result.file_path, "src/main.rs");
        assert_eq!(result.start_line, 42);
        assert_eq!(result.content, "println!(\"Hello, world!\");");
        assert_eq!(result.score, 0.95);
        assert_eq!(result.metadata["context"], "main function");
    }

    #[test]
    fn test_embedding_creation() {
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let embedding = Embedding {
            vector: vector.clone(),
            model: "test-model".to_string(),
            dimensions: 5,
        };

        assert_eq!(embedding.vector, vector);
        assert_eq!(embedding.model, "test-model");
        assert_eq!(embedding.dimensions, 5);
    }

    #[test]
    fn test_embedding_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3],
            model: "test-model".to_string(),
            dimensions: 3,
        };

        let json = serde_json::to_string(&embedding)?;
        let deserialized: Embedding = serde_json::from_str(&json)?;

        assert_eq!(embedding.vector, deserialized.vector);
        assert_eq!(embedding.model, deserialized.model);
        assert_eq!(embedding.dimensions, deserialized.dimensions);
        Ok(())
    }

    #[test]
    fn test_code_chunk_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let chunk = CodeChunk {
            id: "test-id".to_string(),
            content: "fn test() {}".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"author": "test"}),
        };

        let json = serde_json::to_string(&chunk)?;
        let deserialized: CodeChunk = serde_json::from_str(&json)?;

        assert_eq!(chunk.id, deserialized.id);
        assert_eq!(chunk.content, deserialized.content);
        assert_eq!(chunk.file_path, deserialized.file_path);
        assert_eq!(chunk.start_line, deserialized.start_line);
        assert_eq!(chunk.end_line, deserialized.end_line);
        assert_eq!(chunk.language, deserialized.language);
        assert_eq!(chunk.metadata, deserialized.metadata);
        Ok(())
    }

    #[test]
    fn test_search_result_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let result = SearchResult {
            id: "test-id".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 42,
            content: "println!(\"Hello, world!\");".to_string(),
            score: 0.95,
            metadata: serde_json::json!({"context": "main function"}),
        };

        let json = serde_json::to_string(&result)?;
        let deserialized: SearchResult = serde_json::from_str(&json)?;

        assert_eq!(result.id, deserialized.id);
        assert_eq!(result.file_path, deserialized.file_path);
        assert_eq!(result.start_line, deserialized.start_line);
        assert_eq!(result.content, deserialized.content);
        assert_eq!(result.score, deserialized.score);
        assert_eq!(result.metadata, deserialized.metadata);
        Ok(())
    }

    #[test]
    fn test_multiple_language_extensions() {
        let extensions = vec![
            ("cpp", Language::Cpp),
            ("cc", Language::Cpp),
            ("cxx", Language::Cpp),
            ("cs", Language::CSharp),
            ("kt", Language::Kotlin),
            ("scala", Language::Scala),
        ];

        for (ext, expected) in extensions {
            assert_eq!(Language::from_extension(ext), expected);
        }
    }

    #[test]
    fn test_empty_vector_embedding() {
        let embedding = Embedding {
            vector: vec![],
            model: "empty-model".to_string(),
            dimensions: 0,
        };

        assert_eq!(embedding.vector.len(), 0);
        assert_eq!(embedding.dimensions, 0);
    }

    #[test]
    fn test_large_embedding_vector() {
        let vector: Vec<f32> = (0..1000).map(|i| i as f32 * 0.001).collect();
        let embedding = Embedding {
            vector: vector.clone(),
            model: "large-model".to_string(),
            dimensions: 1000,
        };

        assert_eq!(embedding.vector.len(), 1000);
        assert_eq!(embedding.dimensions, 1000);
        assert_eq!(embedding.vector, vector);
    }

    #[test]
    fn test_code_chunk_with_empty_content() {
        let chunk = CodeChunk {
            id: "empty-id".to_string(),
            content: String::new(),
            file_path: "src/empty.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        assert_eq!(chunk.content, "");
        assert_eq!(chunk.start_line, chunk.end_line);
    }

    #[test]
    fn test_search_result_with_zero_score() {
        let result = SearchResult {
            id: "zero-id".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            content: "use std::io;".to_string(),
            score: 0.0,
            metadata: serde_json::json!({}),
        };

        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_search_result_with_perfect_score() {
        let result = SearchResult {
            id: "perfect-id".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            content: "fn main() {}".to_string(),
            score: 1.0,
            metadata: serde_json::json!({}),
        };

        assert_eq!(result.score, 1.0);
    }
}
