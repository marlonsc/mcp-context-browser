# Integration Test Skipping Strategy

## Overview

Integration tests in MCB can depend on external services (Milvus, Ollama, Redis, PostgreSQL). To prevent CI timeouts and enable graceful test skipping when services are unavailable, we use service detection at test startup.

## Service Detection Helpers

Located in: `crates/mcb-server/tests/integration/helpers.rs`

### Available Detection Functions

```rust
pub fn is_milvus_available() -> bool      // Port 19530
pub fn is_ollama_available() -> bool      // Port 11434
pub fn is_redis_available() -> bool       // Port 6379
pub fn is_postgres_available() -> bool    // Port 5432
pub fn check_service_available(host: &str, port: u16) -> bool  // Generic
```

All functions use TCP connection attempts with 300ms timeout. They check both `127.0.0.1` and `localhost`.

## Test Skip Macros

### Single Service Skip

Use `skip_if_service_unavailable!()` when test requires a single service:

```rust
#[tokio::test]
async fn test_with_milvus_connection() {
    skip_if_service_unavailable!("Milvus", is_milvus_available());

    // Test code that requires Milvus
    // This code only runs if Milvus is available
    let store = create_milvus_store().await;
    store.create_collection("test", 384).await.expect("create collection");
}
```

Output when service unavailable:

```
⊘ SKIPPED: Milvus service not available (skipping test)
```

### Multiple Service Skip

Use `skip_if_any_service_unavailable!()` when test requires multiple services:

```rust
#[tokio::test]
async fn test_with_milvus_and_ollama() {
    skip_if_any_service_unavailable!(
        "Milvus" => is_milvus_available(),
        "Ollama" => is_ollama_available()
    );

    // Test code requiring both services
    let embedding_provider = get_ollama_embeddings().await;
    let vector_store = get_milvus_store().await;
}
```

Output when any service unavailable:

```
⊘ SKIPPED: Milvus service not available (skipping test)
```

## When to Use Service Skipping

### YES - Use service skipping if

-   Test calls `init_app()` with configuration pointing to external services
-   Test uses actual provider implementations (Milvus, Ollama, Redis, PostgreSQL)
-   Test performs real network connections
-   Test integration is optional (not core to CI validation)

### NO - Don't use if

-   Test uses mock/null providers (default config)
-   Test uses in-memory implementations (InMemoryVectorStore)
-   Test is critical and **must** run (no external dependencies)
-   Test exercises local-only functionality

## Test Categories

### Category A: Always Run (No External Dependencies)

These tests use mock/null providers and always pass:

```rust
#[tokio::test]
async fn test_init_app_with_default_config_succeeds() {
    // Default config uses null/memory providers
    let config = AppConfig::default();  // ← null providers
    let ctx = init_app(config).await.expect("init_app");
    assert!(ctx.embedding_handle().get().is_some());
}
```

**Files**:

-   `error_recovery_integration.rs` - config validation, error handling
-   `golden_acceptance_integration.rs` - in-memory acceptance tests
-   `browse_api_integration.rs` - mock-based API tests

### Category B: Skip if Services Unavailable

These tests require external services and should skip gracefully:

```rust
#[tokio::test]
async fn test_with_real_milvus_store() {
    skip_if_service_unavailable!("Milvus", is_milvus_available());

    // Only runs if Milvus is actually available
    let config = AppConfig {
        vector_store: VectorStoreConfig {
            provider: "milvus",
            address: "http://localhost:19530",
            ..
        },
        ..
    };
    let ctx = init_app(config).await.expect("init_app");
}
```

**When to apply**: Tests that would timeout or fail when services are unavailable.

## Implementation Notes

### Placement

Place skip macros at the **very beginning** of the test function, before any async operations:

```rust
#[tokio::test]
async fn test_example() {
    // ✅ CORRECT - Skip check is first
    skip_if_service_unavailable!("Milvus", is_milvus_available());

    // Now it's safe to use the service
    let store = connect_to_milvus().await;
}
```

