//! Unit tests for CodeChunk entity
//!
//! Tests the core domain entity for code chunks, ensuring proper
//! creation, validation, and business rule enforcement.

#[cfg(test)]
mod tests {
    use mcb_domain::CodeChunk;

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk {
            id: "test-chunk-001".to_string(),
            content: "fn hello() { println!(\"Hello!\"); }".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: "rust".to_string(),
            metadata: serde_json::json!({
                "type": "function",
                "name": "hello"
            }),
        };

        assert_eq!(chunk.id, "test-chunk-001");
        assert_eq!(chunk.content, "fn hello() { println!(\"Hello!\"); }");
        assert_eq!(chunk.file_path, "src/main.rs");
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 3);
        assert_eq!(chunk.language, "rust");
        assert_eq!(chunk.metadata["type"], "function");
        assert_eq!(chunk.metadata["name"], "hello");
    }

    #[test]
    fn test_code_chunk_with_empty_metadata() {
        let chunk = CodeChunk {
            id: "test-chunk-002".to_string(),
            content: "print('hello')".to_string(),
            file_path: "script.py".to_string(),
            start_line: 1,
            end_line: 1,
            language: "python".to_string(),
            metadata: serde_json::json!({}),
        };

        assert_eq!(chunk.id, "test-chunk-002");
        assert_eq!(chunk.language, "python");
        assert!(chunk.metadata.is_object());
        assert!(chunk.metadata.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_code_chunk_with_complex_metadata() {
        let chunk = CodeChunk {
            id: "complex-chunk".to_string(),
            content: "class User {\n  constructor(name) {\n    this.name = name;\n  }\n}"
                .to_string(),
            file_path: "src/User.js".to_string(),
            start_line: 1,
            end_line: 5,
            language: "javascript".to_string(),
            metadata: serde_json::json!({
                "type": "class",
                "name": "User",
                "methods": ["constructor"],
                "properties": ["name"],
                "complexity": 2
            }),
        };

        assert_eq!(chunk.id, "complex-chunk");
        assert_eq!(chunk.language, "javascript");
        assert_eq!(chunk.metadata["type"], "class");
        assert_eq!(chunk.metadata["name"], "User");
        assert_eq!(chunk.metadata["complexity"], 2);
        assert!(chunk.metadata["methods"].is_array());
    }
}
