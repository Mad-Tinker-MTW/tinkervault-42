mod crypto;
mod errors;
mod geoastro;
mod nautical57;
mod payload;
mod seedforge;
mod utils;
mod vault_format;

use crate::seedforge::ForgeInput;
use crate::vault_format::{unwrap_path, wrap_path};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VaultRequest {
    payload_path: String,
    seed: String,
    place: String,
    date: String,
    sky_time: String,
    star: String,
    #[serde(default)]
    overwrite: bool,
}

#[tauri::command]
fn wrap_payload(request: VaultRequest) -> Result<String, String> {
    let output = wrap_path(
        &PathBuf::from(request.payload_path),
        ForgeInput {
            seed: request.seed,
            place: request.place,
            date: request.date,
            sky_time: request.sky_time,
            star: request.star,
        },
        request.overwrite,
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Wrapped successfully:\n{}", output.display()))
}

#[tauri::command]
fn unwrap_vault(request: VaultRequest) -> Result<String, String> {
    let output = unwrap_path(
        &PathBuf::from(request.payload_path),
        ForgeInput {
            seed: request.seed,
            place: request.place,
            date: request.date,
            sky_time: request.sky_time,
            star: request.star,
        },
        request.overwrite,
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Unwrapped successfully:\n{}", output.display()))
}

#[tauri::command]
fn clear_local_cache() -> Result<String, String> {
    let mut removed = 0usize;
    let mut failed: Vec<String> = Vec::new();
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(&local).join("TinkerVault 42"));
        candidates.push(PathBuf::from(&local).join("com.madtinkersworkshop.tinkervault42"));
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        candidates.push(PathBuf::from(&appdata).join("TinkerVault 42"));
        candidates.push(PathBuf::from(&appdata).join("com.madtinkersworkshop.tinkervault42"));
    }

    if let Ok(temp) = std::env::var("TEMP") {
        if let Ok(entries) = std::fs::read_dir(temp) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_lowercase().contains("tinkervault") {
                    candidates.push(entry.path());
                }
            }
        }
    }

    for path in candidates {
        if path.exists() {
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            match result {
                Ok(_) => removed += 1,
                Err(e) => failed.push(format!("{}: {}", path.display(), e)),
            }
        }
    }

    if failed.is_empty() {
        Ok(format!("Cleanup complete. Removed {} cache/temp location(s).", removed))
    } else {
        Ok(format!(
            "Cleanup partial. Removed {}. Failed: {}",
            removed,
            failed.join("; ")
        ))
    }
}

#[tauri::command]
fn delete_original(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("Path not found: {}", path));
    }
    secure_delete_path(p).map_err(|e| e.to_string())?;
    Ok(format!("Deleted: {}", path))
}

fn secure_delete_path(path: &std::path::Path) -> crate::errors::Result<()> {
    if path.is_file() {
        secure_wipe_file(path)?;
        std::fs::remove_file(path).map_err(crate::errors::VaultError::Io)?;
    } else if path.is_dir() {
        for entry in walkdir::WalkDir::new(path).into_iter().flatten() {
            if entry.path().is_file() {
                let _ = secure_wipe_file(entry.path());
            }
        }
        std::fs::remove_dir_all(path).map_err(crate::errors::VaultError::Io)?;
    }
    Ok(())
}

fn secure_wipe_file(path: &std::path::Path) -> crate::errors::Result<()> {
    use std::io::Write;
    let len = path.metadata()?.len();
    if len == 0 {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new().write(true).open(path)?;
    let chunk = vec![0u8; (len.min(1_048_576)) as usize];
    let mut rem = len;
    while rem > 0 {
        let n = (chunk.len() as u64).min(rem) as usize;
        f.write_all(&chunk[..n])?;
        rem -= n as u64;
    }
    f.flush()?;
    Ok(())
}

#[tauri::command]
fn check_wrap_collision(input_path: String) -> Option<String> {
    let path = std::path::Path::new(&input_path);
    if !path.exists() {
        return None;
    }
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let stem = if path.is_dir() {
        path.file_name().unwrap_or_default().to_string_lossy().to_string()
    } else {
        path.file_stem().unwrap_or_default().to_string_lossy().to_string()
    };
    let out = parent.join(format!("{}.TinkerVault", stem));
    if out.exists() {
        Some(out.display().to_string())
    } else {
        None
    }
}

#[tauri::command]
fn open_coords_url(query: String) -> Result<(), String> {
    let encoded: String = query
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || ",-._~".contains(c) {
                c.to_string()
            } else if c == ' ' {
                "+".to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect();
    let url = format!("https://madtinkersworkshop.com/coords?q={}", encoded);
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &url])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            wrap_payload,
            unwrap_vault,
            clear_local_cache,
            delete_original,
            check_wrap_collision,
            open_coords_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TinkerVault 42");
}
