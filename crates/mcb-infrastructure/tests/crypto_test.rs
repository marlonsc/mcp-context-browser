//! Cryptography Tests

use mcb_infrastructure::crypto::{
    CryptoService, HashUtils, PasswordService, SecureErasure, TokenGenerator,
};

#[test]
fn test_crypto_service_encrypt_decrypt() {
    let master_key = CryptoService::generate_master_key();
    let service = CryptoService::new(master_key).unwrap();

    let plaintext = b"Hello, World!";
    let encrypted = service.encrypt(plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_crypto_service_invalid_key_size() {
    let invalid_key = vec![0u8; 16]; // Wrong size
    assert!(CryptoService::new(invalid_key).is_err());
}

#[test]
fn test_password_service_hash_verify() {
    let service = PasswordService::new();

    let password = "test_password_123";
    let hash = service.hash_password(password).unwrap();

    assert!(service.verify_password(password, &hash).unwrap());
    assert!(!service.verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_token_generator() {
    let token1 = TokenGenerator::generate_secure_token(32);
    let token2 = TokenGenerator::generate_secure_token(32);

    assert_eq!(token1.len(), 64); // 32 bytes * 2 hex chars
    assert_eq!(token2.len(), 64);
    assert_ne!(token1, token2);
}

#[test]
fn test_hash_utils_hmac() {
    let key = b"secret_key";
    let data = b"test_data";
    let hmac1 = HashUtils::hmac_sha256(key, data).expect("HMAC should succeed");
    let hmac2 = HashUtils::hmac_sha256(key, data).expect("HMAC should succeed");

    assert_eq!(hmac1, hmac2);
    assert_eq!(hmac1.len(), 32); // SHA256 output size
}

#[test]
fn test_secure_erasure() {
    let mut data = vec![1, 2, 3, 4, 5];
    SecureErasure::zeroize(&mut data);
    assert_eq!(data, vec![0, 0, 0, 0, 0]);
}

#[test]
fn test_constant_time_eq() {
    assert!(HashUtils::constant_time_eq(b"test", b"test"));
    assert!(!HashUtils::constant_time_eq(b"test", b"different"));
    assert!(!HashUtils::constant_time_eq(b"test", b"test_longer"));
}
