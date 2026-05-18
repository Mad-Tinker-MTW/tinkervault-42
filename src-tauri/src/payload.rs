use crate::errors::{Result, VaultError};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
#[serde(tag = "payload_type")]
pub enum Payload {
    #[serde(rename = "file")]
    File {
        original_name: String,
        file_sha256: String,
        file_b64: String,
    },

    #[serde(rename = "folder")]
    Folder {
        original_name: String,
        entries: Vec<FolderEntry>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct FolderEntry {
    relative_path: String,
    sha256: String,
    data_b64: String,
}

pub fn build_payload(input: &Path) -> Result<Payload> {
    if input.is_file() {
        let data = fs::read(input)?;
        Ok(Payload::File {
            original_name: input
                .file_name()
                .ok_or_else(|| VaultError::Message("Missing file name.".into()))?
                .to_string_lossy()
                .to_string(),
            file_sha256: sha256_hex(&data),
            file_b64: B64.encode(data),
        })
    } else if input.is_dir() {
        let root = input.canonicalize()?;
        let mut entries = Vec::new();

        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|_| VaultError::Message("Folder path error.".into()))?
                .to_string_lossy()
                .replace('\\', "/");

            validate_relative_path(&rel)?;
            let data = fs::read(path)?;
            entries.push(FolderEntry {
                relative_path: rel,
                sha256: sha256_hex(&data),
                data_b64: B64.encode(data),
            });
        }

        Ok(Payload::Folder {
            original_name: input
                .file_name()
                .ok_or_else(|| VaultError::Message("Missing folder name.".into()))?
                .to_string_lossy()
                .to_string(),
            entries,
        })
    } else {
        Err(VaultError::Message("Input path must be a file or folder.".into()))
    }
}

pub fn restore_payload(payload: Payload, vault_path: &Path) -> Result<PathBuf> {
    match payload {
        Payload::File {
            original_name,
            file_sha256,
            file_b64,
        } => {
            let out = unique_path(&vault_path.parent().unwrap_or_else(|| Path::new(".")).join(original_name));
            let data = B64.decode(file_b64)?;
            if sha256_hex(&data) != file_sha256 {
                return Err(VaultError::Message("Restored file hash check failed.".into()));
            }
            fs::write(&out, data)?;
            Ok(out)
        }

        Payload::Folder {
            original_name,
            entries,
        } => {
            let root = unique_path(&vault_path.parent().unwrap_or_else(|| Path::new(".")).join(original_name));
            fs::create_dir_all(&root)?;

            for entry in entries {
                validate_relative_path(&entry.relative_path)?;
                let out_file = root.join(&entry.relative_path);
                if let Some(parent) = out_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                let data = B64.decode(entry.data_b64)?;
                if sha256_hex(&data) != entry.sha256 {
                    return Err(VaultError::Message(format!(
                        "Restored file hash check failed for {}.",
                        entry.relative_path
                    )));
                }
                fs::write(out_file, data)?;
            }

            Ok(root)
        }
    }
}

fn validate_relative_path(rel: &str) -> Result<()> {
    if rel.contains("..") || rel.starts_with('/') || rel.starts_with('\\') || rel.contains(':') {
        return Err(VaultError::Message(format!("Unsafe path rejected: {}", rel)));
    }
    Ok(())
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

fn sha256_hex(data: &[u8]) -> String {
    let d = Sha256::digest(data);
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
