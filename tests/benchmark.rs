//! Benchmark tests for performance measurement
//!
//! These tests measure the performance characteristics of key operations
//! to ensure they meet performance requirements.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mcp_context_browser::core::types::{CodeChunk, Embedding, Language};

/// Benchmark core type operations
pub fn bench_core_types(c: &mut Criterion) {
    c.bench_function("create_code_chunk", |b| {
        b.iter(|| {
            let id = black_box("benchmark_id");
            let content = black_box("fn benchmark() { println!(\"test\"); }");
            let file_path = black_box("src/benchmark.rs");
            let start_line = black_box(1);
            let end_line = black_box(3);
            let chunk = CodeChunk {
                id: id.to_string(),
                content: content.to_string(),
                file_path: file_path.to_string(),
                start_line,
                end_line,
                language: Language::Rust,
                metadata: serde_json::json!({"benchmark": true}),
            };
            black_box(chunk);
        });
    });

    c.bench_function("create_embedding", |b| {
        b.iter(|| {
            let vector = black_box(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
            let model = black_box("benchmark-model");
            let dimensions = black_box(5);
            let embedding = Embedding {
                vector,
                model: model.to_string(),
                dimensions,
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
            let chunk_ref = black_box(&chunk);
            let serialized = serde_json::to_string(chunk_ref).unwrap();
            black_box(serialized);
        });
    });

    c.bench_function("deserialize_code_chunk", |b| {
        let json_str = r#"{"id":"benchmark_id","content":"fn benchmark() { println!(\"test\"); }","file_path":"src/benchmark.rs","start_line":1,"end_line":3,"language":"Rust","metadata":{"benchmark":true}}"#;

        b.iter(|| {
            let json_ref = black_box(json_str);
            let chunk: CodeChunk = serde_json::from_str(json_ref).unwrap();
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
            let is_valid = black_box(
                !chunk.content.is_empty()
                    && !chunk.file_path.is_empty()
                    && chunk.start_line > 0
                    && chunk.end_line >= chunk.start_line
            );
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
            let vector_len = embedding.vector.len();
            let is_valid = !embedding.vector.is_empty()
                && embedding.dimensions == vector_len
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
                let operation_count = black_box(1);
                map.insert("operation_count".to_string(), serde_json::json!(operation_count));
                map.insert("last_access".to_string(), serde_json::json!("benchmark"));
                let metadata_size = map.len();
                black_box(metadata_size);
            }
        });
    });
}

/// Benchmark provider operations (mock implementations)
pub fn bench_provider_operations(c: &mut Criterion) {
    c.bench_function("provider_embedding_operation", |b| {
        b.iter(|| {
            // Simulate embedding operation
            let vector = black_box(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
            let model = black_box("benchmark-provider");
            let dimensions = black_box(5);
            let embedding = Embedding {
                vector,
                model: model.to_string(),
                dimensions,
            };
            black_box(embedding);
        });
    });

    c.bench_function("provider_dimension_calculation", |b| {
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        b.iter(|| {
            let vector_ref = black_box(&vector);
            black_box(vector_ref.len());
        });
    });
}

/// Benchmark service operations (mock implementations)
pub fn bench_service_operations(c: &mut Criterion) {
    c.bench_function("service_data_processing", |b| {
        b.iter(|| {
            // Simulate service data processing
            let chunks: Vec<CodeChunk> = (0..10)
                .map(|i| {
                    let id = format!("chunk_{}", i);
                    let content = format!("content_{}", i);
                    let file_path = "test.rs";
                    CodeChunk {
                        id: black_box(id),
                        content: black_box(content),
                        file_path: black_box(file_path.to_string()),
                        start_line: black_box(i as u32 + 1),
                        end_line: black_box(i as u32 + 1),
                        language: Language::Rust,
                        metadata: serde_json::json!({"batch": i}),
                    }
                })
                .collect();

            black_box(chunks.len());
        });
    });

    c.bench_function("service_metadata_aggregation", |b| {
        let chunks: Vec<CodeChunk> = (0..50)
            .map(|i| {
                let id = format!("chunk_{}", i);
                let file_path = format!("file_{}.rs", i);
                CodeChunk {
                    id,
                    content: "content".to_string(),
                    file_path,
                    start_line: 1,
                    end_line: 1,
                    language: Language::Rust,
                    metadata: serde_json::json!({"index": i, "category": "benchmark"}),
                }
            })
            .collect();

        b.iter(|| {
            // Simulate metadata aggregation
            let total_chunks = chunks.len();
            let rust_files = chunks
                .iter()
                .filter(|c| c.language == Language::Rust)
                .count();
            let total_lines: u32 = chunks.len() as u32; // All chunks have exactly 1 line (end_line - start_line + 1 = 1)
            let avg_line_count = if total_chunks > 0 {
                total_lines as f64 / total_chunks as f64
            } else {
                0.0
            };

            black_box((total_chunks, rust_files, avg_line_count));
        });
    });
}

/// Benchmark memory operations
pub fn bench_memory_operations(c: &mut Criterion) {
    c.bench_function("memory_chunk_allocation", |b| {
        b.iter(|| {
            let content = black_box("x".repeat(1000)); // 1KB content
            let chunk = Box::new(CodeChunk {
                id: "memory_test".to_string(),
                content,
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
            let vector_data: Vec<f32> = (0..1000).map(|i| (i as f32).sin()).collect();
            let vector = black_box(vector_data);
            let model = black_box("memory-benchmark");
            let dimensions = black_box(1000);
            let embedding = Embedding {
                vector,
                model: model.to_string(),
                dimensions,
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
            let results: Vec<CodeChunk> = (0..100)
                .map(|i| {
                    let id = format!("concurrent_{}", i);
                    let content = format!("content_{}", i);
                    let file_path = "concurrent.rs";
                    let start_line = i as u32 % 100 + 1;
                    let end_line = i as u32 % 100 + 2;
                    CodeChunk {
                        id: black_box(id),
                        content: black_box(content),
                        file_path: black_box(file_path.to_string()),
                        start_line: black_box(start_line),
                        end_line: black_box(end_line),
                        language: Language::Rust,
                        metadata: serde_json::json!({"concurrent": true, "index": i}),
                    }
                })
                .collect();

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
