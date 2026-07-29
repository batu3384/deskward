//! Host-side session handlers (clipboard, file receive, recording).

use crate::features::clipboard::ClipboardSync;
use crate::features::file_receive::FileReceiver;
use crate::features::recording::RecordingSink;

pub struct HostHandlers<'a> {
    pub clipboard: Option<&'a mut dyn ClipboardSync>,
    pub files: Option<&'a mut FileReceiver>,
    pub recording: Option<&'a mut dyn RecordingSink>,
}

impl<'a> Default for HostHandlers<'a> {
    fn default() -> Self {
        Self {
            clipboard: None,
            files: None,
            recording: None,
        }
    }
}

pub fn clipboard_enabled() -> bool {
    std::env::var("DESKWARD_CLIPBOARD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn file_receive_enabled() -> bool {
    std::env::var("DESKWARD_FILE_TRANSFER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true)
}

pub fn recording_enabled() -> bool {
    std::env::var("DESKWARD_RECORDING")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
