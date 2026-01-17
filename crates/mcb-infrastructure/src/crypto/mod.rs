//! Cryptographic services module
//!
//! This module provides cryptographic primitives for:
//! - AES-GCM encryption/decryption
//! - Password hashing with Argon2
//! - Secure token generation
//! - Key derivation and secure erasure utilities

mod encryption;
mod password;
mod token;
mod utils;

pub use encryption::CryptoService;
// EncryptedData is in mcb-domain - use mcb_application::ports::providers::EncryptedData
pub use password::PasswordService;
pub use token::TokenGenerator;
pub use utils::{bytes_to_hex, HashUtils, KeyDerivation, SecureErasure};
