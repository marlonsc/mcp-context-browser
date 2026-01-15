//! Unit tests for SearchResult value object

#[cfg(test)]
mod tests {
    use mcb_domain::SearchResult;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: "chunk-123".to_string(),
            file_path: "src/search.rs".to_string(),
            start_line: 42,
            content: "impl SearchService for DefaultSearch { ... }".to_string(),
            score: 0.87,
            language: "rust".to_string(),
        };

        assert_eq!(result.id, "chunk-123");
        assert_eq!(result.file_path, "src/search.rs");
        assert_eq!(result.start_line, 42);
        assert_eq!(result.content, "impl SearchService for DefaultSearch { ... }");
        assert_eq!(result.score, 0.87);
        assert_eq!(result.language, "rust");
    }

    #[test]
    fn test_search_result_high_score() {
        let result = SearchResult {
            id: "perfect-match".to_string(),
            file_path: "src/perfect.rs".to_string(),
            start_line: 1,
            content: "fn search_perfect_match() {}".to_string(),
            score: 0.99,
            language: "rust".to_string(),
        };

        assert!(result.score > 0.95);
        assert_eq!(result.score, 0.99);
    }

    #[test]
    fn test_search_result_low_score() {
        let result = SearchResult {
            id: "poor-match".to_string(),
            file_path: "src/unrelated.rs".to_string(),
            start_line: 100,
            content: "fn unrelated_function() {}".to_string(),
            score: 0.12,
            language: "rust".to_string(),
        };

        assert!(result.score < 0.2);
        assert_eq!(result.score, 0.12);
    }

    #[test]
    fn test_search_result_different_languages() {
        let rust_result = SearchResult {
            id: "rust-chunk".to_string(),
            file_path: "src/lib.rs".to_string(),
            start_line: 10,
            content: "pub fn process_data(data: &str) -> Result<String> { ... }".to_string(),
            score: 0.85,
            language: "rust".to_string(),
        };

        let python_result = SearchResult {
            id: "python-chunk".to_string(),
            file_path: "src/utils.py".to_string(),
            start_line: 25,
            content: "def process_data(data: str) -> str:\n    return data.upper()".to_string(),
            score: 0.82,
            language: "python".to_string(),
        };

        assert_eq!(rust_result.language, "rust");
        assert_eq!(python_result.language, "python");
        assert!(rust_result.score > python_result.score);
    }

    #[test]
    fn test_search_result_zero_score() {
        let result = SearchResult {
            id: "no-match".to_string(),
            file_path: "src/irrelevant.rs".to_string(),
            start_line: 1,
            content: "unrelated content".to_string(),
            score: 0.0,
            language: "rust".to_string(),
        };

        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_search_result_perfect_score() {
        let result = SearchResult {
            id: "exact-match".to_string(),
            file_path: "src/exact.rs".to_string(),
            start_line: 1,
            content: "exact match content".to_string(),
            score: 1.0,
            language: "rust".to_string(),
        };

        assert_eq!(result.score, 1.0);
    }
}