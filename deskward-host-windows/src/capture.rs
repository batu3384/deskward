//! Windows DXGI/WGC capture stub (Faz 2).

use deskward_core::media::{ScreenCapture, VideoFrame};
use deskward_core::Result;
use tracing::debug;

pub struct WinScreenCapture;

impl ScreenCapture for WinScreenCapture {
    fn capture_frame(&mut self) -> Result<Option<VideoFrame>> {
        debug!("WinScreenCapture: DXGI stub (Faz 2)");
        Ok(None)
    }
}
