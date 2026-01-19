//! Token Fingerprinting using Rabin-Karp Rolling Hash
//!
//! Provides fast initial duplication detection using a rolling hash algorithm.
//! This serves as a first-pass filter before more expensive AST similarity analysis.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// A fingerprint represents a hash of a code fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fingerprint(u64);

impl Fingerprint {
    /// Create a new fingerprint from a hash value
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }

    /// Get the raw hash value
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Location of a fingerprinted code fragment
#[derive(Debug, Clone)]
pub struct FingerprintLocation {
    /// File path
    pub file: PathBuf,
    /// Starting line number (1-based)
    pub start_line: usize,
    /// Ending line number (1-based)
    pub end_line: usize,
    /// Number of tokens in the fragment
    pub token_count: usize,
}

/// A match between two fingerprinted locations
#[derive(Debug, Clone)]
pub struct FingerprintMatch {
    /// First location
    pub location1: FingerprintLocation,
    /// Second location (the duplicate)
    pub location2: FingerprintLocation,
    /// The fingerprint value
    pub fingerprint: Fingerprint,
}

/// Token fingerprinter using Rabin-Karp rolling hash
///
/// Uses a sliding window approach to generate fingerprints for code fragments.
/// Fingerprints are stored in a hash map for O(1) duplicate lookup.
pub struct TokenFingerprinter {
    /// Window size in tokens
    window_size: usize,
    /// Base for the polynomial hash
    base: u64,
    /// Modulus for the hash (large prime)
    modulus: u64,
    /// Precomputed base^(window_size-1) mod modulus
    base_power: u64,
    /// Map from fingerprint to locations
    fingerprint_map: HashMap<Fingerprint, Vec<FingerprintLocation>>,
}

impl TokenFingerprinter {
    /// Create a new fingerprinter with the given window size
    pub fn new(window_size: usize) -> Self {
        let base: u64 = 31;
        let modulus: u64 = 1_000_000_007; // Large prime

        // Precompute base^(window_size-1) mod modulus
        let base_power = Self::mod_pow(base, window_size.saturating_sub(1) as u64, modulus);

        Self {
            window_size,
            base,
            modulus,
            base_power,
            fingerprint_map: HashMap::new(),
        }
    }

