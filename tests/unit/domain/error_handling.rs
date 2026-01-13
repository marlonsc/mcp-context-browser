//! Comprehensive error handling tests for core modules
//!
//! This test suite validates that all core modules properly handle errors
//! without using unwrap/expect, ensuring robust error propagation.

use mcp_context_browser::infrastructure::auth::{AuthConfig, AuthService};
use mcp_context_browser::infrastructure::cache::{
    create_cache_provider, CacheBackendConfig, CacheConfig,
};

#[cfg(test)]
mod auth_error_handling_tests {
    use super::*;

    #[test]
    fn test_auth_service_handles_disabled_auth_errors() {
        let config = AuthConfig {
            enabled: false,
            ..Default::default()
        };
        let auth = AuthService::new(config);

        // authenticate() is sync, should return proper error for disabled auth
        let result = auth.authenticate("user", "pass");
        assert!(result.is_err());
        let err = result.expect_err("Expected error for disabled auth");
        assert!(
            err.to_string().to_lowercase().contains("disabled"),
            "Error should mention 'disabled', got: {}",
            err
        );
    }

    #[test]
    fn test_auth_service_handles_invalid_credentials_errors() {
        // Create auth with authentication enabled
        let config = AuthConfig {
            enabled: true,
            ..Default::default()
        };
        let auth = AuthService::new(config);

        // Should return proper error instead of panicking
        let result = auth.authenticate("invalid@email.com", "wrongpass");
        assert!(result.is_err());
        let err = result.expect_err("Expected error for invalid credentials");
        // Error should be about invalid credentials
        assert!(
            err.to_string().to_lowercase().contains("invalid")
                || err.to_string().to_lowercase().contains("credentials"),
            "Error should mention invalid credentials, got: {}",
            err
        );
    }

    #[test]
    fn test_auth_service_handles_token_validation_errors() {
        // Need enabled auth for token validation
        let config = AuthConfig {
            enabled: true,
            ..Default::default()
        };
        let auth = AuthService::new(config);

        // Should return proper error for invalid tokens instead of panicking
        let result = auth.validate_token("invalid.jwt.token");
        assert!(result.is_err());
        let err = result.expect_err("Expected error for invalid token");
        assert!(
            err.to_string().to_lowercase().contains("invalid")
                || err.to_string().to_lowercase().contains("token"),
            "Error should mention token issue, got: {}",
            err
        );
    }

    #[test]
    fn test_auth_service_disabled_token_validation_errors() {
        // Disabled auth should fail validation
        let auth = AuthService::default(); // disabled by default

        let result = auth.validate_token("any.token.here");
        assert!(result.is_err());
        let err = result.expect_err("Expected error for disabled auth");
        assert!(
            err.to_string().to_lowercase().contains("disabled"),
            "Error should mention disabled, got: {}",
            err
        );
    }
}

#[cfg(test)]
mod cache_error_handling_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_provider_handles_disabled_cache_operations() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        // Disabled cache should return NullCacheProvider
        let cache = create_cache_provider(&config)
            .await
            .expect("Disabled cache should create successfully");

        // Operations on disabled cache should not panic, just no-op
        let set_result = cache
            .set(
                "test",
                "key",
                "value".as_bytes().to_vec(),
                Duration::from_secs(3600),
            )
            .await;
        assert!(
            set_result.is_ok(),
            "Set on disabled cache should succeed (no-op)"
        );

        let get_result = cache.get("test", "key").await;
        assert!(get_result.is_ok(), "Get on disabled cache should succeed");
        assert!(
            get_result.unwrap().is_none(),
            "Get on disabled cache should return None"
        );
    }

    #[tokio::test]
    async fn test_cache_provider_handles_local_moka_operations() {
        let config = CacheConfig {
            enabled: true,
            backend: CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 3600,
            },
            ..Default::default()
        };

        let cache = create_cache_provider(&config)
            .await
            .expect("Local cache should create successfully");

        // These operations should not panic
        let clear_result = cache.clear(Some("test_ns")).await;
        assert!(clear_result.is_ok(), "Clear namespace should succeed");

        let delete_result = cache.delete("test_ns", "key").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let stats = cache.get_stats("test_ns").await;
        // Stats should be valid
        assert!(stats.is_ok(), "get_stats should succeed");
        if let Ok(stat) = stats {
            assert!(stat.hit_ratio >= 0.0 && stat.hit_ratio <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_cache_provider_handles_large_data_operations() {
        let config = CacheConfig {
            enabled: true,
            backend: CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 3600,
            },
            ..Default::default()
        };

        let cache = create_cache_provider(&config)
            .await
            .expect("Local cache should create successfully");

        // Test with moderately large data (100KB to avoid memory issues in tests)
        let large_data = "x".repeat(100 * 1024);
        let large_bytes = large_data.as_bytes().to_vec();

        let set_result = cache
            .set(
                "test",
                "large_key",
                large_bytes.clone(),
                Duration::from_secs(3600),
            )
            .await;
        assert!(set_result.is_ok(), "Set with large data should succeed");

        let get_result = cache.get("test", "large_key").await;
        assert!(get_result.is_ok(), "Get should succeed");
        let data = get_result.unwrap();
        assert!(data.is_some(), "Expected data in cache");
        assert_eq!(data.unwrap(), large_bytes, "Data should match");
    }
}

