//! Language-specific AST parsers using Tree-sitter
//!
//! Provides parsers for different programming languages that convert
//! Tree-sitter AST to unified format.

use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

use super::{AstParseResult, AstParser};
use crate::Result;

/// Rust AST parser using tree-sitter-rust
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::language();
        parser.set_language(&language).expect("Failed to load Rust grammar");

        Self { parser }
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for RustParser {
    fn language(&self) -> &'static str {
        "rust"
    }

    fn parse_file(&self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| crate::ValidationError::Parse {
                file: filename.into(),
                message: "Failed to parse Rust code".into(),
            })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(), // Tree-sitter doesn't provide detailed errors
        })
    }
}

/// Python AST parser using tree-sitter-python
pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_python::language();
        parser.set_language(&language).expect("Failed to load Python grammar");

        Self { parser }
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for PythonParser {
    fn language(&self) -> &'static str {
        "python"
    }

    fn parse_file(&self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| crate::ValidationError::Parse {
                file: filename.into(),
                message: "Failed to parse Python code".into(),
            })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(),
        })
    }
}

/// JavaScript AST parser using tree-sitter-javascript
pub struct JavaScriptParser {
    parser: Parser,
}

impl JavaScriptParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_javascript::language();
        parser.set_language(&language).expect("Failed to load JavaScript grammar");

        Self { parser }
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for JavaScriptParser {
    fn language(&self) -> &'static str {
        "javascript"
    }

    fn parse_file(&self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| crate::ValidationError::Parse {
                file: filename.into(),
                message: "Failed to parse JavaScript code".into(),
            })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(),
        })
    }
}

/// TypeScript AST parser using tree-sitter-typescript
pub struct TypeScriptParser {
    parser: Parser,
}

impl TypeScriptParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::language_tsx();
        parser.set_language(&language).expect("Failed to load TypeScript grammar");

        Self { parser }
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for TypeScriptParser {
    fn language(&self) -> &'static str {
        "typescript"
    }

    fn parse_file(&self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| crate::ValidationError::Parse {
                file: filename.into(),
                message: "Failed to parse TypeScript code".into(),
            })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(),
        })
    }
}

/// Go AST parser using tree-sitter-go
pub struct GoParser {
    parser: Parser,
}

impl GoParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_go::language();
        parser.set_language(&language).expect("Failed to load Go grammar");

        Self { parser }
    }
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for GoParser {
    fn language(&self) -> &'static str {
        "go"
    }

    fn parse_file(&self, path: &Path) -> Result<AstParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content, &path.to_string_lossy())
    }

    fn parse_content(&self, content: &str, filename: &str) -> Result<AstParseResult> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| crate::ValidationError::Parse {
                file: filename.into(),
                message: "Failed to parse Go code".into(),
            })?;

        let root = super::decoder::AstDecoder::decode_tree(&tree, content);

        Ok(AstParseResult {
            root,
            errors: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rust_parser_creation() {
        let parser = RustParser::new();
        assert_eq!(parser.language(), "rust");
    }

    #[test]
    fn test_python_parser_creation() {
        let parser = PythonParser::new();
        assert_eq!(parser.language(), "python");
    }

    #[test]
    fn test_javascript_parser_creation() {
        let parser = JavaScriptParser::new();
        assert_eq!(parser.language(), "javascript");
    }

    #[test]
    fn test_typescript_parser_creation() {
        let parser = TypeScriptParser::new();
        assert_eq!(parser.language(), "typescript");
    }

    #[test]
    fn test_go_parser_creation() {
        let parser = GoParser::new();
        assert_eq!(parser.language(), "go");
    }

    #[test]
    fn test_parse_simple_rust_function() {
        let parser = RustParser::new();
        let code = r#"
fn hello_world() {
    println!("Hello, World!");
}
"#;

        let result = parser.parse_content(code, "test.rs").unwrap();
        assert_eq!(result.root.kind, "source_file");
        assert!(!result.root.children.is_empty());
    }

    #[test]
    fn test_parse_simple_python_function() {
        let parser = PythonParser::new();
        let code = r#"
def hello_world():
    print("Hello, World!")
"#;

        let result = parser.parse_content(code, "test.py").unwrap();
        assert_eq!(result.root.kind, "module");
        assert!(!result.root.children.is_empty());
    }
}