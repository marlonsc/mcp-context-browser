//! Unit tests for CodeChunker domain service interface

#[cfg(test)]
mod tests {
    use mcb_domain::{CodeChunker, CodeChunk, Language};

    // Mock implementation for testing
    struct MockCodeChunker;

    #[async_trait::async_trait]
    impl CodeChunker for MockCodeChunker {
        async fn chunk_file(
            &self,
            _content: &str,
            _file_path: &str,
            _language: &Language,
        ) -> mcb_domain::Result<Vec<CodeChunk>> {
            Ok(vec![
                CodeChunk {
                    id: "chunk-1".to_string(),
                    content: "fn main() {}".to_string(),
                    file_path: "test.rs".to_string(),
                    start_line: 1,
                    end_line: 1,
                    language: "rust".to_string(),
                    metadata: serde_json::json!({"type": "function"}),
                }
            ])
        }

        fn supported_languages(&self) -> Vec<Language> {
            vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
            ]
        }

        fn is_language_supported(&self, language: &Language) -> bool {
            matches!(language.as_str(), "rust" | "python" | "javascript")
        }
    }

    #[test]
    fn test_code_chunker_interface() {
        let chunker = MockCodeChunker;

        // Test supported languages
        let languages = chunker.supported_languages();
        assert_eq!(languages.len(), 3);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
    }

    #[test]
    fn test_is_language_supported() {
        let chunker = MockCodeChunker;

        assert!(chunker.is_language_supported(&"rust".to_string()));
        assert!(chunker.is_language_supported(&"python".to_string()));
        assert!(chunker.is_language_supported(&"javascript".to_string()));

        assert!(!chunker.is_language_supported(&"go".to_string()));
        assert!(!chunker.is_language_supported(&"java".to_string()));
        assert!(!chunker.is_language_supported(&"unknown".to_string()));
    }

    #[tokio::test]
    async fn test_chunk_file() {
        let chunker = MockCodeChunker;

        let content = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let file_path = "src/main.rs";
        let language = "rust".to_string();

        let result = chunker.chunk_file(content, file_path, &language).await;
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);

        let chunk = &chunks[0];
        assert_eq!(chunk.id, "chunk-1");
        assert_eq!(chunk.content, "fn main() {}");
        assert_eq!(chunk.file_path, "test.rs"); // Mock returns different path
        assert_eq!(chunk.language, "rust");
    }

    #[test]
    fn test_code_chunker_trait_object() {
        // Test that we can use CodeChunker as a trait object
        let chunker: Box<dyn CodeChunker> = Box::new(MockCodeChunker);

        assert!(chunker.is_language_supported(&"rust".to_string()));
        assert!(!chunker.is_language_supported(&"haskell".to_string()));
    }
}