    /// Modular exponentiation: base^exp mod modulus
    fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        let mut result: u64 = 1;
        base %= modulus;
        while exp > 0 {
            if exp % 2 == 1 {
                result = result.wrapping_mul(base) % modulus;
            }
            exp /= 2;
            base = base.wrapping_mul(base) % modulus;
        }
        result
    }

    /// Convert a token to a numeric value for hashing
    fn token_value(token: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        token.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute the initial hash for a window of tokens
    fn initial_hash(&self, tokens: &[&str]) -> u64 {
        let mut hash: u64 = 0;
        for token in tokens.iter().take(self.window_size) {
            let val = Self::token_value(token) % self.modulus;
            hash = (hash.wrapping_mul(self.base) + val) % self.modulus;
        }
        hash
    }

    /// Compute the rolling hash by removing old token and adding new token
    fn rolling_hash(&self, current_hash: u64, old_token: &str, new_token: &str) -> u64 {
        let old_val = Self::token_value(old_token) % self.modulus;
        let new_val = Self::token_value(new_token) % self.modulus;

        // Remove old token contribution: hash - old_val * base^(n-1)
        let mut hash = current_hash;
        let old_contribution = old_val.wrapping_mul(self.base_power) % self.modulus;
        hash = (hash + self.modulus - old_contribution) % self.modulus;

        // Shift and add new token: hash * base + new_val
        hash = (hash.wrapping_mul(self.base) + new_val) % self.modulus;
        hash
    }

    /// Fingerprint a file's tokens and add to the map
    pub fn fingerprint_file(&mut self, file: PathBuf, tokens: &[Token]) {
        if tokens.len() < self.window_size {
            return;
        }

        let token_strs: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();

        // Compute initial hash
        let mut hash = self.initial_hash(&token_strs[..self.window_size]);

        // Store first fingerprint
        let fingerprint = Fingerprint::new(hash);
        let location = FingerprintLocation {
            file: file.clone(),
            start_line: tokens[0].line,
            end_line: tokens[self.window_size - 1].line,
            token_count: self.window_size,
        };
        self.fingerprint_map
            .entry(fingerprint)
            .or_insert_with(Vec::new)
            .push(location);

        // Roll through remaining tokens
        for i in 1..=(tokens.len() - self.window_size) {
            hash = self.rolling_hash(
                hash,
                &token_strs[i - 1],
                &token_strs[i + self.window_size - 1],
            );

            let fingerprint = Fingerprint::new(hash);
            let location = FingerprintLocation {
                file: file.clone(),
                start_line: tokens[i].line,
                end_line: tokens[i + self.window_size - 1].line,
                token_count: self.window_size,
            };
            self.fingerprint_map
                .entry(fingerprint)
                .or_insert_with(Vec::new)
                .push(location);
        }
    }

    /// Find all duplicate fingerprints
    pub fn find_duplicates(&self) -> Vec<FingerprintMatch> {
        let mut matches = Vec::new();

        for (fingerprint, locations) in &self.fingerprint_map {
            if locations.len() < 2 {
                continue;
            }

            // Generate pairs of duplicates
            for i in 0..locations.len() {
                for j in (i + 1)..locations.len() {
                    let loc1 = &locations[i];
                    let loc2 = &locations[j];

                    // Skip if in the same file and overlapping
                    if loc1.file == loc2.file {
                        let overlap = Self::lines_overlap(
                            loc1.start_line,
                            loc1.end_line,
                            loc2.start_line,
                            loc2.end_line,
                        );
                        if overlap {
                            continue;
                        }
                    }

                    matches.push(FingerprintMatch {
                        location1: loc1.clone(),
                        location2: loc2.clone(),
                        fingerprint: *fingerprint,
                    });
                }
            }
        }

        matches
    }

    /// Check if two line ranges overlap
    fn lines_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
        !(end1 < start2 || end2 < start1)
    }

    /// Clear all stored fingerprints
    pub fn clear(&mut self) {
        self.fingerprint_map.clear();
    }

    /// Get statistics about stored fingerprints
    pub fn stats(&self) -> FingerprintStats {
        let total_fingerprints = self.fingerprint_map.len();
        let total_locations: usize = self.fingerprint_map.values().map(|v| v.len()).sum();
        let duplicates: usize = self
            .fingerprint_map
            .values()
            .filter(|v| v.len() > 1)
            .count();

        FingerprintStats {
            total_fingerprints,
            total_locations,
            unique_fingerprints: total_fingerprints - duplicates,
            duplicate_fingerprints: duplicates,
        }
    }
}

/// A simple token representation
#[derive(Debug, Clone)]
pub struct Token {
    /// Token text
    pub text: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Token type (for normalized comparison)
    pub token_type: TokenType,
}

/// Token type for normalization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Identifier (variable, function, type name)
    Identifier,
    /// Keyword (if, for, while, etc.)
    Keyword,
    /// Literal (string, number, etc.)
    Literal,
    /// Operator (+, -, *, etc.)
    Operator,
    /// Punctuation ({, }, (, ), etc.)
    Punctuation,
    /// Comment
    Comment,
    /// Whitespace
    Whitespace,
    /// Other/unknown
    Other,
}

impl Token {
    /// Create a new token
    pub fn new(text: String, line: usize, column: usize, token_type: TokenType) -> Self {
        Self {
            text,
            line,
            column,
            token_type,
        }
    }

    /// Normalize the token for Type 2 (renamed) clone detection
    ///
    /// Replaces identifiers with a placeholder while keeping structure
    pub fn normalized_text(&self) -> &str {
        match self.token_type {
            TokenType::Identifier => "$ID",
            TokenType::Literal => "$LIT",
            _ => &self.text,
        }
    }
}

