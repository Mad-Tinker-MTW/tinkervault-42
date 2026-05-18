mod crypto;
mod errors;
mod geoastro;
mod nautical57;
mod payload;
mod seedforge;
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
    )
    .map_err(|e| e.to_string())?;

    Ok(format!("Unwrapped successfully:\n{}", output.display()))
}

#[tauri::command]
fn clear_local_cache() -> Result<String, String> {
    let mut removed = 0usize;
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
            if result.is_ok() {
                removed += 1;
            }
        }
    }

    Ok(format!("Cleanup complete. Removed {} cache/temp location(s).", removed))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![wrap_payload, unwrap_vault, clear_local_cache])
        .run(tauri::generate_context!())
        .expect("error while running TinkerVault 42");
}