```rust
#[tokio::test]
async fn test_example_bad() {
    // ❌ WRONG - Skip check is after setup
    let config = load_config().await;  // Could timeout if Milvus down
    skip_if_service_unavailable!("Milvus", is_milvus_available());
}
```

### Detection Timing

Service detection checks are **very fast** (300ms timeout per service):

-   Succeeds immediately if service is up (TCP handshake succeeds)
-   Fails quickly if service is down (timeout after 300ms)
-   Total test function overhead: ~300-900ms for all checks

### Example: Complete Test with Skip

```rust
//! Full-stack integration test requiring Milvus

#[tokio::test]
async fn test_milvus_vector_search_end_to_end() {
    // 1. Check service availability at test start
    skip_if_service_unavailable!("Milvus", is_milvus_available());

    // 2. Configure with real Milvus connection
    let config = AppConfig {
        providers: ProvidersConfig {
            vector_store: VectorStoreConfigContainer {
                provider: Some("milvus".to_string()),
                address: Some("http://localhost:19530".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    // 3. Initialize with real provider
    let ctx = init_app(config)
        .await
        .expect("Failed to initialize with Milvus");

    // 4. Test Milvus functionality
    let store = ctx.vector_store_handle().get();
    let collection = "test_search";

    store
        .create_collection(collection, 384)
        .await
        .expect("Failed to create collection");

    let embeddings = vec![vec![0.1; 384]];
    store
        .upsert_vectors(collection, embeddings, vec![vec![("id", "1")]])
        .await
        .expect("Failed to upsert vectors");

    // Test the search
    let results = store
        .search_similar(collection, &vec![0.1; 384], 10, None)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty());
}
```

## CI Integration

### Local Development

Tests with skip macros run:

-   **Full**: If all required services are up
-   **Skipped**: If any required service is down
-   Never timeout or fail due to service unavailability

```bash
# Runs all tests, skipping those with unavailable services
make test

# Runs with service detection
cargo test -p mcb-server --test integration
```

### GitHub Actions

CI should:

1.  **Skip integration tests by default** in fast CI path:

   ```yaml
   test-pr:
     # Fast PR validation - only unit tests
     run: cargo test --lib
   ```

1.  **Run full tests (with skipping) on main branch**:

   ```yaml
   test-main:
     # Full suite with optional integration tests
     run: cargo test
     # Services not available in GH Actions, so tests skip gracefully
   ```

1.  **Run against services in E2E environment** (future):

   ```yaml
   test-e2e:
     services:
       milvus:
         image: milvusdb/milvus
         ports:
           - 19530
     run: cargo test
     # Full integration tests run because Milvus is available
   ```

## Troubleshooting

### Tests Skipping When Service Is Running

If tests skip even though services are running:

1.  **Check service is on correct port**:

   ```bash
   netstat -tlnp | grep -E "19530|11434|5432|6379"
   ```

1.  **Test detection manually**:

   ```bash
   cargo test -p mcb-server test_service_detection_logic -- --nocapture
   ```

1.  **Check firewall/localhost resolution**:

   ```bash
   nc -zv 127.0.0.1 19530
   nc -zv localhost 19530
   ```

### Tests Timing Out Despite Skip

If tests timeout:

1.  **Verify skip macro is at test start** (not after async operations)
2.  **Check if skip is conditional** on wrong service
3.  **Increase timeout temporarily for debugging**:

   ```rust
   skip_if_service_unavailable!("Milvus", is_milvus_available());
   // Add explicit timeout
   let timeout = std::time::Duration::from_secs(5);
   ```

## Related Files

-   `crates/mcb-server/tests/integration/helpers.rs` - Detection functions and macros
-   `crates/mcb-server/tests/integration.rs` - Test module root
-   `make/Makefile.quality.mk` - Coverage target configuration
-   `.github/workflows/ci.yml` - CI pipeline

## Future Improvements

1.  **Service availability reporting**: Summary of which services were available during test run
2.  **Conditional test groups**: Run full suite only if all services available
3.  **Docker Compose for local E2E**: Auto-start services for comprehensive testing
4.  **Coverage integration**: Exclude skipped tests from coverage calculations
