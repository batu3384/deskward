//! RGBA → JPEG/H264 and H264 → JPEG for Flutter display.

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;
use openh264::encoder::Encoder;
use openh264::formats::{RgbaSliceU8, YUVBuffer};

use crate::media::{Codec, VideoFrame};
use crate::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecProfile {
    Jpeg,
    H264,
    /// macOS VideoToolbox — encoded in host agent, not via openh264.
    H264Hw,
}

impl CodecProfile {
    pub fn from_env() -> Self {
        match std::env::var("DESKWARD_CODEC")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "h264-hw" | "h264_hw" => Self::H264Hw,
            "h264" => Self::H264,
            _ => Self::Jpeg,
        }
    }

    pub fn from_wire(s: &str) -> Self {
        if s.eq_ignore_ascii_case("h264-hw") || s.eq_ignore_ascii_case("h264_hw") {
            Self::H264Hw
        } else if s.eq_ignore_ascii_case("h264") {
            Self::H264
        } else {
            Self::Jpeg
        }
    }

    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::H264 | Self::H264Hw => "h264",
        }
    }
}

pub fn encode_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    profile: CodecProfile,
) -> Result<VideoFrame> {
    match profile {
        CodecProfile::Jpeg => encode_jpeg(rgba, width, height),
        CodecProfile::H264 => encode_h264(rgba, width, height),
        CodecProfile::H264Hw => Err(crate::Error::Protocol(
            "H264Hw must be encoded by platform host (VideoToolbox)".into(),
        )),
    }
}

pub(crate) fn encode_jpeg(rgba: &[u8], width: u32, height: u32) -> Result<VideoFrame> {
    let rgb = rgba_to_rgb(rgba);
    let mut jpeg = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg);
    let mut enc = JpegEncoder::new_with_quality(&mut cursor, 70);
    enc.encode(&rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(|e| crate::Error::Protocol(e.to_string()))?;
    Ok(VideoFrame {
        width,
        height,
        data: jpeg,
        codec: Codec::Jpeg,
        keyframe: true,
    })
}

fn rgba_to_rgb(rgba: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for px in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
    }
    rgb
}

fn map_h264_err<E: std::fmt::Debug>(e: E) -> crate::Error {
    crate::Error::Protocol(format!("openh264: {e:?}"))
}

fn encode_h264(rgba: &[u8], width: u32, height: u32) -> Result<VideoFrame> {
    let w = (width & !1).max(2);
    let h = (height & !1).max(2);
    let rgba = if w == width && h == height {
        rgba.to_vec()
    } else {
        crop_rgba(rgba, width, height, w, h)
    };
    let mut enc = Encoder::new().map_err(map_h264_err)?;
    let rgba_source = RgbaSliceU8::new(&rgba, (w as usize, h as usize));
    let yuv = YUVBuffer::from_rgb_source(rgba_source);
    let bitstream = enc.encode(&yuv).map_err(map_h264_err)?;
    Ok(VideoFrame {
        width: w,
        height: h,
        data: bitstream.to_vec(),
        codec: Codec::H264,
        keyframe: true,
    })
}

fn crop_rgba(rgba: &[u8], width: u32, _height: u32, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        let src = (row * width * 4) as usize;
        let dst = (row * w * 4) as usize;
        let len = (w * 4) as usize;
        out[dst..dst + len].copy_from_slice(&rgba[src..src + len]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::h264_decode::H264DecoderState;

    fn checker_rgba(w: u32, h: u32) -> Vec<u8> {
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let v = if (x + y) % 2 == 0 { 200 } else { 40 };
                rgba[i] = v;
                rgba[i + 1] = v / 2;
                rgba[i + 2] = 255 - v;
                rgba[i + 3] = 255;
            }
        }
        rgba
    }

    #[test]
    fn jpeg_roundtrip() {
        let rgba = checker_rgba(64, 64);
        let frame = encode_rgba(&rgba, 64, 64, CodecProfile::Jpeg).unwrap();
        assert_eq!(frame.codec, Codec::Jpeg);
        let mut dec = H264DecoderState::default();
        let out = dec.decode_for_display(&frame).unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn h264_roundtrip() {
        let rgba = checker_rgba(128, 128);
        let frame = encode_rgba(&rgba, 128, 128, CodecProfile::H264).unwrap();
        assert_eq!(frame.codec, Codec::H264);
        assert!(!frame.data.is_empty());
        let mut dec = H264DecoderState::default();
        let jpeg = dec.decode_for_display(&frame).unwrap();
        assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    }
}
