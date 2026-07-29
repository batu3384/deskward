use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPayload {
    pub mime: String,
    pub data: Vec<u8>,
}

/// Sync clipboard between controller and host (Faz 2 impl).
pub trait ClipboardSync: Send {
    fn push_local(&mut self) -> crate::Result<Option<ClipboardPayload>>;
    fn apply_remote(&mut self, payload: &ClipboardPayload) -> crate::Result<()>;
}
