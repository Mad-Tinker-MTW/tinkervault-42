// TODO BUG-02 remaining: document metadata (needs format-specific parsers),
//      general NTFS ADS enumeration (needs BackupRead).
//      Implemented: timestamps (files+dirs via FILE_FLAG_BACKUP_SEMANTICS),
//      file_attributes, Zone.Identifier, owner SID (best-effort, requires SeRestorePrivilege on restore).

use crate::errors::{Result, VaultError};
use crate::utils::unique_path;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Default)]
pub struct FileMetadata {
    #[serde(default)]
    pub created_secs: Option<u64>,
    #[serde(default)]
    pub modified_secs: Option<u64>,
    #[serde(default)]
    pub accessed_secs: Option<u64>,
    #[serde(default)]
    pub file_attributes: Option<u32>,
    #[serde(default)]
    pub zone_identifier: Option<String>,
    #[serde(default)]
    pub owner_sid: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "payload_type")]
pub enum Payload {
    #[serde(rename = "file")]
    File {
        original_name: String,
        file_sha256: String,
        file_b64: String,
        #[serde(default)]
        metadata: Option<FileMetadata>,
    },

    #[serde(rename = "folder")]
    Folder {
        original_name: String,
        entries: Vec<FolderEntry>,
        #[serde(default)]
        folder_metadata: Option<FileMetadata>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct FolderEntry {
    relative_path: String,
    sha256: String,
    data_b64: String,
    #[serde(default)]
    metadata: Option<FileMetadata>,
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
            file_b64: B64.encode(&data),
            metadata: Some(capture_metadata(input)),
        })
    } else if input.is_dir() {
        let root = input.canonicalize()?;
        let folder_metadata = Some(capture_metadata(&root));
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
                data_b64: B64.encode(&data),
                metadata: Some(capture_metadata(path)),
            });
        }

        Ok(Payload::Folder {
            original_name: input
                .file_name()
                .ok_or_else(|| VaultError::Message("Missing folder name.".into()))?
                .to_string_lossy()
                .to_string(),
            entries,
            folder_metadata,
        })
    } else {
        Err(VaultError::Message("Input path must be a file or folder.".into()))
    }
}

