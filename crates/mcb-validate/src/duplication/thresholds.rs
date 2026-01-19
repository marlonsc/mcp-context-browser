//! Duplication Detection Thresholds and Types
//!
//! Defines duplication types (clone categories) and configurable thresholds
//! for the duplication detection system.

use serde::{Deserialize, Serialize};

/// Clone type classification following established taxonomy
///
/// - Type 1 (Exact): Identical code fragments
/// - Type 2 (Renamed): Code with renamed identifiers
/// - Type 3 (Gapped): Near-miss clones with small modifications
/// - Type 4 (Semantic): Functionally equivalent code (future)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DuplicationType {
    /// Type 1: Exact copy-paste (100% identical)
    ExactClone,
    /// Type 2: Renamed identifiers only
    RenamedClone,
    /// Type 3: Near-miss with small modifications
    GappedClone,
    /// Type 4: Functionally similar (future implementation)
    SemanticClone,
}

impl DuplicationType {
    /// Get the rule ID prefix for this duplication type
    pub fn rule_id(&self) -> &'static str {
        match self {
            DuplicationType::ExactClone => "DUP001",
            DuplicationType::RenamedClone => "DUP002",
            DuplicationType::GappedClone => "DUP003",
            DuplicationType::SemanticClone => "DUP004",
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            DuplicationType::ExactClone => "Exact Clone",
            DuplicationType::RenamedClone => "Renamed Clone",
            DuplicationType::GappedClone => "Gapped Clone",
            DuplicationType::SemanticClone => "Semantic Clone",
        }
    }

    /// Get minimum similarity threshold for this type
    pub fn min_similarity(&self) -> f64 {
        match self {
            DuplicationType::ExactClone => 1.0,
            DuplicationType::RenamedClone => 0.95,
            DuplicationType::GappedClone => 0.80,
            DuplicationType::SemanticClone => 0.70,
        }
    }
}

impl std::fmt::Display for DuplicationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration thresholds for duplication detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationThresholds {
    /// Minimum number of lines for a clone to be reported
    pub min_lines: usize,
    /// Minimum number of tokens for a clone to be reported
    pub min_tokens: usize,
    /// Similarity threshold (0.0 - 1.0) for considering code as duplicate
    pub similarity_threshold: f64,
    /// Enable Type 1 (exact) clone detection
    pub detect_exact: bool,
    /// Enable Type 2 (renamed) clone detection
    pub detect_renamed: bool,
    /// Enable Type 3 (gapped) clone detection
    pub detect_gapped: bool,
    /// Enable Type 4 (semantic) clone detection (experimental)
    pub detect_semantic: bool,
    /// Languages to analyze
    pub languages: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum gap size for gapped clones (number of different tokens)
    pub max_gap_size: usize,
}

impl Default for DuplicationThresholds {
    fn default() -> Self {
        Self {
            min_lines: 6,
            min_tokens: 50,
            similarity_threshold: 0.80,
            detect_exact: true,
            detect_renamed: true,
            detect_gapped: true,
            detect_semantic: false, // Disabled by default (experimental)
            languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
            ],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/vendor/**".to_string(),
            ],
            max_gap_size: 5,
        }
    }
}

impl DuplicationThresholds {
    /// Create thresholds for strict detection (higher sensitivity)
    pub fn strict() -> Self {
        Self {
            min_lines: 4,
            min_tokens: 30,
            similarity_threshold: 0.90,
            ..Default::default()
        }
    }

    /// Create thresholds for lenient detection (lower sensitivity)
    pub fn lenient() -> Self {
        Self {
            min_lines: 10,
            min_tokens: 100,
            similarity_threshold: 0.70,
            ..Default::default()
        }
    }

    /// Check if a duplication type should be detected based on thresholds
    pub fn should_detect(&self, dup_type: DuplicationType) -> bool {
        match dup_type {
            DuplicationType::ExactClone => self.detect_exact,
            DuplicationType::RenamedClone => self.detect_renamed,
            DuplicationType::GappedClone => self.detect_gapped,
            DuplicationType::SemanticClone => self.detect_semantic,
        }
    }

    /// Check if a similarity value meets the threshold for a given type
    pub fn meets_threshold(&self, similarity: f64, dup_type: DuplicationType) -> bool {
        let type_min = dup_type.min_similarity();
        similarity >= self.similarity_threshold.max(type_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplication_type_rule_ids() {
        assert_eq!(DuplicationType::ExactClone.rule_id(), "DUP001");
        assert_eq!(DuplicationType::RenamedClone.rule_id(), "DUP002");
        assert_eq!(DuplicationType::GappedClone.rule_id(), "DUP003");
        assert_eq!(DuplicationType::SemanticClone.rule_id(), "DUP004");
    }

    #[test]
    fn test_default_thresholds() {
        let thresholds = DuplicationThresholds::default();
        assert_eq!(thresholds.min_lines, 6);
        assert_eq!(thresholds.min_tokens, 50);
        assert!(thresholds.detect_exact);
        assert!(thresholds.detect_renamed);
        assert!(thresholds.detect_gapped);
        assert!(!thresholds.detect_semantic);
    }

    #[test]
    fn test_strict_thresholds() {
        let thresholds = DuplicationThresholds::strict();
        assert_eq!(thresholds.min_lines, 4);
        assert_eq!(thresholds.similarity_threshold, 0.90);
    }

    #[test]
    fn test_meets_threshold() {
        let thresholds = DuplicationThresholds::default();

        // Exact clone requires 1.0 similarity
        assert!(thresholds.meets_threshold(1.0, DuplicationType::ExactClone));
        assert!(!thresholds.meets_threshold(0.99, DuplicationType::ExactClone));

        // Gapped clone with default 0.80 threshold
        assert!(thresholds.meets_threshold(0.85, DuplicationType::GappedClone));
        assert!(!thresholds.meets_threshold(0.75, DuplicationType::GappedClone));
    }

    #[test]
    fn test_should_detect() {
        let thresholds = DuplicationThresholds::default();
        assert!(thresholds.should_detect(DuplicationType::ExactClone));
        assert!(!thresholds.should_detect(DuplicationType::SemanticClone));
    }
}
