//! Cache Queue and Batch Processing Tests

use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::cache::queue::{CacheAsideHelper, CacheBatchProcessor};
use mcb_providers::cache::NullCacheProvider;

#[tokio::test]
async fn test_batch_processor_basic_operations() {
    let provider = SharedCacheProvider::new(NullCacheProvider::new());
    let processor = CacheBatchProcessor::new(provider, 10);

    // Queue operations
    processor
        .queue_set("key1", "value1", CacheEntryConfig::default())
        .await
        .unwrap();
    processor.queue_delete("key2").await.unwrap();

    assert_eq!(processor.queued_count().await, 2);

    // Flush operations
    processor.flush().await.unwrap();
    assert_eq!(processor.queued_count().await, 0);
}

#[tokio::test]
async fn test_batch_processor_auto_flush() {
    let provider = SharedCacheProvider::new(NullCacheProvider::new());
    let processor = CacheBatchProcessor::new(provider, 2); // Small batch size

    // Add operations that should trigger auto-flush
    processor
        .queue_set("key1", "value1", CacheEntryConfig::default())
        .await
        .unwrap();
    assert_eq!(processor.queued_count().await, 1);

    processor
        .queue_set("key2", "value2", CacheEntryConfig::default())
        .await
        .unwrap();
    // Should have auto-flushed, so queue should be empty
    assert_eq!(processor.queued_count().await, 0);
}

#[tokio::test]
async fn test_cache_aside_helper() {
    let provider = SharedCacheProvider::new(NullCacheProvider::new());
    let helper = CacheAsideHelper::new(provider);

    let mut call_count = 0;

    let result = helper
        .get_or_compute("test_key", || {
            call_count += 1;
            async { Ok::<_, mcb_domain::error::Error>("computed_value".to_string()) }
        })
        .await
        .unwrap();

    assert_eq!(call_count, 1);
    assert_eq!(result.value, "computed_value");
    assert!(!result.from_cache);
}
