//! AST-based Unwrap Detector using rust-code-analysis
//!
//! Uses our fork with extended Node API for unwrap/expect detection.
//! Replaces the tree-sitter direct implementation with RCA Callback pattern.

use std::path::Path;

use rust_code_analysis::{Callback, Node, ParserTrait, action, guess_language};

use crate::{Result, ValidationError};

/// Detection result for unwrap/expect usage
#[derive(Debug, Clone)]
pub struct UnwrapDetection {
    /// File where the detection occurred
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// The specific method detected ("unwrap", "expect")
    pub method: String,
    /// Whether this is in a test module
    pub in_test: bool,
    /// The source text of the method call
    pub context: String,
}

/// Configuration for unwrap detection callback
struct UnwrapConfig {
    filename: String,
    test_ranges: Vec<(usize, usize)>,
}

/// RCA Callback for unwrap detection
struct UnwrapCallback;

impl Callback for UnwrapCallback {
    type Res = Vec<UnwrapDetection>;
    type Cfg = UnwrapConfig;

    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        let root = parser.get_root();
        let code = parser.get_code();
        let mut detections = Vec::new();

        // Recursive detection through AST
        detect_recursive(&root, code, &cfg, &mut detections);
        detections
    }
}

/// Recursively detect unwrap/expect calls in AST
fn detect_recursive(
    node: &Node,
    code: &[u8],
    cfg: &UnwrapConfig,
    results: &mut Vec<UnwrapDetection>,
) {
    // Check if this is a call_expression with unwrap/expect
    if node.kind() == "call_expression" {
        if let Some(text) = node.utf8_text(code) {
            let method = extract_method(text);
            if matches!(method.as_str(), "unwrap" | "expect") {
                let byte_pos = node.start_byte();
                let in_test = cfg
                    .test_ranges
                    .iter()
                    .any(|(start, end)| byte_pos >= *start && byte_pos < *end);

                results.push(UnwrapDetection {
                    file: cfg.filename.clone(),
                    line: node.start_row() + 1,
                    column: node.start_position().1 + 1,
                    method,
                    in_test,
                    context: text.lines().next().unwrap_or("").trim().to_string(),
                });
            }
        }
    }

    // Recurse through children via inner tree-sitter node (public in our fork)
    let mut cursor = node.0.walk();
    for child in node.0.children(&mut cursor) {
        let child_node = Node(child);
        detect_recursive(&child_node, code, cfg, results);
    }
}

/// Extract method name from call expression text
fn extract_method(text: &str) -> String {
    if text.contains(".unwrap()") {
        "unwrap".to_string()
    } else if text.contains(".expect(") {
        "expect".to_string()
    } else {
        String::new()
    }
}

/// Find test module ranges in the AST
fn find_test_ranges(root: &Node, code: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    find_test_modules_recursive(root, code, &mut ranges);
    ranges
}

fn find_test_modules_recursive(node: &Node, code: &[u8], ranges: &mut Vec<(usize, usize)>) {
    if node.kind() == "mod_item" {
        // Check for #[cfg(test)] attribute before this node
        let start = node.start_byte();
        // Look back up to 50 bytes (or start of file) for the attribute
        let search_start = start.saturating_sub(50);
        let before = std::str::from_utf8(&code[search_start..start]).unwrap_or("");
        if before.contains("#[cfg(test)]") {
            ranges.push((node.start_byte(), node.end_byte()));
            return; // Don't recurse into test modules
        }

        // Also check if the module name is "tests" (common pattern)
        if let Some(name_text) = node.utf8_text(code) {
            if name_text.contains("mod tests") || name_text.contains("mod test") {
                // Double check there's a #[cfg(test)] somewhere before it in the file
                let all_before = std::str::from_utf8(&code[..start]).unwrap_or("");
                // Find the last occurrence of #[cfg(test)] before this position
                if let Some(attr_pos) = all_before.rfind("#[cfg(test)]") {
                    // Make sure there's no other mod_item between the attribute and this module
                    let between = &all_before[attr_pos..];
                    if !between.contains("mod ")
                        || between.rfind("mod ").unwrap_or(0) == between.len() - name_text.len()
                    {
                        ranges.push((node.start_byte(), node.end_byte()));
                        return;
                    }
                }
            }
        }
    }

    // Recurse through children
    let mut cursor = node.0.walk();
    for child in node.0.children(&mut cursor) {
        find_test_modules_recursive(&Node(child), code, ranges);
    }
}

