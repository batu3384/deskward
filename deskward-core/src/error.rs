use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("handshake failed")]
    HandshakeFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
