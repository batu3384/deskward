//! macOS ScreenCaptureKit capture (Faz 1 — stub until SCK wired).

use deskward_core::media::{ScreenCapture, VideoFrame};
use deskward_core::Result;
use tracing::debug;

pub struct MacScreenCapture;

impl MacScreenCapture {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenCapture for MacScreenCapture {
    fn capture_frame(&mut self) -> Result<Option<VideoFrame>> {
        debug!("MacScreenCapture: SCK not wired yet (Faz 1)");
        Ok(None)
    }
}
