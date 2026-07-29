//! System clipboard via arboard.

use deskward_core::features::clipboard::{ClipboardPayload, ClipboardSync};
use deskward_core::Result;

pub struct HostClipboard;

impl HostClipboard {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl ClipboardSync for HostClipboard {
    fn push_local(&mut self) -> Result<Option<ClipboardPayload>> {
        let mut ctx = arboard::Clipboard::new()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        let text = ctx.get_text().map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        Ok(Some(ClipboardPayload {
            mime: "text/plain".into(),
            data: text.into_bytes(),
        }))
    }

    fn apply_remote(&mut self, payload: &ClipboardPayload) -> Result<()> {
        if payload.mime != "text/plain" {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&payload.data).into_owned();
        let mut ctx = arboard::Clipboard::new()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        ctx.set_text(text)
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))
    }
}
