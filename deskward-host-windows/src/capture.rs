//! Windows screen capture via xcap.

use deskward_core::features::frame_codec::{self, CodecProfile};
use deskward_core::media::ScreenCapture;
use deskward_core::Result;
use tracing::warn;
use xcap::Monitor;

use crate::gate::codec_profile_from_setup;

pub struct WinScreenCapture {
    monitor: Monitor,
    profile: CodecProfile,
}

impl WinScreenCapture {
    pub fn new() -> Result<Self> {
        let monitor = Monitor::all()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| deskward_core::Error::Protocol("no display".into()))?;
        Ok(Self {
            monitor,
            profile: codec_profile_from_setup(),
        })
    }
}

impl ScreenCapture for WinScreenCapture {
    fn capture_frame(&mut self) -> Result<Option<deskward_core::media::VideoFrame>> {
        let image = self
            .monitor
            .capture_image()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        let width = image.width();
        let height = image.height();
        let rgba = image.into_raw();
        frame_codec::encode_rgba(&rgba, width, height, self.profile).map(Some)
    }
}

impl Default for WinScreenCapture {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!(?e, "WinScreenCapture init failed");
            panic!("WinScreenCapture requires display access");
        })
    }
}
