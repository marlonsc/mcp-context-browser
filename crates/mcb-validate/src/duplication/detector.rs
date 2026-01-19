//! AST-Based Clone Detection
//!
//! Provides accurate clone detection using tree-sitter AST analysis.
//! Used to verify candidates from the fingerprinting phase and classify clone types.

use std::path::PathBuf;

use super::fingerprint::{FingerprintMatch, Token, TokenType};
use super::thresholds::{DuplicationThresholds, DuplicationType};

/// Result of comparing two code fragments
#[derive(Debug, Clone)]
pub struct CloneCandidate {
    /// File containing the first fragment
    pub file1: PathBuf,
    /// Starting line of first fragment (1-based)
    pub start_line1: usize,
    /// Ending line of first fragment (1-based)
    pub end_line1: usize,
    /// File containing the second fragment
    pub file2: PathBuf,
    /// Starting line of second fragment (1-based)
    pub start_line2: usize,
    /// Ending line of second fragment (1-based)
    pub end_line2: usize,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f64,
    /// Detected clone type
    pub clone_type: DuplicationType,
    /// Number of duplicated lines
    pub duplicated_lines: usize,
}

/// Clone detector using AST analysis
pub struct CloneDetector {
    thresholds: DuplicationThresholds,
}

impl CloneDetector {
    /// Create a new clone detector with the given thresholds
    pub fn new(thresholds: DuplicationThresholds) -> Self {
        Self { thresholds }
    }

    /// Verify fingerprint matches using AST comparison
    ///
    /// Takes candidates from the fingerprinting phase and verifies them
    /// using more accurate AST-based similarity comparison.
    pub fn verify_candidates(&self, matches: &[FingerprintMatch]) -> Vec<CloneCandidate> {
        let mut candidates = Vec::new();

        for m in matches {
            if let Some(candidate) = self.verify_single_match(m) {
                if self.passes_thresholds(&candidate) {
                    candidates.push(candidate);
                }
            }
        }

        // Deduplicate overlapping candidates
        self.deduplicate_candidates(candidates)
    }

    /// Verify a single fingerprint match
    fn verify_single_match(&self, m: &FingerprintMatch) -> Option<CloneCandidate> {
        let lines1 = m.location1.end_line.saturating_sub(m.location1.start_line) + 1;
        let lines2 = m.location2.end_line.saturating_sub(m.location2.start_line) + 1;
        let duplicated_lines = lines1.min(lines2);

        // For now, use a simplified similarity calculation based on token count
        // In a full implementation, this would use AST node comparison
        let similarity = self.calculate_similarity(m);
        let clone_type = self.classify_clone_type(similarity);

        if self.thresholds.should_detect(clone_type) {
            Some(CloneCandidate {
                file1: m.location1.file.clone(),
                start_line1: m.location1.start_line,
                end_line1: m.location1.end_line,
                file2: m.location2.file.clone(),
                start_line2: m.location2.start_line,
                end_line2: m.location2.end_line,
                similarity,
                clone_type,
                duplicated_lines,
            })
        } else {
            None
        }
    }

    /// Calculate similarity between two matched fragments
    ///
    /// In a full implementation, this would compare AST structures.
    /// For now, we use the fingerprint match as evidence of exact match.
    fn calculate_similarity(&self, _m: &FingerprintMatch) -> f64 {
        // Fingerprint matches are exact token sequence matches
        // so they have 100% similarity at the token level
        1.0
    }

    /// Classify the type of clone based on similarity
    fn classify_clone_type(&self, similarity: f64) -> DuplicationType {
        if similarity >= 1.0 {
            DuplicationType::ExactClone
        } else if similarity >= 0.95 {
            DuplicationType::RenamedClone
        } else if similarity >= 0.80 {
            DuplicationType::GappedClone
        } else {
            DuplicationType::SemanticClone
        }
    }

    /// Check if a candidate passes the configured thresholds
    fn passes_thresholds(&self, candidate: &CloneCandidate) -> bool {
        // Check minimum lines
        if candidate.duplicated_lines < self.thresholds.min_lines {
            return false;
        }

        // Check if we should detect this type
        if !self.thresholds.should_detect(candidate.clone_type) {
            return false;
        }

        // Check similarity threshold
        self.thresholds
            .meets_threshold(candidate.similarity, candidate.clone_type)
    }

