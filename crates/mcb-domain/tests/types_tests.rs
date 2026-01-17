//! Unit tests for Domain Type Aliases

#[cfg(test)]
mod tests {
    use mcb_domain::{EmbeddingProviderKind, Language, OperationType, VectorStoreProviderKind};

    #[test]
    fn test_language_type_alias() {
        let lang: Language = "rust".to_string();
        assert_eq!(lang, "rust");

        let python: Language = "python".to_string();
        assert_eq!(python, "python");

        let custom_lang: Language = "kotlinscript".to_string();
        assert_eq!(custom_lang, "kotlinscript");
    }

    #[test]
    fn test_operation_type_alias() {
        let op: OperationType = "indexing".to_string();
        assert_eq!(op, "indexing");

        let search: OperationType = "search".to_string();
        assert_eq!(search, "search");

        let custom_op: OperationType = "custom-operation".to_string();
        assert_eq!(custom_op, "custom-operation");
    }

    #[test]
    fn test_embedding_provider_kind_alias() {
        let provider: EmbeddingProviderKind = "openai".to_string();
        assert_eq!(provider, "openai");

        let ollama: EmbeddingProviderKind = "ollama".to_string();
        assert_eq!(ollama, "ollama");

        let custom: EmbeddingProviderKind = "my-custom-embedder".to_string();
        assert_eq!(custom, "my-custom-embedder");
    }

    #[test]
    fn test_vector_store_provider_kind_alias() {
        let store: VectorStoreProviderKind = "qdrant".to_string();
        assert_eq!(store, "qdrant");

        let filesystem: VectorStoreProviderKind = "filesystem".to_string();
        assert_eq!(filesystem, "filesystem");

        let custom: VectorStoreProviderKind = "my-vector-db".to_string();
        assert_eq!(custom, "my-vector-db");
    }

    #[test]
    fn test_type_alias_equality() {
        let lang1: Language = "javascript".to_string();
        let lang2: Language = "javascript".to_string();
        assert_eq!(lang1, lang2);

        let op1: OperationType = "embedding".to_string();
        let op2: OperationType = "embedding".to_string();
        assert_eq!(op1, op2);
    }

    #[test]
    fn test_type_alias_inequality() {
        let lang1: Language = "rust".to_string();
        let lang2: Language = "python".to_string();
        assert_ne!(lang1, lang2);

        let provider1: EmbeddingProviderKind = "openai".to_string();
        let provider2: EmbeddingProviderKind = "anthropic".to_string();
        assert_ne!(provider1, provider2);
    }
}
