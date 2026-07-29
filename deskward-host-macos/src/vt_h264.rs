//! VideoToolbox hardware H.264 encoder (macOS only).

use apple_cf::iosurface::IOSurface;
use deskward_core::media::{Codec as MediaCodec, VideoFrame};
use deskward_core::Result;
use tracing::warn;
use videotoolbox::compression::{CompressionSession, ProfileLevel};
use videotoolbox::session::Codec as VtCodec;

const BGRA: u32 = u32::from_be_bytes(*b"BGRA");

pub struct VtH264Encoder {
    width: u32,
    height: u32,
    session: CompressionSession,
    pts: i64,
}

impl VtH264Encoder {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let w = ((width & !1).max(2)) as i32;
        let h = ((height & !1).max(2)) as i32;
        let session = CompressionSession::builder(w, h, VtCodec::H264)
            .with_real_time(true)
            .with_expected_frame_rate(10.0)
            .with_max_keyframe_interval(60)
            .with_profile_level(ProfileLevel::H264BaselineAutoLevel)
            .build()
            .map_err(|e| deskward_core::Error::Protocol(format!("vt session: {e}")))?;
        Ok(Self {
            width: w as u32,
            height: h as u32,
            session,
            pts: 0,
        })
    }

    pub fn encode_rgba(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<VideoFrame> {
        let w = (width & !1).max(2);
        let h = (height & !1).max(2);
        if w != self.width || h != self.height {
            *self = Self::new(width, height)?;
        }
        if rgba.len() != (width * height * 4) as usize {
            return Err(deskward_core::Error::Protocol(
                "rgba buffer size mismatch".into(),
            ));
        }

        let surface = IOSurface::create(w as usize, h as usize, BGRA, 4).ok_or_else(|| {
            deskward_core::Error::Protocol("IOSurface create failed".into())
        })?;
        {
            let mut guard = surface
                .lock_read_write()
                .map_err(|e| deskward_core::Error::Protocol(format!("iosurface lock: {e}")))?;
            let stride = guard.bytes_per_row();
            let dst = guard.as_slice_mut().ok_or_else(|| {
                deskward_core::Error::Protocol("iosurface not writable".into())
            })?;
            for row in 0..h as usize {
                for col in 0..w as usize {
                    let src_i = (row * width as usize + col) * 4;
                    let dst_i = row * stride + col * 4;
                    dst[dst_i] = rgba[src_i + 2];
                    dst[dst_i + 1] = rgba[src_i + 1];
                    dst[dst_i + 2] = rgba[src_i];
                    dst[dst_i + 3] = rgba[src_i + 3];
                }
            }
        }

        let encoded = self
            .session
            .encode(&surface, (self.pts, 600))
            .map_err(|e| deskward_core::Error::Protocol(format!("vt encode: {e}")))?;
        self.pts += 1;

        Ok(VideoFrame {
            width: w,
            height: h,
            data: encoded.data,
            codec: MediaCodec::H264,
            keyframe: true,
        })
    }

    pub fn try_new(width: u32, height: u32) -> Option<Self> {
        match Self::new(width, height) {
            Ok(enc) => Some(enc),
            Err(e) => {
                warn!(?e, "VideoToolbox encoder unavailable — fallback to software H264");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker_rgba(w: u32, h: u32) -> Vec<u8> {
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let v = ((x + y) % 255) as u8;
                rgba[i] = v;
                rgba[i + 1] = v / 2;
                rgba[i + 2] = 255 - v;
                rgba[i + 3] = 255;
            }
        }
        rgba
    }

    #[test]
    fn vt_h264_emits_annex_b() {
        let mut enc = VtH264Encoder::new(128, 128).expect("vt encoder");
        let rgba = checker_rgba(128, 128);
        let frame = enc.encode_rgba(&rgba, 128, 128).unwrap();
        assert_eq!(frame.codec, MediaCodec::H264);
        assert!(!frame.data.is_empty());
    }
}