    /// Remove overlapping candidates, keeping the best one
    fn deduplicate_candidates(&self, candidates: Vec<CloneCandidate>) -> Vec<CloneCandidate> {
        if candidates.is_empty() {
            return candidates;
        }

        let mut result = Vec::new();
        let mut used: Vec<bool> = vec![false; candidates.len()];

        // Sort by similarity (descending) to prefer higher similarity matches
        let mut sorted_indices: Vec<usize> = (0..candidates.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            candidates[b]
                .similarity
                .partial_cmp(&candidates[a].similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for &i in &sorted_indices {
            if used[i] {
                continue;
            }

            let candidate = &candidates[i];
            let mut is_duplicate = false;

            // Check if this overlaps with any already selected candidate
            for existing in &result {
                if self.candidates_overlap(candidate, existing) {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                result.push(candidate.clone());
                used[i] = true;

                // Mark overlapping candidates as used
                for (j, other) in candidates.iter().enumerate() {
                    if !used[j] && self.candidates_overlap(candidate, other) {
                        used[j] = true;
                    }
                }
            }
        }

        result
    }

    /// Check if two candidates overlap (same files and overlapping lines)
    fn candidates_overlap(&self, a: &CloneCandidate, b: &CloneCandidate) -> bool {
        // Check first location overlap
        let overlap1 = a.file1 == b.file1
            && Self::lines_overlap(a.start_line1, a.end_line1, b.start_line1, b.end_line1);

        // Check second location overlap
        let overlap2 = a.file2 == b.file2
            && Self::lines_overlap(a.start_line2, a.end_line2, b.start_line2, b.end_line2);

        // Also check cross-overlaps (a.file1 == b.file2, etc.)
        let cross1 = a.file1 == b.file2
            && Self::lines_overlap(a.start_line1, a.end_line1, b.start_line2, b.end_line2);

        let cross2 = a.file2 == b.file1
            && Self::lines_overlap(a.start_line2, a.end_line2, b.start_line1, b.end_line1);

        overlap1 || overlap2 || cross1 || cross2
    }

    /// Check if two line ranges overlap
    fn lines_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
        !(end1 < start2 || end2 < start1)
    }
}

/// Tokenize source code for fingerprinting
///
/// This is a simplified tokenizer. A full implementation would use
/// tree-sitter for language-aware tokenization.
pub fn tokenize_source(source: &str, _language: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_line = 1;
    let mut current_column = 1;
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                current_line += 1;
                current_column = 1;
            }
            c if c.is_whitespace() => {
                current_column += 1;
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = String::new();
                word.push(c);
                let start_column = current_column;

                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                let token_type = if is_keyword(&word) {
                    TokenType::Keyword
                } else {
                    TokenType::Identifier
                };

                tokens.push(Token::new(word.clone(), current_line, start_column, token_type));
                current_column += word.len();
            }
            c if c.is_ascii_digit() => {
                let mut number = String::new();
                number.push(c);
                let start_column = current_column;

                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' || next == '_' {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                tokens.push(Token::new(
                    number.clone(),
                    current_line,
                    start_column,
                    TokenType::Literal,
                ));
                current_column += number.len();
            }
            '"' | '\'' => {
                let quote = c;
                let mut string = String::new();
                string.push(c);
                let start_column = current_column;

                while let Some(next) = chars.next() {
                    string.push(next);
                    if next == quote && !string.ends_with("\\") {
                        break;
                    }
                    if next == '\n' {
                        current_line += 1;
                    }
                }

                tokens.push(Token::new(
                    string.clone(),
                    current_line,
                    start_column,
                    TokenType::Literal,
                ));
                current_column += string.len();
            }
            '/' => {
                if chars.peek() == Some(&'/') {
                    // Line comment
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        if next == '\n' {
                            break;
                        }
                        chars.next();
                    }
                } else if chars.peek() == Some(&'*') {
                    // Block comment
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\n' {
                            current_line += 1;
                        } else if next == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::new(
                        c.to_string(),
                        current_line,
                        current_column,
                        TokenType::Operator,
                    ));
                    current_column += 1;
                }
            }
            c if "+-*%=<>!&|^~".contains(c) => {
                tokens.push(Token::new(
                    c.to_string(),
                    current_line,
                    current_column,
                    TokenType::Operator,
                ));
                current_column += 1;
            }
            c if "(){}[];:,.?".contains(c) => {
                tokens.push(Token::new(
                    c.to_string(),
                    current_line,
                    current_column,
                    TokenType::Punctuation,
                ));
                current_column += 1;
            }
            _ => {
                current_column += 1;
            }
        }
    }

    tokens
}

