//! Integration tests for entities and value objects working together

#[cfg(test)]
mod tests {
    use mcb_domain::{CodeChunk, Embedding, SearchResult, Language};

    #[test]
    fn test_code_chunk_with_embedding_integration() {
        // Test how CodeChunk entity works with Embedding value object
        let chunk = CodeChunk {
            id: "chunk-with-embedding".to_string(),
            content: "fn process_data(data: Vec<f32>) -> Vec<f32>".to_string(),
            file_path: "src/ml.rs".to_string(),
            start_line: 10,
            end_line: 15,
            language: "rust".to_string(),
            metadata: serde_json::json!({
                "type": "function",
                "purpose": "machine learning processing"
            }),
        };

        let embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            model: "text-embedding-3-small".to_string(),
            dimensions: 8,
        };

        // Integration: CodeChunk and Embedding can coexist and represent
        // different aspects of the same semantic unit
        assert_eq!(chunk.language, "rust");
        assert_eq!(embedding.model, "text-embedding-3-small");
        assert_eq!(chunk.content.lines().count(), 1); // Single line function signature
        assert_eq!(embedding.vector.len(), embedding.dimensions);
    }

    #[test]
    fn test_search_result_from_code_chunk() {
        // Test integration between CodeChunk and SearchResult
        let chunk = CodeChunk {
            id: "searchable-chunk".to_string(),
            content: "impl Database {\n    pub fn connect(&self) -> Result<Connection> {\n        // Connection logic\n    }\n}".to_string(),
            file_path: "src/database.rs".to_string(),
            start_line: 20,
            end_line: 25,
            language: "rust".to_string(),
            metadata: serde_json::json!({
                "type": "implementation",
                "class": "Database",
                "method": "connect"
            }),
        };

        // Create a SearchResult based on the CodeChunk
        let search_result = SearchResult {
            id: chunk.id.clone(),
            file_path: chunk.file_path.clone(),
            start_line: chunk.start_line,
            content: chunk.content.clone(),
            score: 0.95,
            language: chunk.language.clone(),
        };

        // Integration test: SearchResult preserves CodeChunk information
        assert_eq!(search_result.id, chunk.id);
        assert_eq!(search_result.file_path, chunk.file_path);
        assert_eq!(search_result.start_line, chunk.start_line);
        assert_eq!(search_result.content, chunk.content);
        assert_eq!(search_result.language, chunk.language);
        assert!(search_result.score > 0.9); // High relevance score
    }

    #[test]
    fn test_multi_language_code_chunks() {
        // Test integration with different programming languages
        let rust_chunk = CodeChunk {
            id: "rust-func".to_string(),
            content: "pub fn calculate_mean(data: &[f64]) -> f64 {\n    data.iter().sum::<f64>() / data.len() as f64\n}".to_string(),
            file_path: "src/stats.rs".to_string(),
            start_line: 5,
            end_line: 7,
            language: "rust".to_string(),
            metadata: serde_json::json!({"type": "function", "name": "calculate_mean"}),
        };

        let python_chunk = CodeChunk {
            id: "python-func".to_string(),
            content: "def calculate_mean(data: List[float]) -> float:\n    return sum(data) / len(data)".to_string(),
            file_path: "src/stats.py".to_string(),
            start_line: 8,
            end_line: 9,
            language: "python".to_string(),
            metadata: serde_json::json!({"type": "function", "name": "calculate_mean"}),
        };

        // Integration: Both chunks represent the same logical concept but in different languages
        assert_ne!(rust_chunk.language, python_chunk.language);
        assert_eq!(rust_chunk.language, "rust");
        assert_eq!(python_chunk.language, "python");

        // Both have similar metadata structure
        assert_eq!(rust_chunk.metadata["type"], "function");
        assert_eq!(python_chunk.metadata["type"], "function");
        assert_eq!(rust_chunk.metadata["name"], "calculate_mean");
        assert_eq!(python_chunk.metadata["name"], "calculate_mean");
    }

    #[test]
    fn test_embedding_vector_properties() {
        // Test integration of embedding properties
        let embedding = Embedding {
            vector: vec![
                0.123, -0.456, 0.789, 0.012, -0.345,
                0.678, -0.901, 0.234, -0.567, 0.890
            ],
            model: "text-embedding-ada-002".to_string(),
            dimensions: 10,
        };

        // Integration: Vector length matches dimensions
        assert_eq!(embedding.vector.len(), embedding.dimensions);

        // Test vector properties
        let has_positive = embedding.vector.iter().any(|&x| x > 0.0);
        let has_negative = embedding.vector.iter().any(|&x| x < 0.0);

        assert!(has_positive, "Embedding should have positive values");
        assert!(has_negative, "Embedding should have negative values");

        // Test that all values are reasonable (between -1 and 1 for normalized embeddings)
        for &value in &embedding.vector {
            assert!(value >= -1.0 && value <= 1.0,
                   "Embedding value {} is out of expected range [-1, 1]", value);
        }
    }

    #[test]
    fn test_search_result_ranking() {
        // Test integration of search results with different relevance scores
        let results = vec![
            SearchResult {
                id: "exact-match".to_string(),
                file_path: "src/perfect.rs".to_string(),
                start_line: 1,
                content: "fn exact_match_function() {}".to_string(),
                score: 1.0,
                language: "rust".to_string(),
            },
            SearchResult {
                id: "high-match".to_string(),
                file_path: "src/good.rs".to_string(),
                start_line: 5,
                content: "fn similar_function() {}".to_string(),
                score: 0.85,
                language: "rust".to_string(),
            },
            SearchResult {
                id: "medium-match".to_string(),
                file_path: "src/fair.rs".to_string(),
                start_line: 10,
                content: "fn somewhat_related() {}".to_string(),
                score: 0.65,
                language: "rust".to_string(),
            },
            SearchResult {
                id: "low-match".to_string(),
                file_path: "src/weak.rs".to_string(),
                start_line: 15,
                content: "fn barely_related() {}".to_string(),
                score: 0.25,
                language: "rust".to_string(),
            },
        ];

        // Integration: Results should be properly ranked by score
        for i in 0..results.len() - 1 {
            assert!(results[i].score >= results[i + 1].score,
                   "Results should be ordered by decreasing score: {} >= {}",
                   results[i].score, results[i + 1].score);
        }

        // Test score ranges
        assert_eq!(results[0].score, 1.0); // Perfect match
        assert!(results[1].score > 0.8);   // High match
        assert!(results[2].score > 0.6);   // Medium match
        assert!(results[3].score < 0.3);   // Low match
    }
}