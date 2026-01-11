#[cfg(test)]
mod reproduction_test {
    use mcp_context_browser::infrastructure::cache::{
        CacheConfig, CacheManager, CacheNamespacesConfig,
    };
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_concurrent_access_freeze() {
        let config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(), // Local only
            default_ttl_seconds: 60,
            max_size: 1000,
            namespaces: CacheNamespacesConfig::default(),
        };

        let manager = Arc::new(CacheManager::new(config, None).await.unwrap());
        let mut handles = vec![];

        // Spawn 100 concurrent readers/writers
        for _i in 0..100 {
            let m = manager.clone();
            handles.push(tokio::spawn(async move {
                for j in 0..1000 {
                    let key = format!("key-{}", j % 100);
                    if j % 10 == 0 {
                        let _ = m.set("test", &key, "value".to_string()).await;
                    } else {
                        let _ = m.get::<String>("test", &key).await;
                    }
                }
            }));
        }

        // Should finish quickly, but if it locks up, timeout will catch it
        let result = timeout(Duration::from_secs(5), async {
            for h in handles {
                h.await.unwrap();
            }
        })
        .await;

        if result.is_err() {
            panic!("Test timed out - likely deadlock or extreme contention");
        }
    }
}
