use crate::errors::{Result, VaultError};
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;

pub const ARGON2_MEMORY_KIB: u32 = 1_048_576;
pub const ARGON2_ITERATIONS: u32 = 4;
pub const ARGON2_PARALLELISM: u32 = 1;
pub const KEY_LEN: usize = 32;

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn derive_key(material: &str, salt: &[u8], memory_kib: u32, iterations: u32, parallelism: u32) -> Result<[u8; 32]> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(KEY_LEN))
        .map_err(|e| VaultError::Message(format!("Argon2 parameter error: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(material.as_bytes(), salt, &mut key)
        .map_err(|e| VaultError::Message(format!("Argon2 error: {e}")))?;
    Ok(key)
}

pub fn encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .encrypt(Nonce::from_slice(nonce), Payload { msg: plaintext, aad })
        .map_err(|_| VaultError::Crypto)
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|_| VaultError::Crypto)
}