#[cfg(test)]
mod crypto_error_handling_tests {
    use mcp_context_browser::infrastructure::crypto::{
        CryptoService, EncryptionAlgorithm, EncryptionConfig, MasterKeyConfig,
    };

    #[tokio::test]
    async fn test_crypto_service_handles_disabled_crypto_operations() {
        let config = EncryptionConfig {
            enabled: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            master_key: MasterKeyConfig::default(),
        };

        let crypto = CryptoService::new(config)
            .await
            .expect("Disabled crypto service should create successfully");

        // encrypt() is sync and should return error for disabled crypto
        let encrypt_result = crypto.encrypt("test data".as_bytes());
        assert!(encrypt_result.is_err(), "Encrypt should fail when disabled");
        let err = encrypt_result.expect_err("Expected error");
        assert!(
            err.to_string().to_lowercase().contains("disabled"),
            "Error should mention disabled, got: {}",
            err
        );

        // decrypt() also needs an EncryptedEnvelope, but we can test with a dummy
        // Since crypto is disabled, it should fail before checking the envelope
        let dummy_envelope = mcp_context_browser::infrastructure::crypto::EncryptedEnvelope {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            ciphertext: vec![1, 2, 3],
            nonce: vec![0u8; 12],
            key_id: "test".to_string(),
            encrypted_at: 0,
        };
        let decrypt_result = crypto.decrypt(&dummy_envelope);
        assert!(decrypt_result.is_err(), "Decrypt should fail when disabled");
    }

    #[tokio::test]
    async fn test_crypto_service_handles_key_generation() {
        // Enabled config - will create/load master key
        let config = EncryptionConfig {
            enabled: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            master_key: MasterKeyConfig {
                key_path: "/tmp/test_master_key_error_handling.key".to_string(),
                rotation_days: 30,
            },
        };

        let crypto = CryptoService::new(config)
            .await
            .expect("Crypto service should create successfully");

        // generate_data_key() is sync and should work
        let key_result = crypto.generate_data_key();
        assert!(key_result.is_ok(), "Key generation should succeed");
        let key = key_result.expect("Key should be valid");
        assert_eq!(key.key.len(), 32, "Key should be 256 bits");
        assert!(!key.id.is_empty(), "Key should have an ID");

        // Clean up test key file
        let _ = std::fs::remove_file("/tmp/test_master_key_error_handling.key");
    }

    #[tokio::test]
    async fn test_crypto_service_encrypt_decrypt_roundtrip() {
        let config = EncryptionConfig {
            enabled: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            master_key: MasterKeyConfig {
                key_path: "/tmp/test_master_key_roundtrip.key".to_string(),
                rotation_days: 30,
            },
        };

        let crypto = CryptoService::new(config)
            .await
            .expect("Crypto service should create successfully");

        let original_data = b"Hello, World! This is test data.";

        // Encrypt (sync)
        let envelope = crypto
            .encrypt(original_data)
            .expect("Encryption should succeed");
        assert!(
            !envelope.ciphertext.is_empty(),
            "Ciphertext should not be empty"
        );
        assert_eq!(envelope.nonce.len(), 12, "Nonce should be 96 bits");

        // Decrypt (sync)
        let decrypted = crypto
            .decrypt(&envelope)
            .expect("Decryption should succeed");
        assert_eq!(
            decrypted.as_slice(),
            original_data,
            "Decrypted data should match"
        );

        // Clean up
        let _ = std::fs::remove_file("/tmp/test_master_key_roundtrip.key");
    }
}

#[cfg(test)]
mod database_error_handling_tests {
    use mcp_context_browser::adapters::database::{DatabaseConfig, DatabasePool};
    use std::time::Duration;

