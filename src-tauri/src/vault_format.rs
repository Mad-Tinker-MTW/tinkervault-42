use crate::crypto::{decrypt, derive_key, encrypt, random_bytes, ARGON2_ITERATIONS, ARGON2_MEMORY_KIB, ARGON2_PARALLELISM};
use crate::errors::{Result, VaultError};
use crate::payload::{build_payload, restore_payload, Payload};
use crate::seedforge::{forge_seed, ForgeInput};
use crate::utils;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"TVLT42\x00\x01";
const CIPHER_ID_AES_256_GCM: u8 = 1;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

// magic[8], cipher_id[1], mem_kib[u32], iterations[u32], parallelism[u32], salt_len[u8], nonce_len[u8]
const HEADER_LEN: usize = 8 + 1 + 4 + 4 + 4 + 1 + 1;

#[derive(Clone)]
pub struct Header {
    cipher_id: u8,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
}

impl Header {
    pub fn new() -> Self {
        Header {
            cipher_id: CIPHER_ID_AES_256_GCM,
            memory_kib: ARGON2_MEMORY_KIB,
            iterations: ARGON2_ITERATIONS,
            parallelism: ARGON2_PARALLELISM,
            salt: random_bytes::<SALT_LEN>(),
            nonce: random_bytes::<NONCE_LEN>(),
        }
    }

    pub fn aad(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + SALT_LEN + NONCE_LEN);
        out.extend_from_slice(MAGIC);
        out.push(self.cipher_id);
        out.extend_from_slice(&self.memory_kib.to_be_bytes());
        out.extend_from_slice(&self.iterations.to_be_bytes());
        out.extend_from_slice(&self.parallelism.to_be_bytes());
        out.push(SALT_LEN as u8);
        out.push(NONCE_LEN as u8);
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.nonce);
        out
    }

    pub fn parse(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < HEADER_LEN + SALT_LEN + NONCE_LEN {
            return Err(VaultError::Message("File too small to be a TinkerVault 42 vault.".into()));
        }

        if &data[0..8] != MAGIC {
            return Err(VaultError::Message("Invalid or unsupported TinkerVault vault.".into()));
        }

        let cipher_id = data[8];
        if cipher_id != CIPHER_ID_AES_256_GCM {
            return Err(VaultError::Message("Unsupported cipher ID.".into()));
        }

        let memory_kib = u32::from_be_bytes(data[9..13].try_into().unwrap());
        let iterations = u32::from_be_bytes(data[13..17].try_into().unwrap());
        let parallelism = u32::from_be_bytes(data[17..21].try_into().unwrap());
        let salt_len = data[21] as usize;
        let nonce_len = data[22] as usize;

        if salt_len != SALT_LEN || nonce_len != NONCE_LEN {
            return Err(VaultError::Message("Unsupported salt/nonce length.".into()));
        }

        let salt_start = HEADER_LEN;
        let nonce_start = salt_start + salt_len;
        let cipher_start = nonce_start + nonce_len;

        if data.len() <= cipher_start {
            return Err(VaultError::Message("Vault missing ciphertext.".into()));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&data[salt_start..nonce_start]);

        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&data[nonce_start..cipher_start]);

        Ok((
            Header {
                cipher_id,
                memory_kib,
                iterations,
                parallelism,
                salt,
                nonce,
            },
            &data[cipher_start..],
        ))
    }
}

pub fn wrap_path(path: &Path, input: ForgeInput, overwrite: bool) -> Result<PathBuf> {
    if !path.exists() {
        return Err(VaultError::Message("Input path does not exist.".into()));
    }

    let forged = forge_seed(input)?;
    let payload = build_payload(path)?;
    let plaintext = serde_json::to_vec(&payload)?;

    let header = Header::new();
    let aad = header.aad();
    let key = derive_key(
        &forged.material,
        &header.salt,
        header.memory_kib,
        header.iterations,
        header.parallelism,
    )?;

    let ciphertext = encrypt(&key, &header.nonce, &plaintext, &aad)?;

    let output = unique_vault_output(path, overwrite)?;
    let mut bytes = aad;
    bytes.extend_from_slice(&ciphertext);
    fs::write(&output, bytes)?;
    Ok(output)
}

