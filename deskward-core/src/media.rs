//! Screen capture and codec traits (Faz 1+ platform impls).

use crate::Result;

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub codec: Codec,
    pub keyframe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Jpeg,
    Raw,
    H264,
    H265,
    Vp9,
}

impl Codec {
    pub fn wire_name(self) -> &'static str {
        match self {
            Codec::Jpeg | Codec::Raw => "jpeg",
            Codec::H264 => "h264",
            Codec::H265 => "h265",
            Codec::Vp9 => "vp9",
        }
    }

    pub fn from_wire(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "h264" => Codec::H264,
            "h265" | "hevc" => Codec::H265,
            "vp9" => Codec::Vp9,
            _ => Codec::Jpeg,
        }
    }
}

pub trait ScreenCapture {
    fn capture_frame(&mut self) -> Result<Option<VideoFrame>>;
}

pub trait VideoEncoder: Send {
    fn encode(&mut self, raw: &[u8], width: u32, height: u32) -> Result<Option<VideoFrame>>;
}

pub trait VideoDecoder: Send {
    fn decode(&mut self, frame: &VideoFrame) -> Result<Vec<u8>>;
}
