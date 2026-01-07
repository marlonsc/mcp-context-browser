//! Benchmark tests for performance measurement
//!
//! These tests measure the performance characteristics of key operations
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mcp_context_browser::core::types::{CodeChunk, Language, Embedding};

/// Benchmark core type operations
pub fn bench_core_types(c: &mut Criterion) {
    c.bench_function("create_code_chunk", |b| {
        b.iter(|| {
            let chunk = CodeChunk {
                id: black_box("benchmark_id".to_string()),
                content: black_box("fn benchmark() { println!(\"test\"); }".to_string()),
                file_path: black_box("src/benchmark.rs".to_string()),
                start_line: black_box(1),
                end_line: black_box(3),
                language: Language::Rust,
                metadata: serde_json::json!({"benchmark": true}),
            };
            black_box(chunk);
        });
    });

    c.bench_function("create_embedding", |b| {
        b.iter(|| {
            let embedding = Embedding {
                vector: black_box(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
                model: black_box("benchmark-model".to_string()),
                dimensions: black_box(5),
            };
            black_box(embedding);
        });
    });

    c.bench_function("serialize_code_chunk", |b| {
        let chunk = CodeChunk {
            id: "benchmark_id".to_string(),
            content: "fn benchmark() { println!(\"test\"); }".to_string(),
            file_path: "src/benchmark.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"benchmark": true}),
        };

        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&chunk)).unwrap();
            black_box(serialized);
        });
    });

    c.bench_function("deserialize_code_chunk", |b| {
        let json_str = r#"{"id":"benchmark_id","content":"fn benchmark() { println!(\"test\"); }","file_path":"src/benchmark.rs","start_line":1,"end_line":3,"language":"Rust","metadata":{"benchmark":true}}"#;

        b.iter(|| {
            let chunk: CodeChunk = serde_json::from_str(black_box(json_str)).unwrap();
            black_box(chunk);
        });
    });
}

/// Benchmark validation operations
pub fn bench_validation(c: &mut Criterion) {
    c.bench_function("validate_code_chunk_basic", |b| {
        let chunk = CodeChunk {
            id: "valid_id".to_string(),
            content: "fn test() {}".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        b.iter(|| {
            // Basic validation checks
            let is_valid = !chunk.content.is_empty()
                && !chunk.file_path.is_empty()
                && chunk.start_line > 0
                && chunk.end_line >= chunk.start_line;
            black_box(is_valid);
        });
    });

    c.bench_function("validate_embedding_basic", |b| {
        let embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3],
            model: "test".to_string(),
            dimensions: 3,
        };

        b.iter(|| {
            let is_valid = !embedding.vector.is_empty()
                && embedding.dimensions == embedding.vector.len()
                && !embedding.model.is_empty();
            black_box(is_valid);
        });
    });
}