/// Check if a word is a common keyword (simplified, multi-language)
fn is_keyword(word: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        // Rust
        "fn", "let", "mut", "const", "static", "struct", "enum", "impl", "trait", "pub", "mod",
        "use", "crate", "self", "super", "where", "async", "await", "move", "ref", "match", "if",
        "else", "loop", "while", "for", "in", "break", "continue", "return", "type", "as", "dyn",
        "unsafe", "extern", // Python
        "def", "class", "import", "from", "as", "with", "try", "except", "finally", "raise",
        "pass", "yield", "lambda", "global", "nonlocal", "assert", "del", "True", "False", "None",
        "and", "or", "not", "is", // JavaScript/TypeScript
        "function", "var", "const", "let", "class", "extends", "implements", "interface", "type",
        "enum", "namespace", "module", "export", "import", "default", "new", "delete", "typeof",
        "instanceof", "this", "super", "null", "undefined", "true", "false", "void", "throw",
        "try", "catch", "finally", "debugger", "switch", "case",
    ];

    KEYWORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_rust() {
        let source = "fn main() { let x = 42; }";
        let tokens = tokenize_source(source, "rust");

        let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(token_texts.contains(&"fn"));
        assert!(token_texts.contains(&"main"));
        assert!(token_texts.contains(&"let"));
        assert!(token_texts.contains(&"x"));
        assert!(token_texts.contains(&"42"));
    }

    #[test]
    fn test_tokenize_identifies_keywords() {
        let source = "fn foo let bar";
        let tokens = tokenize_source(source, "rust");

        let keywords: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Keyword)
            .collect();
        let identifiers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Identifier)
            .collect();

        assert_eq!(keywords.len(), 2); // fn, let
        assert_eq!(identifiers.len(), 2); // foo, bar
    }

    #[test]
    fn test_clone_detector_thresholds() {
        let thresholds = DuplicationThresholds {
            min_lines: 3,
            ..Default::default()
        };
        let detector = CloneDetector::new(thresholds);

        // Candidate with enough lines
        let good_candidate = CloneCandidate {
            file1: PathBuf::from("a.rs"),
            start_line1: 1,
            end_line1: 5,
            file2: PathBuf::from("b.rs"),
            start_line2: 10,
            end_line2: 14,
            similarity: 1.0,
            clone_type: DuplicationType::ExactClone,
            duplicated_lines: 5,
        };
        assert!(detector.passes_thresholds(&good_candidate));

        // Candidate with too few lines
        let bad_candidate = CloneCandidate {
            duplicated_lines: 2,
            ..good_candidate.clone()
        };
        assert!(!detector.passes_thresholds(&bad_candidate));
    }

    #[test]
    fn test_classify_clone_type() {
        let detector = CloneDetector::new(DuplicationThresholds::default());

        assert_eq!(
            detector.classify_clone_type(1.0),
            DuplicationType::ExactClone
        );
        assert_eq!(
            detector.classify_clone_type(0.97),
            DuplicationType::RenamedClone
        );
        assert_eq!(
            detector.classify_clone_type(0.85),
            DuplicationType::GappedClone
        );
        assert_eq!(
            detector.classify_clone_type(0.70),
            DuplicationType::SemanticClone
        );
    }

    #[test]
    fn test_lines_overlap() {
        assert!(CloneDetector::lines_overlap(1, 5, 3, 7)); // Overlapping
        assert!(CloneDetector::lines_overlap(1, 5, 5, 7)); // Adjacent (touching)
        assert!(!CloneDetector::lines_overlap(1, 5, 6, 10)); // Not overlapping
    }
}
