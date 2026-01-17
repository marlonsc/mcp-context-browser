//! Domain layer constants
//!
//! Contains constants that are part of the domain logic and are used by
//! the application layer. Infrastructure-specific constants remain in
//! `infrastructure/constants.rs`.

// ============================================================================
// INDEXING DOMAIN CONSTANTS
// ============================================================================

/// Default batch size for indexing operations
pub const INDEXING_BATCH_SIZE: usize = 10;

/// Minimum character length for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LENGTH: usize = 25;

/// Minimum number of lines for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LINES: usize = 2;

/// Maximum number of chunks extracted from a single file
pub const INDEXING_CHUNKS_MAX_PER_FILE: usize = 50;
