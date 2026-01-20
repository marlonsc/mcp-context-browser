//! Tests for AST analysis module

#[cfg(test)]
mod ast_tests {
    use mcb_validate::ast::AstEngine;
    use std::path::Path;

    #[test]
    fn test_ast_engine_creation() {
        let engine = AstEngine::new();
        assert!(!engine.supported_languages().is_empty());
    }

    #[test]
    fn test_language_detection() {
        let engine = AstEngine::new();

        assert_eq!(engine.detect_language(Path::new("main.rs")), Some("rust"));
        assert_eq!(
            engine.detect_language(Path::new("script.py")),
            Some("python")
        );
        assert_eq!(
            engine.detect_language(Path::new("app.js")),
            Some("javascript")
        );
        assert_eq!(
            engine.detect_language(Path::new("component.ts")),
            Some("typescript")
        );
        assert_eq!(engine.detect_language(Path::new("server.go")), Some("go"));
        assert_eq!(engine.detect_language(Path::new("unknown.xyz")), None);
    }
}
