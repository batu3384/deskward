//! macOS screen capture via xcap (ScreenCaptureKit backend).

use deskward_core::features::frame_codec::{self, CodecProfile};
use deskward_core::media::ScreenCapture;
use deskward_core::Result;
use tracing::warn;
use xcap::Monitor;

use crate::gate::codec_profile_from_setup;
use crate::vt_h264::VtH264Encoder;

pub struct MacScreenCapture {
    monitor: Monitor,
    profile: CodecProfile,
    vt_encoder: Option<VtH264Encoder>,
}

impl MacScreenCapture {
    pub fn new() -> Result<Self> {
        let monitor = Monitor::all()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| deskward_core::Error::Protocol("no display".into()))?;
        let profile = codec_profile_from_setup();
        let vt_encoder = if profile == CodecProfile::H264Hw {
            // ponytail: dimensions fixed after first frame; session recreated on resize
            None
        } else {
            None
        };
        Ok(Self {
            monitor,
            profile,
            vt_encoder,
        })
    }

    fn encode_frame(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<deskward_core::media::VideoFrame> {
        match self.profile {
            CodecProfile::H264Hw => {
                if self.vt_encoder.is_none() {
                    self.vt_encoder = VtH264Encoder::try_new(width, height);
                }
                if let Some(enc) = self.vt_encoder.as_mut() {
                    return enc.encode_rgba(rgba, width, height);
                }
                warn!("vt encode fallback to software h264");
                frame_codec::encode_rgba(rgba, width, height, CodecProfile::H264)
            }
            other => frame_codec::encode_rgba(rgba, width, height, other),
        }
    }
}

impl ScreenCapture for MacScreenCapture {
    fn capture_frame(&mut self) -> Result<Option<deskward_core::media::VideoFrame>> {
        let image = self
            .monitor
            .capture_image()
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        let width = image.width();
        let height = image.height();
        let rgba = image.into_raw();
        self.encode_frame(&rgba, width, height).map(Some)
    }
}

impl Default for MacScreenCapture {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!(?e, "capture init failed — using fallback");
            panic!("MacScreenCapture requires display access");
        })
    }
}
