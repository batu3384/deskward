use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOffer {
    pub path: String,
    pub size: u64,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub session_id: String,
    pub offset: u64,
    pub data: Vec<u8>,
    pub final_chunk: bool,
}

/// Chunked file transfer (Faz 2 impl).
pub trait FileTransfer: Send {
    fn offer(&mut self, path: &str) -> crate::Result<FileOffer>;
    fn send_chunk(&mut self, chunk: FileChunk) -> crate::Result<()>;
}
