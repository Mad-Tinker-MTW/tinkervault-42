use crate::errors::{Result, VaultError};
use std::path::{Path, PathBuf};

pub fn unique_path(path: &Path, overwrite: bool) -> Result<PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }
    if overwrite {
        return Ok(path.to_path_buf());
    }
    Err(VaultError::Message(format!("COLLISION:{}", path.display())))
}