pub fn restore_payload(payload: Payload, vault_path: &Path, overwrite: bool) -> Result<PathBuf> {
    match payload {
        Payload::File {
            original_name,
            file_sha256,
            file_b64,
            metadata,
        } => {
            let out = unique_path(
                &vault_path.parent().unwrap_or_else(|| Path::new(".")).join(&original_name),
                overwrite,
            )?;
            let data = B64.decode(file_b64)?;
            if sha256_hex(&data) != file_sha256 {
                return Err(VaultError::Message("Restored file hash check failed.".into()));
            }
            fs::write(&out, &data)?;
            if let Some(meta) = &metadata {
                apply_metadata(&out, meta);
            }
            Ok(out)
        }

        Payload::Folder {
            original_name,
            entries,
            folder_metadata,
        } => {
            let root = unique_path(
                &vault_path.parent().unwrap_or_else(|| Path::new(".")).join(&original_name),
                overwrite,
            )?;
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
                fs::write(&out_file, &data)?;
                if let Some(meta) = &entry.metadata {
                    apply_metadata(&out_file, meta);
                }
            }

            if let Some(meta) = &folder_metadata {
                apply_metadata(&root, meta);
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

fn capture_metadata(path: &Path) -> FileMetadata {
    let Ok(meta) = fs::metadata(path) else {
        return FileMetadata::default();
    };

    let to_secs = |t: SystemTime| {
        t.duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    };

    let created_secs = meta.created().ok().and_then(to_secs);
    let modified_secs = meta.modified().ok().and_then(to_secs);
    let accessed_secs = meta.accessed().ok().and_then(to_secs);

    #[cfg(windows)]
    let file_attributes = {
        use std::os::windows::fs::MetadataExt;
        Some(meta.file_attributes())
    };
    #[cfg(not(windows))]
    let file_attributes: Option<u32> = None;

    FileMetadata {
        created_secs,
        modified_secs,
        accessed_secs,
        file_attributes,
        zone_identifier: read_zone_identifier(path),
        owner_sid: capture_owner_sid(path),
    }
}

fn read_zone_identifier(path: &Path) -> Option<String> {
    let stream = format!("{}:Zone.Identifier:$DATA", path.display());
    fs::read_to_string(&stream).ok()
}

fn apply_metadata(path: &Path, meta: &FileMetadata) {
    if let Some(sid) = &meta.owner_sid {
        apply_owner_sid(path, sid);
    }
    apply_timestamps(path, meta);
    apply_zone_identifier(path, meta);
    apply_attributes(path, meta); // attributes last: may set READONLY
}

fn apply_timestamps(path: &Path, meta: &FileMetadata) {
    #[cfg(windows)]
    apply_timestamps_win(path, meta);
    #[cfg(not(windows))]
    apply_timestamps_std(path, meta);
}

// FILETIME = 100-ns intervals since 1601-01-01; Unix epoch offset = 116444736000000000 100-ns intervals
#[cfg(windows)]
fn unix_secs_to_filetime(secs: u64) -> [u32; 2] {
    let ft = secs.saturating_mul(10_000_000).saturating_add(116_444_736_000_000_000u64);
    [(ft & 0xFFFF_FFFF) as u32, (ft >> 32) as u32]
}

#[cfg(windows)]
fn apply_timestamps_win(path: &Path, meta: &FileMetadata) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0x4000_0000u32,  // GENERIC_WRITE
            7u32,            // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            std::ptr::null_mut(),
            3u32,            // OPEN_EXISTING
            0x0200_0000u32,  // FILE_FLAG_BACKUP_SEMANTICS — required for directories
            std::ptr::null_mut(),
        )
    };
    if handle as isize == -1 {
        return;
    }
    let created = meta.created_secs.map(unix_secs_to_filetime);
    let accessed = meta.accessed_secs.map(unix_secs_to_filetime);
    let modified = meta.modified_secs.map(unix_secs_to_filetime);
    unsafe {
        SetFileTime(
            handle,
            created.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()),
            accessed.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()),
            modified.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()),
        );
        CloseHandle(handle);
    }
}

#[cfg(not(windows))]
fn apply_timestamps_std(path: &Path, meta: &FileMetadata) {
    use std::fs::FileTimes;
    use std::time::Duration;
    let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) else {
        return;
    };
    let mut times = FileTimes::new();
    if let Some(secs) = meta.modified_secs {
        times = times.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs));
    }
    if let Some(secs) = meta.accessed_secs {
        times = times.set_accessed(SystemTime::UNIX_EPOCH + Duration::from_secs(secs));
    }
    let _ = file.set_times(times);
}

fn apply_zone_identifier(path: &Path, meta: &FileMetadata) {
    if let Some(zone_id) = &meta.zone_identifier {
        let stream = format!("{}:Zone.Identifier:$DATA", path.display());
        let _ = fs::write(&stream, zone_id.as_bytes());
    }
}

#[cfg(windows)]
fn apply_attributes(path: &Path, meta: &FileMetadata) {
    if let Some(attrs) = meta.file_attributes {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            SetFileAttributesW(wide.as_ptr(), attrs);
        }
    }
}

#[cfg(not(windows))]
fn apply_attributes(_path: &Path, _meta: &FileMetadata) {}

