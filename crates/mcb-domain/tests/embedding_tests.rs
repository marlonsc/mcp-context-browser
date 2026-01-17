//! Unit tests for Embedding value object

use mcb_domain::Embedding;

#[test]
fn test_embedding_creation() {
    let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let embedding = Embedding {
        vector: vector.clone(),
        model: "text-embedding-ada-002".to_string(),
        dimensions: 5,
    };

    assert_eq!(embedding.vector, vector);
    assert_eq!(embedding.model, "text-embedding-ada-002");
    assert_eq!(embedding.dimensions, 5);
}

#[test]
fn test_embedding_with_realistic_data() {
    // Simulate a real embedding vector (truncated for test)
    let vector = vec![
        0.123456, -0.789012, 0.456789, 0.012345, -0.678901, 0.234567, -0.890123, 0.567890,
        -0.123456, 0.789012,
    ];

    let embedding = Embedding {
        vector: vector.clone(),
        model: "text-embedding-3-small".to_string(),
        dimensions: 10,
    };

    assert_eq!(embedding.vector.len(), 10);
    assert_eq!(embedding.dimensions, 10);
    assert_eq!(embedding.model, "text-embedding-3-small");

    // Check some specific values
    assert_eq!(embedding.vector[0], 0.123456);
    assert_eq!(embedding.vector[5], 0.234567);
}

#[test]
fn test_embedding_empty_vector() {
    let embedding = Embedding {
        vector: vec![],
        model: "test-model".to_string(),
        dimensions: 0,
    };

    assert!(embedding.vector.is_empty());
    assert_eq!(embedding.dimensions, 0);
}

#[test]
fn test_embedding_large_vector() {
    let vector: Vec<f32> = (0..1536).map(|i| i as f32 * 0.001).collect();
    let embedding = Embedding {
        vector,
        model: "text-embedding-ada-002".to_string(),
        dimensions: 1536,
    };

    assert_eq!(embedding.vector.len(), 1536);
    assert_eq!(embedding.dimensions, 1536);
    assert_eq!(embedding.vector[0], 0.0);
    // Use approximate comparison for floating-point due to precision
    assert!((embedding.vector[1535] - 1.535).abs() < 0.0001);
}
