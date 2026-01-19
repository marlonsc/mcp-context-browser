//! Tests for expression engine

#[cfg(test)]
mod expression_engine_tests {
    use mcb_validate::ValidationConfig;
    use mcb_validate::engines::RuleContext;
    use mcb_validate::engines::expression_engine::ExpressionEngine;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_context() -> RuleContext {
        let mut file_contents = HashMap::new();
        file_contents.insert(
            "src/main.rs".to_string(),
            "fn main() { println!(\"hello\"); }".to_string(),
        );
        file_contents.insert(
            "src/lib.rs".to_string(),
            "pub fn helper() -> Result<(), Error> { Ok(()) }".to_string(),
        );

        RuleContext {
            workspace_root: PathBuf::from("/test/workspace"),
            config: ValidationConfig::new("/test/workspace"),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents,
        }
    }

    #[test]
    fn test_simple_expression() {
        let engine = ExpressionEngine::new();
        let context = create_test_context();

        // file_count should be 2
        let result = engine.evaluate_expression("file_count == 2", &context);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // file_count > 10 should be false
        let result = engine.evaluate_expression("file_count > 10", &context);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_boolean_expression() {
        let engine = ExpressionEngine::new();
        let context = create_test_context();

        // No unwrap in test context
        let result = engine.evaluate_expression("has_unwrap == false", &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_custom_variables() {
        let engine = ExpressionEngine::new();
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), serde_json::json!(10));
        variables.insert("y".to_string(), serde_json::json!(5));

        let result = engine.evaluate_with_variables("x > y", &variables);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = engine.evaluate_with_variables("x + y == 15", &variables);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_invalid_expression() {
        let engine = ExpressionEngine::new();
        let context = create_test_context();

        let result = engine.evaluate_expression("undefined_var > 0", &context);
        assert!(result.is_err());
    }
}