    #[test]
    fn test_database_pool_handles_disabled_database_operations() {
        let config = DatabaseConfig {
            enabled: false,
            url: String::new(),
            max_connections: 10,
            min_idle: 5,
            max_lifetime: Duration::from_secs(1800),
            idle_timeout: Duration::from_secs(600),
            connection_timeout: Duration::from_secs(30),
        };

        // Disabled database should create successfully (but be non-functional)
        let db = DatabasePool::new(config).expect("Disabled pool should create");
        assert!(!db.is_enabled(), "Pool should be disabled");

        // Operations on disabled pool should return proper errors
        let conn_result = db.get_connection();
        assert!(
            conn_result.is_err(),
            "Get connection should fail when disabled"
        );
        // Use match instead of expect_err to avoid Debug requirement on pooled connection
        match conn_result {
            Err(err) => {
                assert!(
                    err.to_string().to_lowercase().contains("disabled"),
                    "Error should mention disabled, got: {}",
                    err
                );
            }
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[test]
    fn test_database_pool_handles_invalid_url() {
        let config = DatabaseConfig {
            enabled: true,
            url: "invalid://not-a-valid-url".to_string(),
            max_connections: 10,
            min_idle: 5,
            max_lifetime: Duration::from_secs(1800),
            idle_timeout: Duration::from_secs(600),
            connection_timeout: Duration::from_secs(30),
        };

        // Invalid URL should fail on creation
        let result = DatabasePool::new(config);
        assert!(result.is_err(), "Invalid URL should fail");
    }

    #[test]
    fn test_database_pool_stats_when_disabled() {
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };

        let db = DatabasePool::new(config).expect("Disabled pool should create");
        let stats = db.stats();

        // Stats should show zeros for disabled pool
        assert_eq!(stats.connections, 0);
        assert_eq!(stats.idle_connections, 0);
    }

    #[tokio::test]
    async fn test_database_pool_health_check_when_disabled() {
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };

        let db = DatabasePool::new(config).expect("Disabled pool should create");
        let health_result = db.health_check().await;

        assert!(
            health_result.is_err(),
            "Health check should fail when disabled"
        );
    }
}

#[cfg(test)]
mod integration_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_core_services_handle_cascading_errors() {
        // Test that errors propagate correctly through service layers
        // This ensures no unwrap/expect calls break the error chain

        // Auth with enabled=true for testing
        let auth_config = AuthConfig {
            enabled: true,
            ..Default::default()
        };
        let auth = AuthService::new(auth_config);

        // Cache in local mode
        let cache_config = CacheConfig {
            enabled: true,
            backend: mcp_context_browser::infrastructure::cache::CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 3600,
            },
            namespaces: Default::default(),
        };
        let cache =
            mcp_context_browser::infrastructure::cache::create_cache_provider(&cache_config)
                .await
                .expect("Local cache should create");

        // Test auth failure doesn't crash the system (sync call)
        let auth_result = auth.authenticate("nonexistent", "wrong");
        assert!(auth_result.is_err(), "Auth should fail for invalid user");

        // Test cache operations still work after auth failure
        let value = "value".to_string().into_bytes();
        let cache_result = cache
            .set("test", "key", value, std::time::Duration::from_secs(3600))
            .await;
        assert!(cache_result.is_ok(), "Cache should still work");
    }

    #[test]
    fn test_error_context_preservation() {
        // Test that error context is preserved through multiple layers
        // Auth is disabled by default
        let auth = AuthService::default();

        // authenticate() is sync
        let result = auth.authenticate("", "");
        assert!(result.is_err());

        let error = result.expect_err("Expected auth error for empty credentials");
        // Error should contain useful context, not just "Generic error"
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        // Should NOT contain unwrap panic messages
        assert!(
            !error_msg.contains("called `Result::unwrap()`"),
            "Error should not be from unwrap panic"
        );
        // Should NOT contain expect panic messages
        assert!(
            !error_msg.contains("called `Option::expect()`"),
            "Error should not be from expect panic"
        );
    }

    #[tokio::test]
    async fn test_service_isolation() {
        // Test that failures in one service don't affect others
        let cache_config = CacheConfig {
            enabled: true,
            backend: mcp_context_browser::infrastructure::cache::CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 3600,
            },
            namespaces: Default::default(),
        };
        let cache =
            mcp_context_browser::infrastructure::cache::create_cache_provider(&cache_config)
                .await
                .expect("Local cache should create");

        let ttl = std::time::Duration::from_secs(3600);

        // Multiple operations should be isolated
        cache
            .set("ns1", "key1", "value1".to_string().into_bytes(), ttl)
            .await
            .expect("Set 1");
        cache
            .set("ns2", "key2", "value2".to_string().into_bytes(), ttl)
            .await
            .expect("Set 2");

        // Clearing one namespace shouldn't affect the other
        cache.clear(Some("ns1")).await.expect("Clear ns1");

        let result1 = cache.get("ns1", "key1").await;
        assert!(
            result1.is_ok() && result1.unwrap().is_none(),
            "ns1:key1 should be cleared"
        );

        let result2 = cache.get("ns2", "key2").await;
        assert!(
            result2.is_ok() && result2.unwrap().is_some(),
            "ns2:key2 should still exist"
        );
    }
}