#[cfg(windows)]
fn capture_owner_sid(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0x0002_0000u32,  // READ_CONTROL
            7u32,
            std::ptr::null_mut(),
            3u32,            // OPEN_EXISTING
            0x0200_0000u32,  // FILE_FLAG_BACKUP_SEMANTICS
            std::ptr::null_mut(),
        )
    };
    if handle as isize == -1 {
        return None;
    }
    let mut psid_owner: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut psd: *mut std::ffi::c_void = std::ptr::null_mut();
    let ret = unsafe {
        GetSecurityInfo(
            handle,
            1u32,           // SE_FILE_OBJECT
            0x0000_0001u32, // OWNER_SECURITY_INFORMATION
            &mut psid_owner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut psd,
        )
    };
    unsafe { CloseHandle(handle) };
    if ret != 0 {
        if !psd.is_null() {
            unsafe { LocalFree(psd) };
        }
        return None;
    }
    let mut str_ptr: *mut u16 = std::ptr::null_mut();
    let ok = unsafe { ConvertSidToStringSidW(psid_owner, &mut str_ptr) };
    if !psd.is_null() {
        unsafe { LocalFree(psd) };
    }
    if ok == 0 || str_ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    unsafe {
        while *str_ptr.add(len) != 0 {
            len += 1;
        }
    }
    let wide_slice = unsafe { std::slice::from_raw_parts(str_ptr, len) };
    let result = String::from_utf16_lossy(wide_slice);
    unsafe { LocalFree(str_ptr as *mut std::ffi::c_void) };
    Some(result)
}

#[cfg(not(windows))]
fn capture_owner_sid(_path: &Path) -> Option<String> {
    None
}

#[cfg(windows)]
fn apply_owner_sid(path: &Path, sid_str: &str) {
    use std::os::windows::ffi::OsStrExt;
    let sid_wide: Vec<u16> = sid_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut psid: *mut std::ffi::c_void = std::ptr::null_mut();
    let ok = unsafe { ConvertStringSidToSidW(sid_wide.as_ptr(), &mut psid) };
    if ok == 0 || psid.is_null() {
        return;
    }
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            path_wide.as_ptr(),
            0x0008_0000u32,  // WRITE_OWNER
            7u32,
            std::ptr::null_mut(),
            3u32,            // OPEN_EXISTING
            0x0200_0000u32,  // FILE_FLAG_BACKUP_SEMANTICS
            std::ptr::null_mut(),
        )
    };
    if handle as isize != -1 {
        unsafe {
            SetSecurityInfo(
                handle,
                1u32,           // SE_FILE_OBJECT
                0x0000_0001u32, // OWNER_SECURITY_INFORMATION
                psid,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            CloseHandle(handle);
        }
    }
    unsafe { LocalFree(psid) };
}

#[cfg(not(windows))]
fn apply_owner_sid(_path: &Path, _sid_str: &str) {}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn SetFileAttributesW(lpFileName: *const u16, dwFileAttributes: u32) -> i32;
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut std::ffi::c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn SetFileTime(
        hFile: *mut std::ffi::c_void,
        lpCreationTime: *const u32,
        lpLastAccessTime: *const u32,
        lpLastWriteTime: *const u32,
    ) -> i32;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

#[cfg(windows)]
#[link(name = "advapi32")]
extern "system" {
    fn GetSecurityInfo(
        handle: *mut std::ffi::c_void,
        ObjectType: u32,
        SecurityInfo: u32,
        ppsidOwner: *mut *mut std::ffi::c_void,
        ppsidGroup: *mut *mut std::ffi::c_void,
        ppDacl: *mut *mut std::ffi::c_void,
        ppSacl: *mut *mut std::ffi::c_void,
        ppSecurityDescriptor: *mut *mut std::ffi::c_void,
    ) -> u32;
    fn SetSecurityInfo(
        handle: *mut std::ffi::c_void,
        ObjectType: u32,
        SecurityInfo: u32,
        psidOwner: *mut std::ffi::c_void,
        psidGroup: *mut std::ffi::c_void,
        pDacl: *mut std::ffi::c_void,
        pSacl: *mut std::ffi::c_void,
    ) -> u32;
    fn ConvertSidToStringSidW(Sid: *mut std::ffi::c_void, StringSid: *mut *mut u16) -> i32;
    fn ConvertStringSidToSidW(StringSid: *const u16, Sid: *mut *mut std::ffi::c_void) -> i32;
}

fn sha256_hex(data: &[u8]) -> String {
    let d = Sha256::digest(data);
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
