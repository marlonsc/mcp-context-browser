//! Unit tests for domain constants

use mcb_domain::{
    INDEXING_BATCH_SIZE, INDEXING_CHUNKS_MAX_PER_FILE, INDEXING_CHUNK_MIN_LENGTH,
    INDEXING_CHUNK_MIN_LINES,
};

#[test]
fn test_indexing_constants() {
    assert_eq!(INDEXING_BATCH_SIZE, 10);
    assert_eq!(INDEXING_CHUNK_MIN_LENGTH, 25);
    assert_eq!(INDEXING_CHUNK_MIN_LINES, 2);
    assert_eq!(INDEXING_CHUNKS_MAX_PER_FILE, 50);
}

#[test]
fn test_indexing_constants_relationships() {
    // Test that constants have reasonable relationships
    assert!(INDEXING_CHUNK_MIN_LENGTH > 0);
    assert!(INDEXING_CHUNK_MIN_LINES > 0);
    assert!(INDEXING_BATCH_SIZE > 0);
    assert!(INDEXING_CHUNKS_MAX_PER_FILE > 0);

    // Test that batch size is reasonable
    assert!(INDEXING_BATCH_SIZE >= 1);
    assert!(INDEXING_BATCH_SIZE <= 100);

    // Test that chunk limits are reasonable
    assert!(INDEXING_CHUNK_MIN_LENGTH >= 10);
    assert!(INDEXING_CHUNK_MIN_LINES >= 1);
    assert!(INDEXING_CHUNKS_MAX_PER_FILE >= 10);
    assert!(INDEXING_CHUNKS_MAX_PER_FILE <= 200);
}

#[test]
fn test_constants_are_compile_time() {
    // These are compile-time constants, so they should be accessible
    // without any runtime computation
    let _batch_size = INDEXING_BATCH_SIZE;
    let _min_length = INDEXING_CHUNK_MIN_LENGTH;
    let _min_lines = INDEXING_CHUNK_MIN_LINES;
    let _max_chunks = INDEXING_CHUNKS_MAX_PER_FILE;

    // Just verify they can be used in expressions
    assert!(INDEXING_BATCH_SIZE * 2 == 20);
    assert!(INDEXING_CHUNK_MIN_LENGTH + 5 == 30);
}
