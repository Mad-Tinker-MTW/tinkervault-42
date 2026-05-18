use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Crypto error: unlock failed. Wrong seed/place/date/time/star or corrupted vault.")]
    Crypto,

    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, VaultError>;