/// RCA Callback for finding test module ranges
struct TestRangeCallback;

impl Callback for TestRangeCallback {
    type Res = Vec<(usize, usize)>;
    type Cfg = ();

    fn call<T: ParserTrait>(_cfg: (), parser: &T) -> Self::Res {
        find_test_ranges(&parser.get_root(), parser.get_code())
    }
}

/// Detect unwrap/expect in file content
pub fn detect_in_content(content: &str, filename: &str) -> Result<Vec<UnwrapDetection>> {
    let path = Path::new(filename);
    let source = content.as_bytes().to_vec();

    let (lang, _) = guess_language(&source, path);
    let lang = lang.ok_or_else(|| {
        ValidationError::Config(format!("Unsupported language for file: {}", filename))
    })?;

    // First pass: find test module ranges
    let test_ranges = action::<TestRangeCallback>(&lang, source.clone(), path, None, ());

    // Second pass: detect unwraps
    let cfg = UnwrapConfig {
        filename: filename.to_string(),
        test_ranges,
    };

    Ok(action::<UnwrapCallback>(&lang, source, path, None, cfg))
}

/// Detect unwrap/expect in file
pub fn detect_in_file(path: &Path) -> Result<Vec<UnwrapDetection>> {
    let content = std::fs::read_to_string(path)?;
    detect_in_content(&content, &path.to_string_lossy())
}

/// AST-based unwrap detector using rust-code-analysis
///
/// Provides the same API as before but uses RCA internally.
pub struct UnwrapDetector;

impl UnwrapDetector {
    /// Create a new unwrap detector
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Detect unwrap/expect calls in Rust source code
    pub fn detect_in_content(
        &mut self,
        content: &str,
        filename: &str,
    ) -> Result<Vec<UnwrapDetection>> {
        detect_in_content(content, filename)
    }

    /// Detect unwrap/expect calls in a file
    pub fn detect_in_file(&mut self, path: &Path) -> Result<Vec<UnwrapDetection>> {
        detect_in_file(path)
    }
}

impl Default for UnwrapDetector {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_unwrap_in_code() {
        let code = r#"
fn main() {
    let x = Some(42);
    let y = x.unwrap();
}
"#;
        let detections = detect_in_content(code, "test.rs").unwrap();
        assert!(!detections.is_empty());
        assert_eq!(detections[0].method, "unwrap");
        assert!(!detections[0].in_test);
    }

    #[test]
    fn test_detect_expect_in_code() {
        let code = r#"
fn main() {
    let x = Some(42);
    let y = x.expect("should have value");
}
"#;
        let detections = detect_in_content(code, "test.rs").unwrap();
        assert!(!detections.is_empty());
        assert_eq!(detections[0].method, "expect");
    }

    #[test]
    fn test_detect_in_test_module() {
        let code = r#"
fn main() {
    let x = Some(42);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let y = Some(1).unwrap();
    }
}
"#;
        let detections = detect_in_content(code, "test.rs").unwrap();
        // Should find the unwrap in test module
        let test_detections: Vec<_> = detections.iter().filter(|d| d.in_test).collect();
        assert!(!test_detections.is_empty());
    }

    #[test]
    fn test_no_false_positives() {
        let code = r#"
fn main() {
    let x = "unwrap is just a word here";
    let y = Some(42)?;
}
"#;
        let detections = detect_in_content(code, "test.rs").unwrap();
        // Should not detect "unwrap" in string literal as a method call
        let method_calls: Vec<_> = detections
            .iter()
            .filter(|d| d.method == "unwrap" || d.method == "expect")
            .collect();
        assert!(method_calls.is_empty());
    }

    #[test]
    fn test_detector_struct_api() {
        let mut detector = UnwrapDetector::new().unwrap();
        let code = "fn f() { Some(1).unwrap(); }";
        let detections = detector.detect_in_content(code, "test.rs").unwrap();
        assert!(!detections.is_empty());
    }
}