/// Benchmark repository operations (mock implementations)
pub fn bench_repository_operations(c: &mut Criterion) {
    c.bench_function("create_repository_chunk", |b| {
        b.iter(|| {
            // Simulate repository chunk creation
            let chunk = CodeChunk {
                id: "repo_chunk".to_string(),
                content: "repository content".to_string(),
                file_path: "repo/test.rs".to_string(),
                start_line: 1,
                end_line: 1,
                language: Language::Rust,
                metadata: serde_json::json!({"repository": "test"}),
            };
            black_box(chunk);
        });
    });

    c.bench_function("repository_metadata_operations", |b| {
        let mut chunk = CodeChunk {
            id: "metadata_test".to_string(),
            content: "test".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        b.iter(|| {
            // Simulate metadata operations
            if let serde_json::Value::Object(ref mut map) = chunk.metadata {
                map.insert("operation_count".to_string(), serde_json::json!(black_box(1)));
                map.insert("last_access".to_string(), serde_json::json!("benchmark"));
                let _count = map.len();
            }
        });
    });
}

/// Benchmark provider operations (mock implementations)
pub fn bench_provider_operations(c: &mut Criterion) {
    c.bench_function("provider_embedding_operation", |b| {
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        b.iter(|| {
            // Simulate embedding operation
            let embedding = Embedding {
                vector: black_box(vector.clone()),
                model: black_box("benchmark-provider".to_string()),
                dimensions: black_box(vector.len()),
            };
            black_box(embedding);
        });
    });

    c.bench_function("provider_dimension_calculation", |b| {
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

        b.iter(|| {
            let dimensions = black_box(vector.len());
            black_box(dimensions);
        });
    });
}

/// Benchmark service operations (mock implementations)
pub fn bench_service_operations(c: &mut Criterion) {
    c.bench_function("service_data_processing", |b| {
        b.iter(|| {
            // Simulate service data processing
            let chunks: Vec<CodeChunk> = (0..10).map(|i| CodeChunk {
                id: format!("chunk_{}", i),
                content: format!("content_{}", i),
                file_path: "test.rs".to_string(),
                start_line: i as u32 + 1,
                end_line: i as u32 + 1,
                language: Language::Rust,
                metadata: serde_json::json!({"batch": i}),
            }).collect();

            let processed_count = chunks.len();
            black_box(processed_count);
        });
    });

    c.bench_function("service_metadata_aggregation", |b| {
        let chunks: Vec<CodeChunk> = (0..50).map(|i| CodeChunk {
            id: format!("chunk_{}", i),
            content: "content".to_string(),
            file_path: format!("file_{}.rs", i),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({"index": i, "category": "benchmark"}),
        }).collect();

        b.iter(|| {
            // Simulate metadata aggregation
            let total_chunks = chunks.len();
            let rust_files = chunks.iter()
                .filter(|c| c.language == Language::Rust)
                .count();
            let avg_line_count = chunks.iter()
                .map(|c| (c.end_line - c.start_line + 1) as f64)
                .sum::<f64>() / chunks.len() as f64;

            black_box((total_chunks, rust_files, avg_line_count));
        });
    });
}

/// Benchmark memory operations
pub fn bench_memory_operations(c: &mut Criterion) {
    c.bench_function("memory_chunk_allocation", |b| {
        b.iter(|| {
            let chunk = Box::new(CodeChunk {
                id: "memory_test".to_string(),
                content: "x".repeat(1000), // 1KB content
                file_path: "memory/test.rs".to_string(),
                start_line: 1,
                end_line: 10,
                language: Language::Rust,
                metadata: serde_json::json!({"memory_test": true, "size": "1kb"}),
            });
            black_box(chunk);
        });
    });

    c.bench_function("memory_vector_operations", |b| {
        b.iter(|| {
            let vector = (0..1000).map(|i| (i as f32).sin()).collect::<Vec<f32>>();
            let embedding = Embedding {
                vector: black_box(vector),
                model: black_box("memory-benchmark".to_string()),
                dimensions: black_box(1000),
            };
            black_box(embedding);
        });
    });
}

/// Benchmark concurrent operations (simulated)
pub fn bench_concurrent_operations(c: &mut Criterion) {
    c.bench_function("concurrent_data_processing", |b| {
        b.iter(|| {
            // Simulate concurrent processing of multiple chunks
            let results: Vec<_> = (0..100).map(|i| {
                let chunk = CodeChunk {
                    id: format!("concurrent_{}", i),
                    content: format!("content_{}", i),
                    file_path: "concurrent.rs".to_string(),
                    start_line: i as u32 % 100 + 1,
                    end_line: i as u32 % 100 + 2,
                    language: Language::Rust,
                    metadata: serde_json::json!({"concurrent": true, "index": i}),
                };
                chunk
            }).collect();

            black_box(results.len());
        });
    });
}

criterion_group!(
    benches,
    bench_core_types,
    bench_validation,
    bench_repository_operations,
    bench_provider_operations,
    bench_service_operations,
    bench_memory_operations,
    bench_concurrent_operations
);
criterion_main!(benches);