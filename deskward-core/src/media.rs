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
    Raw,
    H264,
    H265,
    Vp9,
}

pub trait ScreenCapture: Send {
    fn capture_frame(&mut self) -> Result<Option<VideoFrame>>;
}

pub trait VideoEncoder: Send {
    fn encode(&mut self, raw: &[u8], width: u32, height: u32) -> Result<Option<VideoFrame>>;
}

pub trait VideoDecoder: Send {
    fn decode(&mut self, frame: &VideoFrame) -> Result<Vec<u8>>;
}