/// Statistics about fingerprinting
#[derive(Debug, Clone)]
pub struct FingerprintStats {
    /// Total number of unique fingerprints
    pub total_fingerprints: usize,
    /// Total number of locations fingerprinted
    pub total_locations: usize,
    /// Number of fingerprints with only one location
    pub unique_fingerprints: usize,
    /// Number of fingerprints with multiple locations (potential duplicates)
    pub duplicate_fingerprints: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tokens(texts: &[&str]) -> Vec<Token> {
        texts
            .iter()
            .enumerate()
            .map(|(i, text)| Token::new(text.to_string(), i + 1, 1, TokenType::Identifier))
            .collect()
    }

    #[test]
    fn test_fingerprint_simple() {
        let mut fp = TokenFingerprinter::new(3);

        let tokens1 = make_tokens(&["fn", "foo", "(", ")", "{", "}"]);
        let tokens2 = make_tokens(&["fn", "foo", "(", ")", "{", "}"]);

        fp.fingerprint_file(PathBuf::from("file1.rs"), &tokens1);
        fp.fingerprint_file(PathBuf::from("file2.rs"), &tokens2);

        let matches = fp.find_duplicates();
        assert!(!matches.is_empty(), "Should find duplicates");
    }

    #[test]
    fn test_no_overlap_same_file() {
        let mut fp = TokenFingerprinter::new(3);

        // Same file, same tokens - should not report overlapping matches
        let tokens = make_tokens(&["a", "b", "c", "d", "e"]);
        fp.fingerprint_file(PathBuf::from("file1.rs"), &tokens);

        let matches = fp.find_duplicates();
        // All fingerprints in the same file will be from different positions
        // But overlapping ones should be filtered out
        for m in &matches {
            if m.location1.file == m.location2.file {
                // They should not overlap
                assert!(
                    m.location1.end_line < m.location2.start_line
                        || m.location2.end_line < m.location1.start_line,
                    "Found overlapping match in same file"
                );
            }
        }
    }

    #[test]
    fn test_rolling_hash_consistency() {
        let fp = TokenFingerprinter::new(3);

        let tokens = ["a", "b", "c", "d"];

        // Compute hash for "a", "b", "c"
        let hash1 = fp.initial_hash(&tokens[0..3]);

        // Compute hash for "b", "c", "d" using rolling
        let hash2_rolling = fp.rolling_hash(hash1, "a", "d");

        // Compute hash for "b", "c", "d" directly
        let hash2_direct = fp.initial_hash(&tokens[1..4]);

        assert_eq!(
            hash2_rolling, hash2_direct,
            "Rolling hash should match direct hash"
        );
    }

    #[test]
    fn test_fingerprint_stats() {
        let mut fp = TokenFingerprinter::new(2);

        let tokens1 = make_tokens(&["a", "b", "c"]);
        let tokens2 = make_tokens(&["a", "b", "d"]); // Shares "a", "b"

        fp.fingerprint_file(PathBuf::from("file1.rs"), &tokens1);
        fp.fingerprint_file(PathBuf::from("file2.rs"), &tokens2);

        let stats = fp.stats();
        assert!(stats.duplicate_fingerprints >= 1, "Should have at least one duplicate fingerprint");
    }

    #[test]
    fn test_token_normalization() {
        let id_token = Token::new("myVariable".to_string(), 1, 1, TokenType::Identifier);
        let kw_token = Token::new("if".to_string(), 1, 1, TokenType::Keyword);
        let lit_token = Token::new("42".to_string(), 1, 1, TokenType::Literal);

        assert_eq!(id_token.normalized_text(), "$ID");
        assert_eq!(kw_token.normalized_text(), "if");
        assert_eq!(lit_token.normalized_text(), "$LIT");
    }
}