pub fn unwrap_path(vault_path: &Path, input: ForgeInput, overwrite: bool) -> Result<PathBuf> {
    let data = fs::read(vault_path)?;
    let (header, ciphertext) = Header::parse(&data)?;
    let forged = forge_seed(input)?;

    let aad = header.aad();
    let key = derive_key(
        &forged.material,
        &header.salt,
        header.memory_kib,
        header.iterations,
        header.parallelism,
    )?;

    let plaintext = decrypt(&key, &header.nonce, ciphertext, &aad)?;
    let payload: Payload = serde_json::from_slice(&plaintext)?;
    restore_payload(payload, vault_path, overwrite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{decrypt, derive_key, encrypt};
    use crate::payload::{build_payload, restore_payload, Payload as VaultPayload};

    // Reduced Argon2 params for fast tests. Production always uses Beast Mode constants.
    const T_MEM: u32 = 64;
    const T_ITER: u32 = 1;
    const T_PARA: u32 = 1;

    fn make_header_bytes(salt: [u8; SALT_LEN], nonce: [u8; NONCE_LEN], mem: u32, iter: u32, para: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(MAGIC);
        b.push(CIPHER_ID_AES_256_GCM);
        b.extend_from_slice(&mem.to_be_bytes());
        b.extend_from_slice(&iter.to_be_bytes());
        b.extend_from_slice(&para.to_be_bytes());
        b.push(SALT_LEN as u8);
        b.push(NONCE_LEN as u8);
        b.extend_from_slice(&salt);
        b.extend_from_slice(&nonce);
        b
    }

    #[test]
    fn header_parse_roundtrip() {
        let salt = [0x11u8; SALT_LEN];
        let nonce = [0x22u8; NONCE_LEN];
        let mut bytes = make_header_bytes(salt, nonce, T_MEM, T_ITER, T_PARA);
        bytes.extend_from_slice(b"dummy-ciphertext");

        let (h, ct) = Header::parse(&bytes).unwrap();
        assert_eq!(h.cipher_id, CIPHER_ID_AES_256_GCM);
        assert_eq!(h.memory_kib, T_MEM);
        assert_eq!(h.iterations, T_ITER);
        assert_eq!(h.parallelism, T_PARA);
        assert_eq!(h.salt, salt);
        assert_eq!(h.nonce, nonce);
        assert_eq!(ct, b"dummy-ciphertext");
    }

    #[test]
    fn header_aad_contains_magic_salt_nonce() {
        let salt = [0xAAu8; SALT_LEN];
        let nonce = [0xBBu8; NONCE_LEN];
        let mut bytes = make_header_bytes(salt, nonce, T_MEM, T_ITER, T_PARA);
        bytes.extend_from_slice(b"ct");

        let (h, _) = Header::parse(&bytes).unwrap();
        let aad = h.aad();

        assert_eq!(&aad[..8], MAGIC as &[u8]);
        assert!(aad.windows(SALT_LEN).any(|w| w == salt));
        assert!(aad.windows(NONCE_LEN).any(|w| w == nonce));
    }

    #[test]
    fn header_wrong_magic_rejected() {
        let mut bytes = make_header_bytes([0u8; SALT_LEN], [0u8; NONCE_LEN], T_MEM, T_ITER, T_PARA);
        bytes.extend_from_slice(b"ct");
        bytes[0] = 0xFF;
        assert!(Header::parse(&bytes).is_err());
    }

    #[test]
    fn header_too_short_rejected() {
        assert!(Header::parse(&[0u8; 10]).is_err());
    }

    #[test]
    fn header_missing_ciphertext_rejected() {
        let bytes = make_header_bytes([0u8; SALT_LEN], [0u8; NONCE_LEN], T_MEM, T_ITER, T_PARA);
        // No ciphertext appended — exactly HEADER_LEN + SALT + NONCE bytes
        assert!(Header::parse(&bytes).is_err());
    }

    // Full wrap/unwrap integration test using reduced KDF params.
    // Beast Mode (1 GB / 4 iter) is not exercised here to keep the suite fast.
    #[test]
    fn full_wrap_unwrap_roundtrip() {
        let dir = std::env::temp_dir();
        let src = dir.join("tvlt_integ_src.txt");
        let vault = dir.join("tvlt_integ_vault.TinkerVault");
        let content = b"MTW TinkerVault 42 integration test content.";

        fs::write(&src, content).unwrap();

        let payload = build_payload(&src).unwrap();
        let plaintext = serde_json::to_vec(&payload).unwrap();

        // Build test header with reduced params
        let salt = [0x42u8; SALT_LEN];
        let nonce = [0x43u8; NONCE_LEN];
        let mut hdr_bytes = make_header_bytes(salt, nonce, T_MEM, T_ITER, T_PARA);
        hdr_bytes.extend_from_slice(b"placeholder");
        let (h, _) = Header::parse(&hdr_bytes).unwrap();
        let aad = h.aad();

        let key = derive_key("test-material", &h.salt, h.memory_kib, h.iterations, h.parallelism).unwrap();
        let ct = encrypt(&key, &h.nonce, &plaintext, &aad).unwrap();

        let mut vault_bytes = aad.clone();
        vault_bytes.extend_from_slice(&ct);
        fs::write(&vault, &vault_bytes).unwrap();

        // Delete src so restore has no collision
        fs::remove_file(&src).unwrap();

        // Unwrap
        let raw = fs::read(&vault).unwrap();
        let (h2, ct2) = Header::parse(&raw).unwrap();
        let aad2 = h2.aad();
        assert_eq!(aad2, aad);

        let key2 = derive_key("test-material", &h2.salt, h2.memory_kib, h2.iterations, h2.parallelism).unwrap();
        let plain2 = decrypt(&key2, &h2.nonce, ct2, &aad2).unwrap();
        let payload2: VaultPayload = serde_json::from_slice(&plain2).unwrap();

        let out = restore_payload(payload2, &vault, false).unwrap();
        assert_eq!(fs::read(&out).unwrap(), content as &[u8]);

        let _ = fs::remove_file(&out);
        let _ = fs::remove_file(&vault);
    }

    // Beast Mode round-trip: 1 GB RAM / 4 iter. Run with: cargo test -- --ignored
    // Expect ~30-60 seconds depending on machine.
    #[test]
    #[ignore]
    fn beast_mode_wrap_unwrap_roundtrip() {
        let dir = std::env::temp_dir();
        let src = dir.join("tvlt_beast_src.txt");
        let vault = dir.join("tvlt_beast_vault.TinkerVault");
        let content = b"Beast Mode integration test - full Argon2id 1GB/4iter.";

        fs::write(&src, content).unwrap();

        let payload = build_payload(&src).unwrap();
        let plaintext = serde_json::to_vec(&payload).unwrap();
        fs::remove_file(&src).unwrap();

        // Use production params directly from crypto constants
        let salt = [0xBEu8; SALT_LEN];
        let nonce = [0xEFu8; NONCE_LEN];
        let mut hdr_bytes = make_header_bytes(
            salt,
            nonce,
            crate::crypto::ARGON2_MEMORY_KIB,
            crate::crypto::ARGON2_ITERATIONS,
            crate::crypto::ARGON2_PARALLELISM,
        );
        hdr_bytes.extend_from_slice(b"placeholder");
        let (h, _) = Header::parse(&hdr_bytes).unwrap();
        let aad = h.aad();

        let key = derive_key("beast-mode-material", &h.salt, h.memory_kib, h.iterations, h.parallelism).unwrap();
        let ct = encrypt(&key, &h.nonce, &plaintext, &aad).unwrap();

        let mut vault_bytes = aad.clone();
        vault_bytes.extend_from_slice(&ct);
        fs::write(&vault, &vault_bytes).unwrap();

        let raw = fs::read(&vault).unwrap();
        let (h2, ct2) = Header::parse(&raw).unwrap();
        let aad2 = h2.aad();
        assert_eq!(aad2, aad);

        let key2 = derive_key("beast-mode-material", &h2.salt, h2.memory_kib, h2.iterations, h2.parallelism).unwrap();
        let plain2 = decrypt(&key2, &h2.nonce, ct2, &aad2).unwrap();
        let payload2: VaultPayload = serde_json::from_slice(&plain2).unwrap();

        let out = restore_payload(payload2, &vault, false).unwrap();
        assert_eq!(fs::read(&out).unwrap(), content as &[u8]);

        let _ = fs::remove_file(&out);
        let _ = fs::remove_file(&vault);
    }

    #[test]
    fn wrong_key_fails_integration() {
        let dir = std::env::temp_dir();
        let src = dir.join("tvlt_integ_wrongkey.txt");
        let vault = dir.join("tvlt_integ_wrongkey.TinkerVault");
        fs::write(&src, b"locked content").unwrap();

        let payload = build_payload(&src).unwrap();
        let plaintext = serde_json::to_vec(&payload).unwrap();
        fs::remove_file(&src).unwrap();

        let salt = [0x55u8; SALT_LEN];
        let nonce = [0x56u8; NONCE_LEN];
        let mut hdr_bytes = make_header_bytes(salt, nonce, T_MEM, T_ITER, T_PARA);
        hdr_bytes.extend_from_slice(b"placeholder");
        let (h, _) = Header::parse(&hdr_bytes).unwrap();
        let aad = h.aad();
        let key = derive_key("correct-material", &h.salt, h.memory_kib, h.iterations, h.parallelism).unwrap();
        let ct = encrypt(&key, &h.nonce, &plaintext, &aad).unwrap();

        let mut vault_bytes = aad;
        vault_bytes.extend_from_slice(&ct);
        fs::write(&vault, &vault_bytes).unwrap();

        let raw = fs::read(&vault).unwrap();
        let (h2, ct2) = Header::parse(&raw).unwrap();
        let aad2 = h2.aad();
        let bad_key = derive_key("wrong-material", &h2.salt, h2.memory_kib, h2.iterations, h2.parallelism).unwrap();
        assert!(decrypt(&bad_key, &h2.nonce, ct2, &aad2).is_err());

        let _ = fs::remove_file(&vault);
    }
}

fn unique_vault_output(input: &Path, overwrite: bool) -> Result<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = if input.is_dir() {
        input.file_name().unwrap_or_default().to_string_lossy().to_string()
    } else {
        input.file_stem().unwrap_or_default().to_string_lossy().to_string()
    };
    let candidate = parent.join(format!("{}.TinkerVault", stem));
    utils::unique_path(&candidate, overwrite)
}
