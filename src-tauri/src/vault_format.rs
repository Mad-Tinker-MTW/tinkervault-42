use crate::crypto::{decrypt, derive_key, encrypt, random_bytes, ARGON2_ITERATIONS, ARGON2_MEMORY_KIB, ARGON2_PARALLELISM};
use crate::errors::{Result, VaultError};
use crate::payload::{build_payload, restore_payload, Payload};
use crate::seedforge::{forge_seed, ForgeInput};
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

pub fn wrap_path(path: &Path, input: ForgeInput) -> Result<PathBuf> {
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

    let output = unique_vault_output(path);
    let mut bytes = aad;
    bytes.extend_from_slice(&ciphertext);
    fs::write(&output, bytes)?;
    Ok(output)
}

pub fn unwrap_path(vault_path: &Path, input: ForgeInput) -> Result<PathBuf> {
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
    restore_payload(payload, vault_path)
}

fn unique_vault_output(input: &Path) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = if input.is_dir() {
        input.file_name().unwrap_or_default().to_string_lossy().to_string()
    } else {
        input.file_stem().unwrap_or_default().to_string_lossy().to_string()
    };
    let first = parent.join(format!("{}.TinkerVault", stem));
    unique_path(&first)
}

fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());

    for i in 1..10_000 {
        let name = if let Some(ext) = &ext {
            format!("{}_{:03}.{}", stem, i, ext)
        } else {
            format!("{}_{:03}", stem, i)
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}
